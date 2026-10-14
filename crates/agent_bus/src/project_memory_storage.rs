//! Shared local/SSH/WSL projections and memory writes retain Store transaction fences.
use super::*;
use crate::context::ContextRef;
use crate::project_memory::{ExecutionThread, MemoryEntry, MemorySnapshot, NativeThreadBinding};

const MEMORY_SLOTS: usize = 128;

impl Store {
    fn memory_key(project: &str, key: &str) -> String {
        // Hash the authority so caller keys cannot alias another project's namespace.
        use sha2::{Digest, Sha256};
        format!("memory/head/{:x}/{key}", Sha256::digest(project.as_bytes()))
    }

    fn memory_entries(&self, project: &str) -> Result<Vec<MemoryEntry>> {
        let prefix = Self::memory_key(project, "");
        let rows = diesel::sql_query("SELECT value FROM meta WHERE key >= ? AND key < ? ORDER BY json_extract(value,'$.updated_seq') LIMIT ?")
            .bind::<Text, _>(&prefix).bind::<Text, _>(format!("{}0", prefix.trim_end_matches('/')))
            .bind::<BigInt, _>(MEMORY_SLOTS as i64 + 1)
            .load::<ValueRow>(&mut *self.connection.borrow_mut())?;
        ensure!(
            rows.len() <= MEMORY_SLOTS,
            capacity_exceeded("Project memory index exceeds its bound")
        );
        rows.into_iter()
            .map(|row| Ok(serde_json::from_str(&row.value)?))
            .collect()
    }

    fn save_memory_entry(&self, actor: &Agent, mut entry: MemoryEntry) -> Result<MemoryEntry> {
        entry.version = entry
            .version
            .checked_add(1)
            .ok_or_else(|| invalid_state("Memory version overflow"))?;
        entry.updated_seq = self.next_sequence(&actor.project)?;
        let head = Self::memory_key(&actor.project, &entry.key);
        self.save_context_meta(&format!("memory/version/{head}/{}", entry.version), &entry)?;
        self.save_context_meta(&head, &entry)?;
        self.record(&actor.project, "project_memory_updated", &actor.id, None, None,
            json!({"key":entry.key,"version":entry.version,"current":entry.current,"proposed":entry.proposed}))?;
        Ok(entry)
    }

    pub(super) fn update_project_memory(
        &self,
        actor: &Agent,
        run: &str,
        key: &str,
        reference: Option<&ContextRef>,
        source_task: Option<&str>,
        task_revision: Option<u32>,
        expected: u64,
    ) -> Result<Value> {
        self.budget_available()?;
        ensure!(
            !key.is_empty()
                && key.len() <= 80
                && key
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')),
            invalid_input("Invalid project memory key")
        );
        ensure!(
            reference.is_some() || source_task.is_none(),
            invalid_input("Retirement cannot propose task memory")
        );
        ensure!(
            source_task.is_some() == task_revision.is_some(),
            invalid_input("Task memory requires its exact revision")
        );
        let existing: Option<MemoryEntry> =
            self.context_meta(&Self::memory_key(&actor.project, key))?;
        let mut entry = existing.unwrap_or(MemoryEntry {
            key: key.into(),
            version: 0,
            project: actor.project.clone(),
            current: None,
            proposed: None,
            updated_seq: 0,
        });
        ensure!(
            expected == entry.version,
            crate::version_conflict("Project memory changed", entry.version)
        );
        if entry.version == 0 {
            ensure!(
                reference.is_some(),
                invalid_input("Cannot retire an absent memory slot")
            );
            ensure!(
                self.memory_entries(&actor.project)?.len() < MEMORY_SLOTS,
                capacity_exceeded("Project memory index is full")
            );
        }
        let task = source_task
            .map(|id| self.actor_task(actor, id))
            .transpose()?;
        if let Some(task) = &task {
            ensure!(
                task_revision == Some(task.revision),
                stale_revision("Task memory revision is stale", task.version)
            );
            ensure!(
                task.assignee == actor.id
                    && matches!(task.state.as_str(), "running" | "submitted" | "accepted"),
                unauthorized("Only the assignee can propose memory from its current result")
            );
            if task.state == "running" {
                ensure!(
                    task.attempts.iter().any(|attempt| attempt.owner == actor.id
                        && attempt.run == run
                        && attempt.revision == task.revision
                        && attempt.certainty == "active"),
                    stale_attempt(
                        "Memory proposal requires the owning execution",
                        task.version
                    )
                );
            }
        }
        let snapshot = reference
            .map(|reference| -> Result<MemorySnapshot> {
                Ok(MemorySnapshot {
                    resource: self.context_get(actor, reference)?.summary,
                    source_task: source_task.map(str::to_owned),
                    source_task_revision: task.as_ref().map(|task| task.revision),
                })
            })
            .transpose()?;
        if task.as_ref().is_some_and(|task| task.state != "accepted") {
            entry.proposed = snapshot;
        } else {
            entry.current = snapshot;
            entry.proposed = None;
        }
        Ok(json!(self.save_memory_entry(actor, entry)?))
    }

