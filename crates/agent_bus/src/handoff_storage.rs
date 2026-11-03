//! Handoff writes run inside the original task mutation transaction.
use super::*;
use crate::context::{ContextContent, ContextRef, ContextResource, ContextSummary};
use crate::handoff::{HandoffSnapshot, HandoffSubmission};
use crate::project_memory::NativeThreadBinding;

impl Store {
    fn handoff_status(
        &self,
        project: &str,
        reference: &ContextRef,
        snapshot: &HandoffSnapshot,
    ) -> Result<crate::handoff::HandoffStatus> {
        let task = self.operator_task_record(project, &snapshot.task_id)?;
        let head: Option<ContextSummary> =
            self.context_meta(&format!("handoff/task/{}", snapshot.task_id))?;
        let latest = diesel::sql_query(
            "SELECT id AS value FROM attempts WHERE task_id=? ORDER BY rowid DESC LIMIT 1",
        )
        .bind::<Text, _>(&snapshot.task_id)
        .get_result::<ValueRow>(&mut *self.connection.borrow_mut())
        .optional()?;
        Ok(crate::handoff::HandoffStatus {
            state:task.state, task_revision:task.revision,
            result_is_current:task.revision == snapshot.task_revision && head.is_some_and(|head| head.reference == *reference)
                && latest.is_some_and(|attempt| attempt.value == snapshot.attempt_id),
            execution_uncertain:self.count("SELECT COUNT(*) AS count FROM attempts WHERE task_id=? AND (certainty='unknown' OR (finished_at IS NULL AND certainty!='active'))", &[&snapshot.task_id])? > 0,
        })
    }

    pub(super) fn handoff_inputs(
        &self,
        task: &Task,
    ) -> Result<Vec<crate::handoff::HandoffInputStatus>> {
        let mut inputs = Vec::new();
        // Inspect only direct pins; source status never assembles another task or expands its inputs.
        for reference in &task.context_refs {
            let resource: Option<ContextResource> = self.context_meta(&format!(
                "context/version/{}/{}",
                reference.id, reference.revision
            ))?;
            let Some(resource) = resource else {
                continue;
            };
            if resource.summary.kind != "handoff" {
                continue;
            }
            let valid =
                resource.summary.project == task.project && resource.verify(reference).is_ok();
            let mut input = crate::handoff::HandoffInputStatus {
                reference: reference.clone(),
                source_task: None,
                source_task_revision: None,
                status: None,
            };
            if valid {
                if let ContextContent::Handoff { snapshot } = resource.content {
                    input.source_task = Some(snapshot.task_id.clone());
                    input.source_task_revision = Some(snapshot.task_revision);
                    input.status = self
                        .handoff_status(&task.project, reference, &snapshot)
                        .ok();
                }
            }
            inputs.push(input);
        }
        Ok(inputs)
    }

    pub(super) fn handoff_source(
        &self,
        actor: &Agent,
        reference: &ContextRef,
        part: &crate::handoff::HandoffPart,
    ) -> Result<Value> {
        use crate::handoff::HandoffPart;
        let resource = self.context_get(actor, reference)?;
        let ContextContent::Handoff { snapshot } = resource.content else {
            return Err(invalid_input("Source parts require a handoff reference"));
        };
        let source = match part {
            HandoffPart::Status => {
                json!(self.handoff_status(&actor.project, reference, &snapshot)?)
            }
            HandoffPart::Task => {
                let task = self.operator_task(&actor.project, &snapshot.task_id)?;
                json!({"objective":task.description,"acceptance":task.acceptance})
            }
            HandoffPart::Result => {
                let row = diesel::sql_query("SELECT payload AS value FROM events WHERE id=? AND project=? AND resource=? AND attempt=? AND sequence=?")
                    .bind::<Text, _>(&snapshot.source_event_id).bind::<Text, _>(&actor.project)
                    .bind::<Text, _>(&snapshot.task_id).bind::<Text, _>(&snapshot.attempt_id)
                    .bind::<BigInt, _>(snapshot.source_event_sequence as i64)
                    .get_result::<ValueRow>(&mut *self.connection.borrow_mut())?;
                let event: Value = serde_json::from_str(&row.value)?;
                let mut result = serde_json::Map::new();
                for key in [
                    "revision",
                    "result",
                    "evidence",
                    "reason",
                    "note",
                    "waiting_reason",
                ] {
                    if let Some(value) = event.get(key) {
                        result.insert(key.into(), value.clone());
                    }
                }
                Value::Object(result)
            }
            HandoffPart::Evidence { id } => {
                ensure!(
                    snapshot.evidence_ids.contains(id),
                    scope_denied("Evidence is outside this handoff")
                );
                let evidence = self
                    .task_evidence(&snapshot.task_id)?
                    .into_iter()
                    .find(|evidence| &evidence.id == id)
                    .ok_or_else(|| invalid_state("Handoff evidence is unavailable"))?;
                let attempt = if let Some(id) = &evidence.attempt_id {
                    diesel::sql_query("SELECT id, task_id, revision, owner, run, certainty, outcome, started_at, finished_at FROM attempts WHERE id=? AND task_id=?")
                        .bind::<Text, _>(id).bind::<Text, _>(&snapshot.task_id)
                        .get_result::<AttemptRow>(&mut *self.connection.borrow_mut()).optional()?.map(AttemptRow::attempt)
                } else {
                    None
                };
                let binding: Option<NativeThreadBinding> = match &attempt {
                    Some(attempt) => {
                        self.context_meta(&format!("execution/binding/{}", attempt.id))?
                    }
                    None => None,
                };
                let test_outcome_source = if evidence.kind == "test" {
                    "agent_reported"
                } else {
                    "not_applicable"
                };
                json!(crate::handoff::HandoffEvidence {
                    same_attempt: evidence.attempt_id.as_deref() == Some(&snapshot.attempt_id),
                    reported_by: attempt.as_ref().map(|attempt| attempt.owner.clone()),
                    task_revision: attempt.as_ref().map(|attempt| attempt.revision),
                    attempt_available: attempt.is_some(),
                    binding,
                    verification_is_current: true,
                    evidence,
                    test_outcome_source: test_outcome_source.into(),
                    review_is_separate: true,
                })
            }
        };
        let value = json!({"reference":reference,"task_id":snapshot.task_id,"task_revision":snapshot.task_revision,
            "attempt_id":snapshot.attempt_id,"part":part,"source":source});
        ensure!(
            serde_json::to_vec(&value)?.len() <= 48 * 1024,
            capacity_exceeded(
                "Handoff source part exceeds 48 KiB; inspect authorized source evidence directly"
            )
        );
        Ok(value)
    }