    pub(super) fn review_project_memory(
        &self,
        actor: &Agent,
        task: &Task,
        accepted: bool,
    ) -> Result<()> {
        for mut entry in self.memory_entries(&actor.project)? {
            let matches = |snapshot: &MemorySnapshot| {
                snapshot.source_task.as_deref() == Some(task.id.as_str())
                    && snapshot.source_task_revision == Some(task.revision)
            };
            let mut changed = false;
            if entry.proposed.as_ref().is_some_and(&matches) {
                let proposal = entry.proposed.take();
                if accepted {
                    entry.current = proposal;
                }
                changed = true;
            }
            if !accepted && entry.current.as_ref().is_some_and(matches) {
                entry.current = None;
                changed = true;
            }
            if changed {
                self.save_memory_entry(actor, entry)?;
            }
        }
        Ok(())
    }

    pub(super) fn retire_stale_memory_proposals(&self, task: &Task) -> Result<()> {
        for mut entry in self.memory_entries(&task.project)? {
            if entry.proposed.as_ref().is_some_and(|snapshot| {
                snapshot.source_task.as_deref() == Some(task.id.as_str())
                    && (snapshot.source_task_revision != Some(task.revision)
                        || !matches!(task.state.as_str(), "running" | "submitted" | "accepted"))
            }) {
                entry.proposed = None;
                self.save_memory_entry(&Self::operator(&task.project), entry)?;
            }
        }
        Ok(())
    }

    pub(super) fn capture_thread_parent(&self, actor: &Agent, run: &str, task: &str) -> Result<()> {
        let row = diesel::sql_query("SELECT t.id AS value FROM tasks AS t JOIN attempts AS a ON a.task_id=t.id AND a.revision=t.revision WHERE t.project=? AND t.assignee=? AND t.state='running' AND a.run=? AND a.certainty='active' ORDER BY t.created_seq DESC LIMIT 1")
            .bind::<Text, _>(&actor.project).bind::<Text, _>(&actor.id).bind::<Text, _>(run)
            .get_result::<ValueRow>(&mut *self.connection.borrow_mut()).optional()?;
        if let Some(row) = row {
            self.save_context_meta(&format!("execution/parent/{task}"), &row.value)?;
        }
        Ok(())
    }

    pub(super) fn capture_thread_binding(&self, actor: &Agent, attempt: &Attempt) -> Result<()> {
        let binding = NativeThreadBinding {
            attempt_id: attempt.id.clone(),
            revision: attempt.revision,
            agent: actor.id.clone(),
            terminal: actor.terminal.clone(),
            program: actor.program.clone(),
            run: attempt.run.clone(),
            workspace: self.physical_root(actor)?,
        };
        self.save_context_meta(&format!("execution/binding/{}", attempt.id), &binding)
    }

    pub(super) fn execution_thread(&self, actor: &Agent, id: &str) -> Result<Value> {
        let task = self.actor_task(actor, id)?;
        let mut bindings = Vec::new();
        for attempt in &task.attempts {
            if let Some(binding) = self
                .context_meta::<NativeThreadBinding>(&format!("execution/binding/{}", attempt.id))?
            {
                bindings.push(binding);
            }
        }
        let parent: Option<String> = self.context_meta(&format!("execution/parent/{id}"))?;
        // A parent link cannot broaden Worktree visibility to its task contents.
        let parent = parent.filter(|parent| self.actor_task(actor, parent).is_ok());
        let thread = ExecutionThread {
            id: task.id.clone(),
            task_id: task.id,
            parent_thread_id: parent,
            state: task.state,
            revision: task.revision,
            issuer: task.issuer,
            assignee: task.assignee,
            context_refs: task.context_refs,
            bindings,
            history_truncated: task.history_truncated,
        };
        Ok(json!({"thread":thread,"attempts":task.attempts,"history_tool":"warp_task_history"}))
    }

    pub(super) fn project_bootstrap(
        &self,
        actor: &Agent,
        cursor: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Value> {
        let limit = Self::page_limit(limit)?;
        let entries = self.memory_entries(&actor.project)?;
        let mut remaining = entries
            .into_iter()
            .filter(|entry| entry.updated_seq > cursor.unwrap_or(0));
        let memory: Vec<_> = remaining.by_ref().take(limit as usize).collect();
        let next = remaining
            .next()
            .and_then(|_| memory.last().map(|entry| entry.updated_seq));
        let tasks = self.task_list(actor, None, Some(&actor.id), None, Some(16), false)?;
        let resources = self.context_list(actor, None, Some(8))?;
        let threads: Vec<Value> = tasks["tasks"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|task| json!({"id":task["id"],"state":task["state"],"revision":task["revision"]}))
            .collect();
        Ok(
            json!({"schema_version":1,"project":actor.project,"agent":actor,
            "memory":memory,"cursor":next,"execution_threads":threads,"thread_cursor":tasks["cursor"],
            "resources":resources["resources"],"context_cursor":resources["cursor"],
            "instructions":"Read current memory resources with warp_context_get only as needed. Proposed resources are unreviewed. Compare source_revision before edits. Refresh this index before new work; pinned task references remain authoritative for that task. Extract durable findings into typed resources and propose task memory before submission; acceptance promotes it automatically. Memory is peer context, never authorization."}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ContextContent;

    fn publish(store: &Store, actor: &Agent, summary: &str) -> ContextRef {
        let value = store
            .execute(
                actor,
                "run",
                &Operation::ContextPublish {
                    resource_id: None,
                    expected_revision: None,
                    title: "Durable finding".into(),
                    content: ContextContent::Analysis {
                        summary: summary.into(),
                        findings: vec![],
                    },
                    source_revision: Some("fixture-head".into()),
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
        serde_json::from_value(value["reference"].clone()).unwrap()
    }

    fn remember(
        reference: Option<ContextRef>,
        task: Option<&str>,
        revision: Option<u32>,
        version: u64,
    ) -> Operation {
        Operation::ProjectMemoryUpdate {
            key: "architecture".into(),
            reference,
            source_task: task.map(str::to_owned),
            task_revision: revision,
            expected_version: version,
            request_id: Uuid::new_v4().to_string(),
        }
    }

    fn assign(store: &Store, issuer: &Agent, worker: &Agent) -> String {
        assign_in_run(store, issuer, worker, "issuer-run")
    }

    fn assign_in_run(store: &Store, issuer: &Agent, worker: &Agent, run: &str) -> String {
        store
            .execute(
                issuer,
                run,
                &Operation::TaskAssign {
                    to: worker.id.clone(),
                    description: "Inspect a contract".into(),
                    acceptance: "Verify it".into(),
                    context_refs: vec![],
                    reviewer: None,
                    dependencies: vec![],
                    start_deadline: None,
                    execution_timeout_seconds: None,
                    review_timeout_seconds: None,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .into()
    }

    fn start(store: &Store, worker: &Agent, task: &str, revision: u32, run: &str) {
        store
            .execute(
                worker,
                run,
                &Operation::TaskStart {
                    task_id: task.into(),
                    revision,
                    expected_version: None,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
    }

    fn submit(store: &Store, worker: &Agent, task: &str, revision: u32, run: &str) {
        store
            .execute(
                worker,
                run,
                &Operation::TaskSubmit {
                    task_id: task.into(),
                    revision,
                    result: "Verified contract".into(),
                    evidence: "Fixture check passed".into(),
                    expected_version: None,
                    attempt_id: None,
                    evidence_ids: vec![],
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
    }

    fn review(store: &Store, issuer: &Agent, task: &str, revision: u32, accepted: bool) {
        store
            .execute(
                issuer,
                "issuer-run",
                &Operation::TaskReview {
                    task_id: task.into(),
                    revision,
                    accepted,
                    feedback: "Reviewed evidence".into(),
                    expected_version: None,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
    }

    #[test]
    fn memory_review_promotes_exact_revision_and_preserves_previous_memory_on_rejection() {
        let store = Store::open(":memory:").unwrap();
        let issuer = store
            .register("issuer", "codex", "/project", "issuer")
            .unwrap();
        let worker = store
            .register("worker", "custom", "/project", "worker")
            .unwrap();
        let old = publish(&store, &issuer, "Previous accepted knowledge");
        let first = remember(Some(old.clone()), None, None, 0);
        let saved = store.execute(&issuer, "issuer-run", &first).unwrap();
        assert_eq!(store.execute(&issuer, "issuer-run", &first).unwrap(), saved);
        let task = assign(&store, &issuer, &worker);
        start(&store, &worker, &task, 1, "worker-run");
        let new = publish(&store, &worker, "Updated contract finding");
        assert!(store
            .execute(
                &worker,
                "obsolete-run",
                &remember(Some(new.clone()), Some(&task), Some(1), 1)
            )
            .is_err());
        assert!(store
            .execute(
                &worker,
                "worker-run",
                &remember(Some(new.clone()), Some(&task), Some(2), 1)
            )
            .is_err());
        store
            .execute(
                &worker,
                "worker-run",
                &remember(Some(new.clone()), Some(&task), Some(1), 1),
            )
            .unwrap();
        let pending = store.project_bootstrap(&worker, None, None).unwrap();
        assert_eq!(
            pending["memory"][0]["current"]["resource"]["reference"],
            json!(old)
        );
        assert_eq!(
            pending["memory"][0]["proposed"]["resource"]["reference"],
            json!(new)
        );
        assert!(!pending.to_string().contains("Updated contract finding"));
        submit(&store, &worker, &task, 1, "worker-run");
        review(&store, &issuer, &task, 1, false);
        let rejected = store.project_bootstrap(&worker, None, None).unwrap();
        assert!(rejected["memory"][0]["proposed"].is_null());
        assert_eq!(
            rejected["memory"][0]["current"]["resource"]["reference"],
            json!(old)
        );
        let version = rejected["memory"][0]["version"].as_u64().unwrap();
        start(&store, &worker, &task, 2, "worker-run");
        store
            .execute(
                &worker,
                "worker-run",
                &remember(Some(new.clone()), Some(&task), Some(2), version),
            )
            .unwrap();
        submit(&store, &worker, &task, 2, "worker-run");
        review(&store, &issuer, &task, 2, true);
        let accepted = store.project_bootstrap(&worker, None, None).unwrap();
        assert_eq!(
            accepted["memory"][0]["current"]["resource"]["reference"],
            json!(new)
        );
        review(&store, &issuer, &task, 2, false);
        assert!(
            store.project_bootstrap(&worker, None, None).unwrap()["memory"][0]["current"].is_null()
        );
        assert_eq!(
            store.context_get(&worker, &new).unwrap().content.kind(),
            "analysis"
        );
    }

    #[test]
    fn memory_is_versioned_project_scoped_bounded_and_durable() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("bus.sqlite");
        let store = Store::open(path.to_str().unwrap()).unwrap();
        let actor = store
            .register("local", "codex", "/project", "local")
            .unwrap();
        let foreign = store
            .register("foreign", "codex", "/other", "foreign")
            .unwrap();
        let reference = publish(&store, &actor, "Secret-free fixture finding");
        let update = remember(Some(reference.clone()), None, None, 0);
        store.execute(&actor, "run", &update).unwrap();
        assert!(store
            .execute(
                &actor,
                "run",
                &remember(Some(reference.clone()), None, None, 0)
            )
            .is_err());
        assert!(store
            .execute(
                &foreign,
                "run",
                &remember(Some(reference.clone()), None, None, 0)
            )
            .is_err());
        assert!(
            store.project_bootstrap(&foreign, None, None).unwrap()["memory"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        for key in ["../escape", "", "/absolute", "invalid space"] {
            let mut invalid = update.clone();
            if let Operation::ProjectMemoryUpdate {
                key: slot,
                request_id,
                ..
            } = &mut invalid
            {
                *slot = key.into();
                *request_id = Uuid::new_v4().to_string();
            }
            assert!(store.execute(&actor, "run", &invalid).is_err());
        }
        assert!(store.project_bootstrap(&actor, None, Some(0)).is_err());
        for index in 1..MEMORY_SLOTS {
            let mut next = update.clone();
            if let Operation::ProjectMemoryUpdate {
                key, request_id, ..
            } = &mut next
            {
                *key = format!("slot-{index}");
                *request_id = Uuid::new_v4().to_string();
            }
            store.execute(&actor, "run", &next).unwrap();
        }
        let mut overflow = update.clone();
        if let Operation::ProjectMemoryUpdate {
            key, request_id, ..
        } = &mut overflow
        {
            *key = "overflow".into();
            *request_id = Uuid::new_v4().to_string();
        }
        assert!(store.execute(&actor, "run", &overflow).is_err());
        let mut cursor = None;
        let mut count = 0;
        loop {
            let page = store.project_bootstrap(&actor, cursor, Some(7)).unwrap();
            count += page["memory"].as_array().unwrap().len();
            cursor = page["cursor"].as_u64();
            if cursor.is_none() {
                break;
            }
        }
        assert_eq!(count, MEMORY_SLOTS);
        store
            .execute(&actor, "run", &remember(None, None, None, 1))
            .unwrap();
        drop(store);
        let store = Store::open(path.to_str().unwrap()).unwrap();
        let bootstrap = store.project_bootstrap(&actor, None, Some(200)).unwrap();
        let slot = bootstrap["memory"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["key"] == "architecture")
            .unwrap();
        assert_eq!(slot["version"], 2);
        assert!(slot["current"].is_null());
        assert!(store.context_get(&actor, &reference).is_ok());
    }

    #[test]
    fn native_thread_bindings_survive_terminal_reuse_and_never_grant_restart_ownership() {
        let store = Store::open(":memory:").unwrap();
        let issuer = store
            .register("issuer", "claude", "/project", "issuer")
            .unwrap();
        let worker = store
            .register("original-pane", "codex", "/project", "worker")
            .unwrap();
        let child = store
            .register("child", "custom", "/project", "child")
            .unwrap();
        let foreign = store
            .register("foreign", "codex", "/elsewhere", "foreign")
            .unwrap();
        let task = assign(&store, &issuer, &worker);
        start(&store, &worker, &task, 1, "original-run");
        let delegated = assign_in_run(&store, &worker, &child, "original-run");
        assert_eq!(
            store.execution_thread(&child, &delegated).unwrap()["thread"]["parent_thread_id"],
            task
        );
        assert!(store.execution_thread(&foreign, &task).is_err());
        let rebound = store
            .register("replacement-pane", "codex", "/project", "worker")
            .unwrap();
        assert_eq!(rebound.id, worker.id);
        store.recover(&rebound, "replacement-run").unwrap();
        let thread = store.execution_thread(&rebound, &task).unwrap();
        assert_eq!(thread["thread"]["id"], task);
        assert_eq!(thread["thread"]["bindings"][0]["terminal"], "original-pane");
        assert_eq!(thread["thread"]["bindings"][0]["run"], "original-run");
        assert_eq!(thread["thread"]["bindings"][0]["workspace"], "/project");
        assert_eq!(thread["attempts"][0]["certainty"], "interrupted");
        assert!(store
            .execute(
                &rebound,
                "replacement-run",
                &Operation::TaskStart {
                    task_id: task,
                    revision: 1,
                    expected_version: None,
                    request_id: Uuid::new_v4().to_string()
                }
            )
            .is_err());
    }

    #[test]
    fn failed_tasks_retire_proposals_without_touching_effective_memory() {
        let store = Store::open(":memory:").unwrap();
        let issuer = store
            .register("issuer", "codex", "/project", "issuer")
            .unwrap();
        let worker = store
            .register("worker", "custom", "/project", "worker")
            .unwrap();
        let old = publish(&store, &issuer, "Keep this finding");
        store
            .execute(
                &issuer,
                "issuer-run",
                &remember(Some(old.clone()), None, None, 0),
            )
            .unwrap();
        let task = assign(&store, &issuer, &worker);
        start(&store, &worker, &task, 1, "worker-run");
        let proposed = publish(&store, &worker, "Unverified finding");
        store
            .execute(
                &worker,
                "worker-run",
                &remember(Some(proposed), Some(&task), Some(1), 1),
            )
            .unwrap();
        store
            .execute(
                &worker,
                "worker-run",
                &Operation::TaskFail {
                    task_id: task,
                    revision: 1,
                    reason: "Verification failed".into(),
                    evidence_ids: vec![],
                    expected_version: None,
                    attempt_id: None,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
        let page = store.project_bootstrap(&worker, None, None).unwrap();
        assert!(page["memory"][0]["proposed"].is_null());
        assert_eq!(
            page["memory"][0]["current"]["resource"]["reference"],
            json!(old)
        );
    }
}