    pub(super) fn purge_task_handoff(&self, task: &str) -> Result<()> {
        let head: Option<ContextSummary> = self.context_meta(&format!("handoff/task/{task}"))?;
        if let Some(head) = head {
            let prefix = format!("context/version/{}/", head.reference.id);
            diesel::sql_query("DELETE FROM meta WHERE key >= ? AND key < ?")
                .bind::<Text, _>(&prefix)
                .bind::<Text, _>(format!("context/version/{}0", head.reference.id))
                .execute(&mut *self.connection.borrow_mut())?;
            for key in [
                format!("context/head/{}", head.reference.id),
                format!("handoff/task/{task}"),
            ] {
                diesel::sql_query("DELETE FROM meta WHERE key=?")
                    .bind::<Text, _>(key)
                    .execute(&mut *self.connection.borrow_mut())?;
            }
            for prefix in [
                format!("handoff/shared/{}/", head.reference.id),
                format!("handoff/messages/{}/", head.reference.id),
            ] {
                diesel::sql_query("DELETE FROM meta WHERE key >= ? AND key < ?")
                    .bind::<Text, _>(&prefix)
                    .bind::<Text, _>(format!("{}0", prefix.trim_end_matches('/')))
                    .execute(&mut *self.connection.borrow_mut())?;
            }
        }
        Ok(())
    }

    pub(super) fn context_visible(&self, actor: &Agent, summary: &ContextSummary) -> Result<bool> {
        if summary.project != actor.project {
            return Ok(false);
        }
        let Some(source) = &summary.source_task else {
            return Ok(summary.kind != "handoff");
        };
        let visible = |id: &str| self.actor_task_record(actor, id).is_ok();
        if visible(source) {
            return Ok(true);
        }
        let targets: Vec<String> = self
            .context_meta(&format!(
                "handoff/shared/{}/{}",
                summary.reference.id, summary.reference.revision
            ))?
            .unwrap_or_default();
        for target in targets {
            if visible(&target) {
                return Ok(true);
            }
        }
        let messages: Vec<String> = self
            .context_meta(&format!(
                "handoff/messages/{}/{}",
                summary.reference.id, summary.reference.revision
            ))?
            .unwrap_or_default();
        for id in messages {
            if self
                .visible_message(actor, &id)
                .is_ok_and(|message| message.to == actor.id)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(super) fn share_handoff_message(
        &self,
        actor: &Agent,
        reference: &ContextRef,
        message: &Message,
    ) -> Result<()> {
        let resource = self.context_get(actor, reference)?;
        if resource.summary.kind != "handoff" {
            return Ok(());
        }
        let key = format!("handoff/messages/{}/{}", reference.id, reference.revision);
        let mut ids: Vec<String> = self.context_meta(&key)?.unwrap_or_default();
        if !ids.contains(&message.id) {
            ids.push(message.id.clone());
        }
        ensure!(
            ids.len() <= 1000,
            capacity_exceeded("Handoff share limit reached")
        );
        self.save_context_meta(&key, &ids)?;
        self.save_context_meta(
            &format!("handoff/retain/{}/message/{}", resource.summary.source_task.as_deref().ok_or_else(|| invalid_state("Handoff source is missing"))?, message.id),
            &json!({"project":actor.project,"source_task":resource.summary.source_task,"message":message.id}),
        )
    }

    pub(super) fn current_handoff(&self, task: &Task) -> Result<Option<ContextRef>> {
        let head: Option<ContextSummary> =
            self.context_meta(&format!("handoff/task/{}", task.id))?;
        let Some(head) = head else {
            return Ok(None);
        };
        let resource: ContextResource = self
            .context_meta(&format!(
                "context/version/{}/{}",
                head.reference.id, head.reference.revision
            ))?
            .ok_or_else(|| invalid_state("Handoff snapshot is missing"))?;
        if let ContextContent::Handoff { snapshot } = resource.content {
            if snapshot.task_revision == task.revision
                && task
                    .attempts
                    .last()
                    .is_some_and(|attempt| attempt.id == snapshot.attempt_id)
            {
                return Ok(Some(head.reference));
            }
        }
        Ok(None)
    }

    pub(super) fn save_handoff(
        &self,
        actor: &Agent,
        task: &Task,
        attempt: &Attempt,
        request_id: &str,
        reason: &str,
        event_sequence: u64,
        preview: &str,
        evidence_ids: &[String],
        submission: Option<&HandoffSubmission>,
    ) -> Result<ContextRef> {
        let agent_report = submission.cloned().unwrap_or_default();
        agent_report.validate()?;
        let decision_evidence: Vec<String> = match &agent_report.decisions {
            crate::handoff::HandoffSection::Provided(items) => items
                .iter()
                .flat_map(|decision| decision.evidence_ids.clone())
                .collect(),
            _ => vec![],
        };
        self.validate_evidence_ids(task, &decision_evidence)?;
        let event = diesel::sql_query("SELECT id AS value FROM events WHERE project=? AND sequence=? AND resource=? AND attempt=?")
            .bind::<Text, _>(&task.project).bind::<BigInt, _>(event_sequence as i64)
            .bind::<Text, _>(&task.id).bind::<Text, _>(&attempt.id)
            .get_result::<ValueRow>(&mut *self.connection.borrow_mut())?;
        let binding: Option<NativeThreadBinding> =
            self.context_meta(&format!("execution/binding/{}", attempt.id))?;
        let snapshot = HandoffSnapshot {
            schema_version: 1,
            task_id: task.id.clone(),
            task_revision: task.revision,
            execution_thread_id: task.id.clone(),
            attempt_id: attempt.id.clone(),
            producer: actor.id.clone(),
            binding: binding.clone(),
            request_id: request_id.into(),
            reason: reason.into(),
            source_event_id: event.value,
            source_event_sequence: event_sequence,
            result_preview: preview.chars().take(512).collect(),
            preview_truncated: preview.chars().count() > 512,
            evidence_ids: task
                .evidence_records
                .iter()
                .filter(|evidence| {
                    evidence.attempt_id.as_deref() == Some(&attempt.id)
                        || evidence_ids.contains(&evidence.id)
                        || decision_evidence.contains(&evidence.id)
                })
                .map(|evidence| evidence.id.clone())
                .collect(),
            input_context_refs: task.context_refs.clone(),
            agent_report,
        };
        let head: Option<ContextSummary> =
            self.context_meta(&format!("handoff/task/{}", task.id))?;
        let title = format!(
            "Handoff {} · {}",
            task.id.chars().take(8).collect::<String>(),
            task.description
                .chars()
                .filter(|character| !character.is_control())
                .take(40)
                .collect::<String>()
        );
        let value = self.persist_context(
            actor,
            head.as_ref().map(|head| head.reference.id.as_str()),
            head.as_ref().map(|head| head.reference.revision),
            &title,
            &ContextContent::Handoff {
                snapshot: Box::new(snapshot),
            },
            None,
        )?;
        let mut summary: ContextSummary = serde_json::from_value(value)?;
        // Provenance follows TaskStart, never the producer's current cwd.
        summary.source_workspace = binding.map(|binding| binding.workspace);
        let key = format!(
            "context/version/{}/{}",
            summary.reference.id, summary.reference.revision
        );
        let mut resource: ContextResource = self
            .context_meta(&key)?
            .ok_or_else(|| invalid_state("Handoff publication failed"))?;
        resource.summary = summary.clone();
        self.save_context_meta(&key, &resource)?;
        self.save_context_meta(&format!("context/head/{}", summary.reference.id), &summary)?;
        self.save_context_meta(&format!("handoff/task/{}", task.id), &summary)?;
        self.record(&task.project, "handoff_created", &actor.id, Some(&task.id), Some(&attempt.id),
            json!({"reference":summary.reference,"revision":task.revision,"reason":reason,"source_event_sequence":event_sequence}))?;
        Ok(summary.reference)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handoff::HandoffSection;

    fn assign(store: &Store, issuer: &Agent, worker: &Agent, refs: Vec<ContextRef>) -> String {
        store
            .execute(
                issuer,
                "issuer-run",
                &Operation::TaskAssign {
                    to: worker.id.clone(),
                    description: "Check the contract".into(),
                    acceptance: "Report evidence".into(),
                    context_refs: refs,
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
    fn start(store: &Store, worker: &Agent, task: &str, revision: u32) {
        store
            .execute(
                worker,
                "worker-run",
                &Operation::TaskStart {
                    task_id: task.into(),
                    revision,
                    expected_version: None,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
    }
    fn submit(task: &str, revision: u32) -> Operation {
        Operation::TaskSubmit {
            handoff: None,
            task_id: task.into(),
            revision,
            result: "Contract checked".into(),
            evidence: "Agent-reported test passed".into(),
            evidence_ids: vec![],
            attempt_id: None,
            expected_version: None,
            request_id: Uuid::new_v4().to_string(),
        }
    }
    fn reference(value: &Value) -> ContextRef {
        serde_json::from_value(value["handoff_ref"].clone()).unwrap()
    }
    fn actors(store: &Store) -> (Agent, Agent) {
        (
            store
                .register("issuer", "codex", "/project", "issuer")
                .unwrap(),
            store
                .register("worker", "custom", "/project", "worker")
                .unwrap(),
        )
    }

    #[test]
    fn handoff_is_immutable_idempotent_bound_to_attempt_and_durable() {
        let dir = tempfile::tempdir().unwrap();
        let database = dir.path().join("bus.sqlite");
        let store = Store::open(database.to_str().unwrap()).unwrap();
        let (issuer, worker) = actors(&store);
        let task = assign(&store, &issuer, &worker, vec![]);
        start(&store, &worker, &task, 1);
        let progress = Operation::TaskProgress {
            handoff: None,
            task_id: task.clone(),
            revision: 1,
            note: "Working".into(),
            waiting_reason: None,
            attempt_id: None,
            expected_version: None,
            request_id: Uuid::new_v4().to_string(),
        };
        assert!(store
            .execute(&worker, "worker-run", &progress)
            .unwrap()
            .get("handoff_ref")
            .is_none());
        let mut checkpoint = progress.clone();
        if let Operation::TaskProgress {
            handoff,
            request_id,
            ..
        } = &mut checkpoint
        {
            *handoff = Some(HandoffSubmission {
                next_steps: HandoffSection::Provided(vec!["Review evidence".into()]),
                ..Default::default()
            });
            *request_id = Uuid::new_v4().to_string();
        }
        let first = store.execute(&worker, "worker-run", &checkpoint).unwrap();
        let pin = reference(&first);
        assert_eq!(first["state"], "running");
        assert_eq!(
            store.execute(&worker, "worker-run", &checkpoint).unwrap(),
            first
        );
        let operation = submit(&task, 1);
        assert!(store.execute(&worker, "stale-run", &operation).is_err());
        let submitted = store.execute(&worker, "worker-run", &operation).unwrap();
        let second = reference(&submitted);
        assert_eq!(pin.id, second.id);
        assert_eq!(second.revision, 2);
        let history = store
            .task_history_filtered(&issuer, &task, None, Some(1), true)
            .unwrap();
        assert_eq!(history["events"][0]["kind"], "handoff_created");
        assert_eq!(history["events"][0]["payload"]["reference"], json!(pin));
        let next = store
            .task_history_filtered(&issuer, &task, history["cursor"].as_u64(), Some(1), true)
            .unwrap();
        assert_eq!(next["events"][0]["payload"]["reference"], json!(second));
        assert_eq!(
            store
                .handoff_source(&worker, &pin, &crate::handoff::HandoffPart::Status)
                .unwrap()["source"]["result_is_current"],
            false
        );

        assert_eq!(
            store.execute(&worker, "worker-run", &operation).unwrap(),
            submitted
        );
        let mut conflict = operation.clone();
        if let Operation::TaskSubmit { result, .. } = &mut conflict {
            *result = "Changed".into();
        }
        assert!(store.execute(&worker, "worker-run", &conflict).is_err());
        let old = store.context_get(&worker, &pin).unwrap();
        let ContextContent::Handoff { snapshot } = &old.content else {
            panic!("Missing brief");
        };
        assert_eq!(snapshot.binding.as_ref().unwrap().run, "worker-run");
        assert_eq!(snapshot.reason, "checkpoint");
        assert_eq!(snapshot.producer, worker.id);
        let forged = Operation::ContextPublish {
            resource_id: Some(pin.id.clone()),
            expected_revision: Some(2),
            title: "Forged".into(),
            content: old.content,
            source_revision: None,
            request_id: Uuid::new_v4().to_string(),
        };
        assert!(store.execute(&worker, "worker-run", &forged).is_err());
        let review = Operation::TaskReview {
            task_id: task.clone(),
            revision: 1,
            accepted: false,
            feedback: "Need another check".into(),
            expected_version: None,
            request_id: Uuid::new_v4().to_string(),
        };
        store.execute(&issuer, "issuer-run", &review).unwrap();
        assert!(store.task(&issuer, &task).unwrap().handoff_ref.is_none());
        assert!(store.context_get(&issuer, &second).is_ok());
        start(&store, &worker, &task, 2);
        assert!(store
            .execute(&worker, "worker-run", &submit(&task, 1))
            .is_err());
        assert!(store.task(&worker, &task).unwrap().handoff_ref.is_none());
        drop(store);
        let store = Store::open(database.to_str().unwrap()).unwrap();
        assert!(store.context_get(&issuer, &pin).is_ok());
        assert!(store.context_get(&issuer, &second).is_ok());
    }

    #[test]
    fn handoff_reads_inherit_task_scope_and_explicit_shares_do_not_grant_source_access() {
        let store = Store::open(":memory:").unwrap();
        let (issuer, worker) = actors(&store);
        let peer = store
            .register("peer", "custom", "/project", "peer")
            .unwrap();
        let foreign = store
            .register("foreign", "custom", "/foreign", "foreign")
            .unwrap();
        let source = assign(&store, &issuer, &worker, vec![]);
        start(&store, &worker, &source, 1);
        let pin = reference(
            &store
                .execute(&worker, "worker-run", &submit(&source, 1))
                .unwrap(),
        );
        store.set_worktree_mode("/project", true).unwrap();
        assert!(store.context_get(&peer, &pin).is_err());
        assert!(store.context_get(&foreign, &pin).is_err());
        assert!(store.context_list(&peer, None, None).unwrap()["resources"]
            .as_array()
            .unwrap()
            .is_empty());
        store.set_worktree_mode("/project", false).unwrap();
        let target = assign(&store, &issuer, &peer, vec![pin.clone()]);
        store.set_worktree_mode("/project", true).unwrap();
        assert!(store.task(&peer, &source).is_err());
        assert!(store.context_get(&peer, &pin).is_ok());
        let result = store
            .handoff_source(&peer, &pin, &crate::handoff::HandoffPart::Result)
            .unwrap();
        assert_eq!(result["source"]["result"], "Contract checked");
        let status = store
            .handoff_source(&peer, &pin, &crate::handoff::HandoffPart::Status)
            .unwrap();
        assert_eq!(status["source"]["state"], "submitted");
        assert!(store
            .handoff_source(
                &peer,
                &pin,
                &crate::handoff::HandoffPart::Evidence {
                    id: Uuid::new_v4().to_string()
                }
            )
            .is_err());
        assert!(store.task_history(&peer, &source, None, None).is_err());
        assert!(store
            .task_history_filtered(&peer, &source, None, None, true)
            .is_err());
        assert_eq!(
            store.task(&peer, &target).unwrap().context_refs,
            vec![pin.clone()]
        );
        let mut tampered = pin.clone();
        tampered.sha256 = "0".repeat(64);
        assert!(store.context_get(&peer, &tampered).is_err());
        let mut newer = pin.clone();
        newer.revision += 1;
        assert!(store.context_get(&peer, &newer).is_err());
        assert!(store.context_get(&worker, &pin).is_ok());
        store.set_worktree_mode("/project", false).unwrap();
        store
            .execute(
                &issuer,
                "issuer-run",
                &Operation::TaskReview {
                    task_id: source.clone(),
                    revision: 1,
                    accepted: false,
                    feedback: "Revalidate the delivery".into(),
                    expected_version: None,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
        store.set_worktree_mode("/project", true).unwrap();
        let target = store.task(&peer, &target).unwrap();
        assert_eq!(target.state, "queued");
        assert_eq!(target.context_refs, vec![pin.clone()]);
        assert!(target.handoff_inputs[0].requires_revalidation());
        assert_eq!(
            store.project_bootstrap(&peer, None, None).unwrap()["execution_threads"][0]
                ["handoff_warning_count"],
            1
        );
        assert!(store.context_get(&peer, &pin).is_ok());
        assert!(store.task(&peer, &source).is_err());
    }

    #[test]
    fn handoff_partial_outcomes_keep_test_reports_and_task_semantics() {
        for cancelled in [false, true] {
            let store = Store::open(":memory:").unwrap();
            let (issuer, worker) = actors(&store);
            let task = assign(&store, &issuer, &worker, vec![]);
            start(&store, &worker, &task, 1);
            let added = store
                .execute(
                    &worker,
                    "worker-run",
                    &Operation::EvidenceAdd {
                        task_id: task.clone(),
                        kind: "test".into(),
                        attempt_id: None,
                        path: None,
                        hash: None,
                        commit: None,
                        repository: None,
                        branch: None,
                        base: None,
                        head: None,
                        command: Some("fixture-check".into()),
                        outcome: Some("passed".into()),
                        exit_code: Some(0),
                        summary: Some("Agent report, no independent execution".into()),
                        request_id: Uuid::new_v4().to_string(),
                    },
                )
                .unwrap();
            let operation = if cancelled {
                store
                    .execute(
                        &issuer,
                        "issuer-run",
                        &Operation::TaskCancel {
                            task_id: task.clone(),
                            reason: "Stop requested".into(),
                            expected_version: None,
                            request_id: Uuid::new_v4().to_string(),
                        },
                    )
                    .unwrap();
                Operation::TaskFinishCancel {
                    handoff: None,
                    task_id: task.clone(),
                    revision: 1,
                    reason: "Stopped with partial evidence".into(),
                    attempt_id: None,
                    expected_version: None,
                    request_id: Uuid::new_v4().to_string(),
                }
            } else {
                Operation::TaskFail {
                    handoff: None,
                    task_id: task.clone(),
                    revision: 1,
                    reason: "Could not complete".into(),
                    evidence_ids: vec![],
                    attempt_id: None,
                    expected_version: None,
                    request_id: Uuid::new_v4().to_string(),
                }
            };
            let result = store.execute(&worker, "worker-run", &operation).unwrap();
            assert_eq!(
                result["state"],
                if cancelled { "cancelled" } else { "failed" }
            );
            let pin = reference(&result);
            let id = added["evidence_id"].as_str().unwrap().to_owned();
            let evidence = store
                .handoff_source(&issuer, &pin, &crate::handoff::HandoffPart::Evidence { id })
                .unwrap();
            assert_eq!(evidence["source"]["reported_by"], worker.id);
            assert_eq!(evidence["source"]["verification_is_current"], true);
            assert_eq!(evidence["source"]["evidence"]["verified"], false);
            assert_eq!(evidence["source"]["evidence"]["outcome"], "passed");
            assert_eq!(evidence["source"]["test_outcome_source"], "agent_reported");
            assert_eq!(
                store.execute(&worker, "worker-run", &operation).unwrap(),
                result
            );
        }
    }

    #[test]
    fn handoff_message_sharing_is_explicit_version_scoped_and_non_recursive() {
        let store = Store::open(":memory:").unwrap();
        let (issuer, worker) = actors(&store);
        let peer = store
            .register("peer", "custom", "/project", "peer")
            .unwrap();
        let reader = store
            .register("reader", "custom", "/project", "reader")
            .unwrap();
        let input_task = assign(&store, &issuer, &worker, vec![]);
        start(&store, &worker, &input_task, 1);
        let input = reference(
            &store
                .execute(&worker, "worker-run", &submit(&input_task, 1))
                .unwrap(),
        );
        let task = assign(&store, &issuer, &worker, vec![input.clone()]);
        start(&store, &worker, &task, 1);
        let pin = reference(
            &store
                .execute(&worker, "worker-run", &submit(&task, 1))
                .unwrap(),
        );
        let request: Operation = serde_json::from_value(json!({"op":"agent_send", "to":worker.id,
            "body":"Return the registered handoff", "task_id":task, "expects_result":true,
            "request_id":Uuid::new_v4().to_string()}))
        .unwrap();
        let message = store.execute(&peer, "peer-run", &request).unwrap();
        let reply = store
            .execute(
                &worker,
                "worker-run",
                &Operation::AgentReply {
                    handoff_ref: Some(pin.clone()),
                    reply_to: message["id"].as_str().unwrap().into(),
                    body: "Read the pinned result".into(),
                    outcome: crate::ReplyOutcome::Succeeded,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
        assert!(reply["body"]["details"]["instruction"]
            .as_str()
            .unwrap()
            .contains(&pin.uri()));
        assert!(reply["body"]["details"].get("handoff_ref").is_none());
        let send: Operation = serde_json::from_value(json!({"op":"agent_send", "to":reader.id,
            "content":{"objective":"Consume the exact handoff", "context_refs":[pin]},
            "request_id":Uuid::new_v4().to_string()}))
        .unwrap();
        store.execute(&worker, "worker-run", &send).unwrap();
        store.set_worktree_mode("/project", true).unwrap();
        for actor in [&peer, &reader] {
            assert!(store.task(actor, &task).is_err());
            assert!(store.context_get(actor, &pin).is_ok());
            assert!(store.context_get(actor, &input).is_err());
            let list = store.context_list(actor, None, None).unwrap();
            assert_eq!(list["resources"].as_array().unwrap().len(), 1);
            assert_eq!(list["resources"][0]["reference"], json!(pin));
        }
        store.set_worktree_mode("/project", false).unwrap();
        let independent = assign(&store, &issuer, &worker, vec![]);
        start(&store, &worker, &independent, 1);
        let independent_pin = reference(
            &store
                .execute(&worker, "worker-run", &submit(&independent, 1))
                .unwrap(),
        );
        let multiple: Operation = serde_json::from_value(json!({"op":"agent_send", "to":reader.id,
            "content":{"objective":"Read both fixed versions", "context_refs":[independent_pin, input]},
            "request_id":Uuid::new_v4().to_string()})).unwrap();
        store.execute(&worker, "worker-run", &multiple).unwrap();
        store
            .execute(
                &issuer,
                "issuer-run",
                &Operation::TaskCancel {
                    task_id: independent.clone(),
                    reason: "Fixture done".into(),
                    expected_version: None,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
        store
            .task_archive("/project", &issuer, &independent)
            .unwrap();
        store
            .connection
            .borrow_mut()
            .batch_execute("UPDATE messages SET acknowledged=1")
            .unwrap();
        assert_eq!(
            store.purge_preview("/project").unwrap()["handoff_protected_tasks"],
            1
        );
        store
            .transaction(|| store.history_purge("/project", &issuer, true, false))
            .unwrap();
        assert!(store.context_get(&issuer, &independent_pin).is_ok());
    }

    #[test]
    fn handoff_pins_protect_source_history_until_the_target_is_purged() {
        let store = Store::open(":memory:").unwrap();
        let (issuer, worker) = actors(&store);
        let source = assign(&store, &issuer, &worker, vec![]);
        start(&store, &worker, &source, 1);
        let pin = reference(
            &store
                .execute(&worker, "worker-run", &submit(&source, 1))
                .unwrap(),
        );
        let request: Operation = serde_json::from_value(json!({"op":"agent_send", "to":worker.id,
            "body":"Return the source pin", "task_id":source, "expects_result":true,
            "request_id":Uuid::new_v4().to_string()}))
        .unwrap();
        let message = store.execute(&issuer, "issuer-run", &request).unwrap();
        store
            .execute(
                &worker,
                "worker-run",
                &Operation::AgentReply {
                    handoff_ref: Some(pin.clone()),
                    reply_to: message["id"].as_str().unwrap().into(),
                    body: "Pinned result".into(),
                    outcome: crate::ReplyOutcome::Succeeded,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
        store
            .execute(
                &issuer,
                "issuer-run",
                &Operation::TaskReview {
                    task_id: source.clone(),
                    revision: 1,
                    accepted: true,
                    feedback: "Accepted".into(),
                    expected_version: None,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
        let target = assign(&store, &issuer, &worker, vec![pin.clone()]);
        store.task_archive("/project", &issuer, &source).unwrap();
        store
            .connection
            .borrow_mut()
            .batch_execute("UPDATE messages SET acknowledged=1")
            .unwrap();
        assert_eq!(
            store.purge_preview("/project").unwrap()["handoff_protected_tasks"],
            1
        );
        store
            .transaction(|| store.history_purge("/project", &issuer, true, true))
            .unwrap();
        assert!(store.context_get(&worker, &pin).is_ok());
        store
            .execute(
                &issuer,
                "issuer-run",
                &Operation::TaskCancel {
                    task_id: target.clone(),
                    reason: "Fixture complete".into(),
                    expected_version: None,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
        store.task_archive("/project", &issuer, &target).unwrap();
        store
            .connection
            .borrow_mut()
            .batch_execute("UPDATE messages SET acknowledged=1")
            .unwrap();
        store
            .transaction(|| store.history_purge("/project", &issuer, true, true))
            .unwrap();
        assert!(store.context_get(&worker, &pin).is_ok());
        store
            .transaction(|| store.history_purge("/project", &issuer, true, true))
            .unwrap();
        assert!(store.context_get(&worker, &pin).is_err());
    }

    #[test]
    fn handoff_evidence_keeps_its_original_reporter_after_reassignment() {
        let store = Store::open(":memory:").unwrap();
        let (issuer, worker) = actors(&store);
        let replacement = store
            .register("replacement", "custom", "/project", "replacement")
            .unwrap();
        let task = assign(&store, &issuer, &worker, vec![]);
        start(&store, &worker, &task, 1);
        let add: Operation = serde_json::from_value(json!({"op":"evidence_add", "task_id":task,
            "kind":"test", "command":"original-fixture", "outcome":"passed", "exit_code":0,
            "request_id":Uuid::new_v4().to_string()}))
        .unwrap();
        let evidence = store.execute(&worker, "worker-run", &add).unwrap()["evidence_id"]
            .as_str()
            .unwrap()
            .to_owned();
        store
            .execute(
                &worker,
                "worker-run",
                &Operation::TaskFail {
                    handoff: None,
                    task_id: task.clone(),
                    revision: 1,
                    reason: "Partial delivery".into(),
                    evidence_ids: vec![evidence.clone()],
                    attempt_id: None,
                    expected_version: None,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
        store
            .execute(
                &issuer,
                "issuer-run",
                &Operation::TaskReassign {
                    task_id: task.clone(),
                    assignee: replacement.id.clone(),
                    reviewer: None,
                    reason: "Continue the stopped task".into(),
                    start_deadline: None,
                    clear_start_deadline: false,
                    override_uncertain: false,
                    expected_version: None,
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
        let revision = store.task(&replacement, &task).unwrap().revision;
        start(&store, &replacement, &task, revision);
        let mut operation = submit(&task, revision);
        if let Operation::TaskSubmit { evidence_ids, .. } = &mut operation {
            evidence_ids.push(evidence.clone());
        }
        let pin = reference(
            &store
                .execute(&replacement, "worker-run", &operation)
                .unwrap(),
        );
        let source = store
            .handoff_source(
                &issuer,
                &pin,
                &crate::handoff::HandoffPart::Evidence { id: evidence },
            )
            .unwrap();
        assert_eq!(source["source"]["reported_by"], worker.id);
        assert_eq!(source["source"]["binding"]["agent"], worker.id);
        assert_eq!(source["source"]["task_revision"], 1);
        assert_eq!(source["source"]["same_attempt"], false);
        assert!(revision > 1);
    }

    #[test]
    fn handoff_keeps_maximum_legacy_results_readable_without_copying_them_into_content() {
        let store = Store::open(":memory:").unwrap();
        let (issuer, worker) = actors(&store);
        let task = assign(&store, &issuer, &worker, vec![]);
        start(&store, &worker, &task, 1);
        let mut operation = submit(&task, 1);
        if let Operation::TaskSubmit {
            result, evidence, ..
        } = &mut operation
        {
            *result = "\"".repeat(crate::MAX_TEXT);
            *evidence = "\"".repeat(crate::MAX_TEXT);
        }
        let pin = reference(&store.execute(&worker, "worker-run", &operation).unwrap());
        let resource = store.context_get(&issuer, &pin).unwrap();
        assert!(serde_json::to_vec(&resource.content).unwrap().len() <= 32 * 1024);
        let ContextContent::Handoff { snapshot } = resource.content else {
            panic!("Missing brief");
        };
        assert!(snapshot.preview_truncated);
        let part = store
            .handoff_source(&issuer, &pin, &crate::handoff::HandoffPart::Result)
            .unwrap();
        assert_eq!(
            part["source"]["result"].as_str().unwrap().len(),
            crate::MAX_TEXT
        );
        assert!(serde_json::to_vec(&part).unwrap().len() <= 48 * 1024);
    }

    #[test]
    fn handoff_generation_does_not_consume_input_pin_capacity() {
        let store = Store::open(":memory:").unwrap();
        let (issuer, worker) = actors(&store);
        let mut pins = Vec::new();
        for _ in 0..16 {
            let publication = store
                .execute(
                    &issuer,
                    "issuer-run",
                    &Operation::ContextPublish {
                        resource_id: None,
                        expected_revision: None,
                        title: "Input".into(),
                        source_revision: None,
                        content: ContextContent::Analysis {
                            summary: "Input snapshot".into(),
                            findings: vec![],
                        },
                        request_id: Uuid::new_v4().to_string(),
                    },
                )
                .unwrap();
            pins.push(serde_json::from_value(publication["reference"].clone()).unwrap());
        }
        let task = assign(&store, &issuer, &worker, pins.clone());
        start(&store, &worker, &task, 1);
        let result = store
            .execute(&worker, "worker-run", &submit(&task, 1))
            .unwrap();
        assert_eq!(store.task(&worker, &task).unwrap().context_refs, pins);
        assert!(result["handoff_ref"].is_object());
    }

    #[test]
    fn handoff_transaction_rolls_back_resource_state_events_notifications_and_receipt() {
        let store = Store::open(":memory:").unwrap();
        let (issuer, worker) = actors(&store);
        let task = assign(&store, &issuer, &worker, vec![]);
        start(&store, &worker, &task, 1);
        let operation = submit(&task, 1);
        for (table, condition) in [
            ("meta", "NEW.key LIKE 'context/version/%'"),
            ("meta", "NEW.key LIKE 'context/head/%'"),
            ("meta", "NEW.key LIKE 'handoff/task/%'"),
            ("events", "NEW.kind='handoff_created'"),
            ("messages", "NEW.kind='review'"),
            ("requests", "1"),
        ] {
            let before = store.history_sequence("/project").unwrap();
            store.connection.borrow_mut().batch_execute(&format!(
                "CREATE TEMP TRIGGER fail_handoff BEFORE INSERT ON {table} WHEN {condition} BEGIN SELECT RAISE(ABORT, 'fixture failure'); END;"
            )).unwrap();
            assert!(store.execute(&worker, "worker-run", &operation).is_err());
            assert_eq!(store.task(&worker, &task).unwrap().state, "running");
            assert_eq!(store.history_sequence("/project").unwrap(), before);
            assert!(store.task(&worker, &task).unwrap().handoff_ref.is_none());
            assert!(
                store.context_list(&worker, None, None).unwrap()["resources"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
            store
                .connection
                .borrow_mut()
                .batch_execute("DROP TRIGGER fail_handoff")
                .unwrap();
        }
        assert_eq!(
            store.execute(&worker, "worker-run", &operation).unwrap()["state"],
            "submitted"
        );
    }
}
