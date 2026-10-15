//! Trusted panel enrollment keeps the native process and physical checkout intact.
use super::*;

pub(super) fn active_agent(state: &State, agent: &str, run: &str) -> bool {
    state.terminals.values().any(|binding| {
        !binding.revoked
            && binding.physical_root.as_ref().is_none_or(|(root, handle)| {
                crate::companion::root_matches(handle, root).unwrap_or(false)
            })
            && binding.live.as_ref().is_some_and(|live| {
                !live.expired
                    && live.started.elapsed() < MUTATION_EPOCH
                    && live.run == run
                    && live.agent.as_ref().is_some_and(|actor| {
                        actor.id == agent && state.store.authorize(actor).is_ok()
                    })
            })
    })
}

impl Broker {
    /// Native launches bind repository identity before registration or task creation.
    pub fn activate_project(&self, terminal: &str, program: &str, root: &str, initial_prompt: bool) -> Result<()> {
        let checkout = crate::worktrees::Worktree::discover(Path::new(root)).ok();
        let root = checkout.as_ref().map_or(root, |checkout| checkout.root.as_str());
        if checkout.is_some() {
            let mut state = self.store()?;
            let binding = state.terminals.get(terminal)
                .ok_or_else(|| unauthorized("Terminal is not bound"))?;
            ensure!(!binding.revoked, scope_denied("Terminal participation was revoked"));
            if binding.workspace.is_none() && !state.store.worktree_opted_out(root)? {
                if state.store.worktree_binding(root)?.is_none() {
                    state.store.execute_controller(root, &ControllerOperation::WorktreeJoin {
                        root:root.into(), request_id:Uuid::new_v4().to_string(),
                    })?;
                }
                let workspace = state.store.worktree_binding(root)?;
                let binding = state.terminals.get_mut(terminal).unwrap();
                binding.workspace = workspace;
                binding.automatic_workspace = true;
            }
            if let Some(workspace) = &state.terminals[terminal].workspace {
                state.store.initialize_project_mode(&workspace.domain())?;
            }
        }
        self.activate(terminal, program, root, initial_prompt)
    }

    pub(crate) fn worktree_domain(&self, root: &str) -> Result<Option<String>> {
        self.store()?.store.worktree_domain(root)
    }
    pub(crate) fn worktree_opted_out(&self, root: &str) -> Result<bool> {
        self.store()?.store.worktree_opted_out(root)
    }
    pub(crate) fn worktree_role_receipt(
        &self,
        root: &str,
        operation: &ControllerOperation,
    ) -> Result<Option<Value>> {
        self.store()?.store.worktree_role_receipt(root, operation)
    }
    pub(super) fn forwarded(&self, terminal: &str) -> Option<Broker> {
        self.shared.forwarded.lock().ok()?.get(terminal).cloned()
    }

    pub(super) fn forwarded_wakes(&self, pending: bool) -> Vec<Wake> {
        let routes = self
            .shared
            .forwarded
            .lock()
            .map(|routes| routes.clone())
            .unwrap_or_default();
        routes
            .into_iter()
            .flat_map(|(terminal, broker)| {
                let wakes = if pending {
                    broker.pending_work()
                } else {
                    broker.wakeups()
                };
                wakes
                    .into_iter()
                    .filter(move |wake| wake.terminal == terminal)
            })
            .collect()
    }

    pub(crate) fn worktree_candidates(&self, root: &str) -> Result<Vec<Value>> {
        let repository = crate::worktrees::Worktree::discover(Path::new(root))?.repository;
        let state = self.store()?;
        let mut candidates: Vec<_> = state.terminals.values().filter(|binding| !binding.revoked)
            .filter_map(|binding| {
                let live = binding.live.as_ref()?;
                if live.expired || live.started.elapsed() >= MUTATION_EPOCH { return None; }
                let agent = live.agent.as_ref()?;
                if !active_agent(&state, &agent.id, &live.run) { return None; }
                state.store.authorize(agent).ok()?;
                let physical = state.store.physical_root(agent).ok()?;
                let checkout = crate::worktrees::Worktree::revalidate(Path::new(&physical)).ok()?;
                if checkout.repository != repository { return None; }
                Some(json!({"agent":agent,"run":live.run,"root":physical,"origin_agent":live.origin_agent.as_ref().map(|agent| &agent.id)}))
            }).collect();
        drop(state);
        for candidate in &mut candidates {
            candidate["branch"] = json!(crate::worktrees::branch(candidate["root"].as_str().unwrap()));
        }
        Ok(candidates)
    }

    /// Repository participation follows native runs, never the focused pane.
    pub(crate) fn enroll_worktree_agents(&self, source: &Broker, root: &str) -> Result<()> {
        let Some(domain) = self.worktree_domain(root)? else { return Ok(()); };
        let roles = {
            let state = self.store()?;
            let roles = state.store.worktree_roles(&domain)?;
            if !roles.iter().any(|role| role.role == "coordinator"
                && role.run.as_ref().is_some_and(|run| active_agent(&state, &role.agent, run))) {
                return Ok(());
            }
            roles
        };
        let needs_admission = {
            let state = source.store()?;
            state.terminals.values().filter(|binding| !binding.revoked).filter_map(|binding| binding.live.as_ref())
                .filter(|live| !live.expired).filter_map(|live| live.agent.as_ref().map(|agent| (agent, live)))
                .any(|(agent, live)| !roles.iter().any(|role| role.agent == agent.id && role.run.as_deref() == Some(live.run.as_str())))
        };
        if !needs_admission { return Ok(()); }
        for candidate in source.worktree_candidates(root)? {
            let agent = candidate["agent"]["id"].as_str().unwrap();
            let run = candidate["run"].as_str().unwrap();
            if roles.iter().any(|role| role.agent == agent && role.run.as_deref() == Some(run)) {
                continue;
            }
            if self.store()?.store.worktree_opted_out(candidate["root"].as_str().unwrap())? {
                continue;
            }
            let operation = ControllerOperation::WorktreeWorker {
                root: candidate["root"].as_str().unwrap().into(),
                agent: agent.into(), run: run.into(), request_id: Uuid::new_v4().to_string(),
            };
            // Existing private work retains its authority until it is resolved.
            if let Err(error) = self.enroll_worktree(source, &operation) {
                if crate::DomainError::from_error(error).code != "invalid_state" {
                    return Err(scope_denied("Could not admit an active worktree Agent"));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn enroll_worktree(
        &self,
        source: &Broker,
        operation: &ControllerOperation,
    ) -> Result<Value> {
        let (root, id, run, role) = match operation {
            ControllerOperation::WorktreeCoordinator {
                root, agent, run, ..
            } => (root, agent, run, "coordinator"),
            ControllerOperation::WorktreeWorker {
                root, agent, run, ..
            } => (root, agent, run, "worker"),
            _ => return Err(invalid_input("Not an enrollment intent")),
        };
        ensure!(
            operation
                .request_id()
                .is_some_and(|request| Uuid::parse_str(request).is_ok()),
            invalid_input("request_id must be a UUID")
        );
        let root = crate::project_root(Path::new(root))?;
        if let Some(receipt) = self.worktree_role_receipt(&root, operation)? {
            return Ok(receipt);
        }
        let same = Arc::ptr_eq(&self.shared, &source.shared);
        let mut source_state = source.store()?;
        let terminal = source_state
            .terminals
            .iter()
            .find_map(|(terminal, binding)| {
                let live = binding.live.as_ref()?;
                (!binding.revoked
                    && !live.expired
                    && live.started.elapsed() < MUTATION_EPOCH
                    && live.run == *run
                    && (live.agent.as_ref().is_some_and(|agent| agent.id == *id)
                        || live
                            .origin_agent
                            .as_ref()
                            .is_some_and(|agent| agent.id == *id)))
                .then(|| terminal.clone())
            })
            .ok_or_else(|| scope_denied("Selected Agent run is no longer active"))?;
        let live = source_state.terminals[&terminal].live.as_ref().unwrap();
        let original = live.agent.as_ref().unwrap().clone();
        source_state.store.authorize(&original)?;
        ensure!(
            source_state.store.physical_root(&original)? == root,
            scope_denied("Agent must already run in the selected checkout")
        );
        let physical = source_state.terminals[&terminal].physical_root.as_ref();
        ensure!(
            physical.is_some_and(
                |(path, handle)| crate::companion::root_matches(handle, path).unwrap_or(false)
            ),
            scope_denied("Selected physical checkout was replaced")
        );
        let enroll = |state: &State| -> Result<(crate::WorkspaceBinding, Agent, Value)> {
            let binding = match state.store.worktree_binding(&root)? {
                Some(binding) => binding,
                None => {
                    state.store.execute_controller(
                        &root,
                        &ControllerOperation::WorktreeJoin {
                            root: root.clone(),
                            request_id: Uuid::new_v4().to_string(),
                        },
                    )?;
                    state
                        .store
                        .worktree_binding(&root)?
                        .ok_or_else(|| invalid_state("Checkout admission unavailable"))?
                }
            };
            let domain = binding.domain();
            let roles = state.store.worktree_roles(&domain)?;
            if role == "worker" {
                ensure!(
                    roles.iter().any(|entry| entry.role == "coordinator"
                        && entry.run.as_ref().is_some_and(|run| active_agent(
                            state,
                            &entry.agent,
                            run
                        ))),
                    invalid_state("Select an active Coordinator first")
                );
            }
            let name = match state.store.agent(&domain, &original.name).ok() {
                Some(existing) if existing.terminal != terminal
                    && (state.store.physical_root(&existing)? != root
                        || state.terminals.values().any(|binding| binding.live.as_ref()
                            .is_some_and(|live| live.agent.as_ref().is_some_and(|agent| agent.id == existing.id)))) => {
                    format!("{}-{}", original.name.chars().take(55).collect::<String>(),
                        original.id.chars().take(8).collect::<String>())
                }
                _ => original.name.clone(),
            };
            let actor = if original.project == domain { original.clone() } else {
                state.store.register_in_workspace(&terminal, &original.program, &binding, &name)?
            };
            let result = state
                .store
                .worktree_role_intent(&root, operation, &actor, run, role)?;
            Ok((binding, actor, result))
        };
        let (mut target, binding, actor, result) = if same {
            if source_state.store.worktree_domain(&root)?.as_deref()
                != Some(original.project.as_str())
            {
                ensure!(
                    !source_state.store.unresolved_project_work(&original)?,
                    invalid_state("Resolve existing Project work before enrolling this Agent")
                );
            }
            let (binding, actor, result) = enroll(&source_state)?;
            (source_state, binding, actor, result)
        } else {
            ensure!(
                !source_state.store.unresolved_project_work(&original)?,
                invalid_state("Resolve existing Project work before enrolling this Agent")
            );
            let mut target = self.store()?;
            let (binding, actor, result) = enroll(&target)?;
            let native = source_state.terminals.remove(&terminal).unwrap();
            target.terminals.insert(terminal.clone(), native);
            source
                .shared
                .forwarded
                .lock()
                .map_err(|_| coordinator_unavailable("Broker unavailable"))?
                .insert(terminal.clone(), self.clone());
            drop(source_state);
            (target, binding, actor, result)
        };
        if role == "coordinator" {
            for binding in target.terminals.values_mut() {
                if let Some(live) = binding.live.as_mut().filter(|live| live.project == actor.project) {
                    live.generation += 1;
                    if let Some(wake) = live.wake.take() { live.delivered.remove(&wake.message_id); }
                }
            }
        }
        let native = target.terminals.get_mut(&terminal).unwrap();
        native.workspace = Some(binding);
        let live = native.live.as_mut().unwrap();
        if live.origin_agent.is_none() {
            live.origin_agent = Some(original);
        }
        live.project = actor.project.clone();
        live.agent = Some(actor.clone());
        live.generation += 1;
        live.delivered.clear();
        live.wake = None;
        live.waiting = false;
        let message = if role == "coordinator" {
            "You are the explicitly selected Worktree Coordinator. Assign tasks and context to participating Agents, collect results, designate a different worker for peer review, choose accepted results for integration with warp_task_integrate, and create the final integration commit in your checkout. Agents in any checkout may become Coordinator when the user changes the selection. Workers in different checkouts communicate through you."
        } else {
            "You are a Worktree participant. Work only in your own checkout, execute assigned tasks, report results and commit/test evidence, and review another worker when designated. Communicate directly with Agents in your checkout or with the Coordinator; other checkouts are isolated. The selected Coordinator owns task allocation and integration decisions."
        };
        if target.store.inbox_count(&actor.id)? < crate::storage::MAX_PENDING_PER_AGENT {
            target.store.execute(
                &Store::operator(&actor.project),
                crate::storage::OPERATOR_EPOCH,
                &Operation::AgentSend { content: None, expects_result: false,
                    to: actor.name.clone(),
                    body: message.into(),
                    subject: Some("Worktree role".into()),
                    thread_id: None,
                    reply_to: None,
                    task_id: None,
                    request_id: operation.request_id().unwrap().into(),
                },
            )?;
        }
        self.shared.changed.notify_all();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worktrees::tests::{assign, call, client, git, id, Fixture};

    #[test]
    fn every_agent_dispatches_idle_drafts_once_and_fences_cancelled_or_changed_runs() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let owner = RunningBroker::start(&root.join("draft-transitions.sqlite")).unwrap();
        let broker = &owner.broker;
        let issuer = client(broker, &root, "issuer");
        let programs = include_str!("../../../app/src/terminal/cli_agent.rs").lines()
            .filter_map(|line| line.trim().strip_prefix("CLIAgent::"))
            .filter(|line| line.contains("=> &[\""))
            .filter_map(|line| line.split('"').nth(1));
        for program in programs.chain(std::iter::once("custom")) {
            for draft in [b"unsent draft".as_slice(), "几个字 👩‍🔬".as_bytes(),
                b"\x1b[200~first\nsecond\x1b[201~", b"\x1b[A", b"/status\r"] {
                let terminal = id();
                let capability = broker.prepare(&terminal).unwrap();
                broker.activate_project(&terminal, program, root.to_str().unwrap(), false).unwrap();
                let mut worker = Request { protocol_major:PROTOCOL_MAJOR, terminal:terminal.clone(), capability,
                    run:None, directory:Some(root.to_str().unwrap().into()), defer_initial_ready:true, native_activity:None,
                    operation:Operation::AgentRegister { name:format!("worker-{}", id()) } };
                let actor = call(broker, &worker, worker.operation.clone()).unwrap();
                worker.run = actor["run"].as_str().map(str::to_owned);
                broker.input_bytes(&terminal, draft);
                let age = || {
                    let mut state = broker.store().unwrap();
                    let live = state.terminals.get_mut(&terminal).unwrap().live.as_mut().unwrap();
                    live.last_input = Instant::now() - Duration::from_secs(2);
                    live.last_output = live.last_input;
                    live.observed = live.last_input;
                    live.ready = Some(live.last_input);
                };
                age();
                assert!(broker.observe_screen(&terminal, &broker.screen_context(&terminal).unwrap(), Activity::Idle));
                age();
                let task = call(broker, &issuer, assign(actor["agent"]["name"].as_str().unwrap())).unwrap();
                let wake = broker.wakeups().into_iter().find(|wake| wake.terminal == terminal).unwrap();
                assert!(broker.claim_wake(&wake), "{program}: native draft is not execution");
                assert!(!broker.claim_wake(&wake), "one claim per message");
                assert!(broker.wake_valid(&wake));
                let expected_discard = draft != b"/status\r";
                assert_eq!(broker.discard_native_draft(&wake), expected_discard);
                assert!(broker.wake_valid(&wake), "owner-authorized clearing preserves the claim");
                broker.input_bytes(&terminal, b"changed during delayed Enter");
                assert!(!broker.wake_valid(&wake), "new input fences delayed submission");
                broker.input_bytes(&terminal, b"\x03");
                age();
                broker.observe_screen(&terminal, &broker.screen_context(&terminal).unwrap(), Activity::Idle);
                assert!(broker.wakeups().iter().all(|wake| wake.terminal != terminal), "cancel does not auto-retry");
                broker.input_bytes(&terminal, b"resumed\r");
                age();
                broker.observe_screen(&terminal, &broker.screen_context(&terminal).unwrap(), Activity::Idle);
                age();
                let retry = broker.wakeups().into_iter().find(|wake| wake.terminal == terminal).unwrap();
                assert_eq!(retry.message_id, wake.message_id, "retry retains the original task message");
                assert!(broker.claim_wake(&retry));
                broker.finish_wake(&retry, true);
                let inbox = call(broker, &worker, Operation::AgentInbox { cursor:None, limit:None }).unwrap();
                assert!(inbox["messages"].as_array().unwrap().iter().any(|message| message["id"] == retry.message_id
                    && message["task_id"] == task["id"]), "PTY submit cannot acknowledge MCP work");
                broker.end(&terminal);
                assert!(!broker.wake_valid(&retry));
            }
        }
    }

    #[test]
    fn unchanged_guards_and_read_only_mcp_calls_do_not_invalidate_empty_queue_cache() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let owner = RunningBroker::start(&root.join("revision.sqlite")).unwrap();
        let broker = &owner.broker;
        let worker = client(broker, &root, "worker");
        broker.composer_guard(&worker.terminal, false, false);
        let revision = broker.work_revision().unwrap();
        for _ in 0..3 {
            broker.composer_guard(&worker.terminal, false, false);
            call(broker, &worker, Operation::AgentInbox { cursor:None, limit:None }).unwrap();
            call(broker, &worker, Operation::AgentList).unwrap();
        }
        assert_eq!(broker.work_revision(), Some(revision));
        broker.composer_guard(&worker.terminal, false, true);
        assert_ne!(broker.work_revision(), Some(revision));
        let revision = broker.work_revision();
        let issuer = client(broker, &root, "issuer");
        call(broker, &issuer, assign("worker")).unwrap();
        assert_ne!(broker.work_revision(), revision);
        assert!(broker.pending_work().iter().any(|wake| wake.terminal == worker.terminal));
    }

    #[test]
    fn lazy_deadline_notifications_invalidate_otherwise_read_only_queue_cache() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let owner = RunningBroker::start(&root.join("deadline-revision.sqlite")).unwrap();
        let broker = &owner.broker;
        let issuer = client(broker, &root, "issuer");
        let worker = client(broker, &root, "worker");
        let mut operation = assign("worker");
        if let Operation::TaskAssign { execution_timeout_seconds, .. } = &mut operation {
            *execution_timeout_seconds = Some(1);
        }
        let task = call(broker, &issuer, operation).unwrap();
        call(broker, &worker, Operation::TaskStart { task_id:task["id"].as_str().unwrap().into(),
            revision:1, expected_version:None, request_id:id() }).unwrap();
        assert!(broker.pending_work().is_empty());
        let revision = broker.work_revision();
        std::thread::sleep(Duration::from_millis(1200));
        call(broker, &issuer, Operation::TaskGet { task_id:task["id"].as_str().unwrap().into() }).unwrap();
        assert_ne!(broker.work_revision(), revision, "a read can emit a lazy deadline notice");
        assert!(broker.pending_work().iter().any(|wake| wake.terminal == worker.terminal));
    }

    #[test]
    fn full_lifecycle_hooks_keep_authority_over_screen_fallbacks() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let owner = RunningBroker::start(&root.join("hook-authority.sqlite")).unwrap();
        let broker = &owner.broker;
        for program in ["pi", "omp", "opencode"] {
            let terminal = id();
            broker.prepare(&terminal).unwrap();
            broker.activate_project(&terminal, program, root.to_str().unwrap(), false).unwrap();
            for authoritative in [Activity::Working, Activity::WaitingApproval, Activity::Idle] {
                broker.activity(&terminal, authoritative);
                let context = broker.screen_context(&terminal).unwrap();
                assert!(broker.observe_screen(&terminal, &context, authoritative));
                let guess = if authoritative.is_idle() { Activity::Working } else { Activity::Idle };
                assert!(!broker.observe_screen(&terminal, &context, guess));
                assert_eq!(broker.terminal_activity(&terminal), Some(authoritative));
            }
        }
    }

    #[test]
    fn blocked_native_sessions_keep_mcp_connected_and_resume_original_queued_work() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let owner = RunningBroker::start(&root.join("blocked.sqlite")).unwrap();
        let broker = &owner.broker;
        let issuer = client(broker, &root, "issuer");
        let worker = client(broker, &root, "worker");
        for blocker in [Activity::WaitingApproval, Activity::WaitingInput] {
            broker.activity(&worker.terminal, Activity::Working);
            broker.observe_screen(&worker.terminal, &broker.screen_context(&worker.terminal).unwrap(), blocker);
            let task = call(broker, &issuer, assign("worker")).unwrap();
            let inbox = call(broker, &worker, Operation::AgentInbox { cursor:None, limit:None }).unwrap();
            let message = inbox["messages"].as_array().unwrap().iter()
                .find(|message| message["task_id"] == task["id"]).unwrap();
            let original = message["id"].as_str().unwrap().to_owned();
            assert!(broker.pending_work().iter().any(|wake| wake.message_id == original));
            assert!(broker.wakeups().iter().all(|wake| wake.terminal != worker.terminal));
            assert_eq!(broker.terminal_activity(&worker.terminal), Some(blocker));
            {
                let mut state = broker.store().unwrap();
                let live = state.terminals.get_mut(&worker.terminal).unwrap().live.as_mut().unwrap();
                live.last_input = Instant::now() - Duration::from_secs(2);
                live.last_output = live.last_input;
                live.observed = live.last_input;
            }
            assert!(broker.observe_screen(&worker.terminal, &broker.screen_context(&worker.terminal).unwrap(), Activity::Idle));
            {
                let mut state = broker.store().unwrap();
                state.terminals.get_mut(&worker.terminal).unwrap().live.as_mut().unwrap().ready =
                    Some(Instant::now() - Duration::from_secs(2));
            }
            let wake = broker.wakeups().into_iter().find(|wake| wake.message_id == original).unwrap();
            assert!(broker.claim_wake(&wake));
            broker.finish_wake(&wake, true);
            let detail = call(broker, &issuer, Operation::TaskGet { task_id:task["id"].as_str().unwrap().into() }).unwrap();
            assert_ne!(detail["state"], "completed");
            let inbox = call(broker, &worker, Operation::AgentInbox { cursor:None, limit:None }).unwrap();
            assert!(inbox["messages"].as_array().unwrap().iter().any(|message| message["id"] == original),
                "native delivery must not acknowledge an MCP message");
        }
    }

    #[test]
    fn screen_permission_observation_notifies_task_issuer_once() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let owner = RunningBroker::start(&root.join("attention.sqlite")).unwrap();
        let broker = &owner.broker;
        let issuer = client(broker, &root, "issuer");
        let worker = client(broker, &root, "worker");
        let task = call(broker, &issuer, assign("worker")).unwrap();
        call(broker, &worker, Operation::TaskStart { task_id:task["id"].as_str().unwrap().into(),
            revision:1, expected_version:None, request_id:id() }).unwrap();
        for _ in 0..3 {
            assert!(broker.observe_screen(&worker.terminal, &broker.screen_context(&worker.terminal).unwrap(), Activity::WaitingApproval));
        }
        let inbox = call(broker, &issuer, Operation::AgentInbox { cursor:None, limit:None }).unwrap();
        let observations: Vec<_> = inbox["messages"].as_array().unwrap().iter()
            .filter(|message| message["kind"] == "observation").collect();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0]["task_id"], task["id"]);
        assert!(broker.wakeups().iter().all(|wake| wake.terminal != worker.terminal));
        {
            let mut state = broker.store().unwrap();
            let live = state.terminals.get_mut(&worker.terminal).unwrap().live.as_mut().unwrap();
            live.last_input = Instant::now() - Duration::from_secs(2);
            live.last_output = live.last_input;
            live.observed = live.last_input;
        }
        for _ in 0..3 {
            assert!(broker.observe_screen(&worker.terminal, &broker.screen_context(&worker.terminal).unwrap(), Activity::Idle));
        }
        let detail = call(broker, &issuer, Operation::TaskGet { task_id:task["id"].as_str().unwrap().into() }).unwrap();
        assert_eq!(detail["state"], "running", "returning to the prompt cannot submit the task");
        let inbox = call(broker, &issuer, Operation::AgentInbox { cursor:None, limit:None }).unwrap();
        assert_eq!(inbox["messages"].as_array().unwrap().iter().filter(|message| message["kind"] == "observation").count(), 2,
            "idle with an unfinished task adds one durable MCP supervision notice");
    }

    #[test]
    fn native_screen_fences_inputs_redraws_restarts_and_preserves_idle_leases() {
        let directory = tempfile::tempdir().unwrap();
        let root = crate::project_root(directory.path()).unwrap();
        let owner = RunningBroker::start(&directory.path().join("screen.sqlite")).unwrap();
        let broker = &owner.broker;
        let sender = client(broker, std::path::Path::new(&root), "observer");
        let programs = include_str!("../../../app/src/terminal/cli_agent.rs").lines()
            .filter_map(|line| line.trim().strip_prefix("CLIAgent::"))
            .filter(|line| line.contains("=> &[\""))
            .filter_map(|line| line.split('"').nth(1));
        for program in programs.chain(std::iter::once("custom")) {
            let terminal = id();
            let capability = broker.prepare(&terminal).unwrap();
            broker.activate_project(&terminal, program, &root, false).unwrap();
            let mut request = Request { protocol_major:PROTOCOL_MAJOR, terminal:terminal.clone(), capability,
                run:None, directory:Some(root.clone()), defer_initial_ready:true, native_activity:None,
                operation:Operation::AgentRegister { name:format!("peer-{}", id()) } };
            let actor = call(broker, &request, request.operation.clone()).unwrap();
            request.run = actor["run"].as_str().map(str::to_owned);
            let context = broker.screen_context(&terminal).unwrap();
            assert!(context.same_screen(&broker.screen_context(&terminal).unwrap()));
            broker.input_bytes(&terminal, b"draft");
            assert!(!context.same_screen(&broker.screen_context(&terminal).unwrap()));
            assert!(!broker.observe_screen(&terminal, &context, Activity::Idle));
            let context = broker.screen_context(&terminal).unwrap();
            broker.output(&terminal);
            assert!(!broker.observe_screen(&terminal, &context, Activity::Working));
            {
                let mut state = broker.store().unwrap();
                let live = state.terminals.get_mut(&terminal).unwrap().live.as_mut().unwrap();
                live.last_input = Instant::now() - Duration::from_secs(2);
                live.last_output = live.last_input;
                live.observed = live.last_input;
            }
            let context = broker.screen_context(&terminal).unwrap();
            assert!(broker.observe_screen(&terminal, &context, Activity::Idle));
            let (generation, ready) = {
                let state = broker.store().unwrap();
                let live = state.terminals[&terminal].live.as_ref().unwrap();
                assert_eq!(live.draft.state(), "present", "screen evidence cannot delete user text");
                (live.generation, live.ready)
            };
            for _ in 0..4 {
                assert!(broker.observe_screen(&terminal, &broker.screen_context(&terminal).unwrap(), Activity::Idle));
            }
            {
                let state = broker.store().unwrap();
                let live = state.terminals[&terminal].live.as_ref().unwrap();
                assert_eq!((live.generation, live.ready), (generation, ready));
            }
            let peers = call(broker, &sender, Operation::AgentList).unwrap();
            let peer = peers.as_array().unwrap().iter().find(|peer| peer["terminal"] == terminal).unwrap();
            assert_eq!(peer["activity"], "idle");
            assert_eq!(peer["readiness_source"], "native_screen");
            call(broker, &sender, Operation::AgentSend { content: None, expects_result:false,
                to:actor["agent"]["name"].as_str().unwrap().into(), body:"Check".into(), subject:None,
                thread_id:None, reply_to:None, task_id:None, request_id:id() }).unwrap();
            {
                let mut state = broker.store().unwrap();
                state.terminals.get_mut(&terminal).unwrap().live.as_mut().unwrap().ready = Some(Instant::now() - Duration::from_secs(2));
            }
            let wake = broker.wakeups().into_iter().find(|wake| wake.terminal == terminal).unwrap();
            assert!(broker.claim_wake(&wake));
            assert!(broker.observe_screen(&terminal, &broker.screen_context(&terminal).unwrap(), Activity::Idle));
            assert!(broker.wake_valid(&wake), "unchanged screen cannot invalidate a claimed wake");
            let context = broker.screen_context(&terminal).unwrap();
            broker.end(&terminal);
            broker.activate_project(&terminal, program, &root, false).unwrap();
            assert!(!broker.observe_screen(&terminal, &context, Activity::Idle), "old run cannot complete its successor");
        }
    }

    #[test]
    fn replaced_repository_at_old_path_registers_fresh_agents_without_old_authority() {
        for opted_out in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let main = directory.path().join("main");
            std::fs::create_dir(&main).unwrap();
            git(&main, &["init", "--initial-branch=main"]);
            let root = crate::project_root(&main).unwrap();
            let database = directory.path().join("persisted.sqlite");
            let owner = RunningBroker::start(&database).unwrap();
            let broker = &owner.broker;
            let register = |broker: &Broker, program: &str| {
                let terminal = id();
                let capability = broker.prepare(&terminal).unwrap();
                broker.activate_project(&terminal, program, &root, true).unwrap();
                let mut request = Request { protocol_major:PROTOCOL_MAJOR, terminal, capability,
                    run:None, directory:Some(root.clone()), defer_initial_ready:false, native_activity:None,
                    operation:Operation::AgentRegister { name:program.into() } };
                let actor = call(broker, &request, request.operation.clone()).unwrap();
                request.run = actor["run"].as_str().map(str::to_owned);
                (request, actor)
            };
            let (old, actor) = register(broker, "codex");
            broker.control(&root, &ControllerOperation::WorktreeCoordinator { root:root.clone(),
                agent:actor["agent"]["id"].as_str().unwrap().into(), run:old.run.clone().unwrap(), request_id:id() }).unwrap();
            if opted_out {
                broker.control(&root, &ControllerOperation::WorktreeLeave { root:root.clone(), request_id:id() }).unwrap();
            }
            std::fs::rename(&main, directory.path().join("old main")).unwrap();
            std::fs::create_dir(&main).unwrap();
            git(&main, &["init", "--initial-branch=main"]);
            assert!(call(broker, &old, Operation::AgentList).is_err());
            let empty = broker.operator_panel(&PanelQuery { project:root.clone(), ..Default::default() }).unwrap();
            assert_eq!(empty["worktree_available"], true);
            assert!(empty["agents"].as_array().unwrap().is_empty());
            broker.control(&root, &ControllerOperation::WorktreeMode { worktree:true, request_id:id() }).unwrap();
            assert_eq!(broker.operator_panel(&PanelQuery { project:root.clone(), ..Default::default() }).unwrap()["worktree_mode"], true);
            let old_pane = broker.operator_panel(&PanelQuery { project:root.clone(), terminal:Some(old.terminal.clone()), ..Default::default() }).unwrap();
            assert_eq!(old_pane["worktree_mode"], true, "a stale pane must not hide the fresh main-only repository mode");
            assert!(old_pane["agents"].as_array().unwrap().is_empty());
            let (fresh, current) = register(broker, "codex");
            register(broker, "agy");
            assert_ne!(actor["agent"]["project"], current["agent"]["project"]);
            assert_ne!(actor["agent"]["id"], current["agent"]["id"]);
            assert!(call(broker, &old, Operation::AgentList).is_err());
            assert_eq!(call(broker, &fresh, Operation::AgentList).unwrap().as_array().unwrap().len(), 1);
            for worktree in [false, true] {
                let panel = broker.operator_panel(&PanelQuery { project:root.clone(), worktree, ..Default::default() }).unwrap();
                assert_eq!(panel["agents"].as_array().unwrap().len(), 2);
                assert_eq!(panel["coordinator_online"], false);
                assert!(panel["agents"].as_array().unwrap().iter().all(|row| row["activity"] == "idle"));
            }
            drop(owner);
            let reopened = RunningBroker::start(&database).unwrap();
            register(&reopened.broker, "agy");
            assert_eq!(reopened.broker.operator_panel(&PanelQuery { project:root, ..Default::default() }).unwrap()["agents"].as_array().unwrap().len(), 1);
        }
    }

    #[test]
    fn main_only_checkout_registers_each_program_and_switches_modes() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("main only");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "--initial-branch=main"]);
        let root = crate::project_root(&root).unwrap();
        let owner = RunningBroker::start(&directory.path().join("main-only.sqlite")).unwrap();
        let broker = &owner.broker;
        assert_eq!(crate::worktrees::list(&root).unwrap().len(), 1);
        broker.control(&root, &ControllerOperation::WorktreeMode { worktree:true, request_id:id() }).unwrap();
        assert_eq!(broker.operator_panel(&PanelQuery { project:root.clone(), worktree:true, ..Default::default() }).unwrap()["worktree_mode"], true);
        broker.control(&root, &ControllerOperation::WorktreeMode { worktree:false, request_id:id() }).unwrap();
        let mut lead = None;
        let mut project = None;
        for program in ["codex", "claude", "gemini", "qoder", "cursor-agent", "custom"] {
            let terminal = id();
            let capability = broker.prepare(&terminal).unwrap();
            broker.activate_project(&terminal, program, &root, true).unwrap();
            let mut request = Request { protocol_major:PROTOCOL_MAJOR, terminal, capability,
                run:None, directory:Some(root.clone()), defer_initial_ready:false, native_activity:None,
                operation:Operation::AgentRegister { name:program.into() } };
            let actor = call(broker, &request, request.operation.clone()).unwrap();
            assert_eq!(actor["agent"]["program"], program);
            if let Some(project) = &project { assert_eq!(&actor["agent"]["project"], project); }
            else { project = Some(actor["agent"]["project"].clone()); }
            request.run = actor["run"].as_str().map(str::to_owned);
            if lead.is_none() { lead = Some((request, actor["agent"].clone())); }
        }
        let panel = broker.operator_panel(&PanelQuery { project:root.clone(), ..Default::default() }).unwrap();
        assert_eq!(panel["agents"].as_array().unwrap().len(), 6);
        let (lead, actor) = lead.unwrap();
        broker.control(&root, &ControllerOperation::WorktreeCoordinator { root:root.clone(),
            agent:actor["id"].as_str().unwrap().into(), run:lead.run.unwrap(), request_id:id() }).unwrap();
        let panel = broker.operator_panel(&PanelQuery { project:root, worktree:true, ..Default::default() }).unwrap();
        assert_eq!(panel["coordinator_online"], true);
        assert_eq!(panel["candidates"].as_array().unwrap().len(), 6);
    }

    #[test]
    fn automatic_workspace_follows_new_run_after_changing_checkout() {
        let fixture = Fixture::new();
        let owner = RunningBroker::start(&fixture.directory.path().join("change-checkout.sqlite")).unwrap();
        let broker = &owner.broker;
        let terminal = id();
        let capability = broker.prepare(&terminal).unwrap();
        let mut identities = std::collections::HashMap::new();
        for root in [&fixture.linked, &fixture.main, &fixture.linked] {
            broker.activate_project(&terminal, "codex", root.to_str().unwrap(), true).unwrap();
            let request = Request { protocol_major:PROTOCOL_MAJOR, terminal:terminal.clone(),
                capability:capability.clone(), run:None, directory:Some(root.to_str().unwrap().into()),
                defer_initial_ready:false, native_activity:None,
                operation:Operation::AgentRegister { name:String::new() } };
            let actor = call(broker, &request, request.operation.clone()).unwrap();
            let panel = broker.operator_panel(&PanelQuery { project:root.to_str().unwrap().into(),
                terminal:Some(terminal.clone()), ..Default::default() }).unwrap();
            let row = panel["agents"].as_array().unwrap().iter().find(|row| row["agent"]["id"] == actor["agent"]["id"]).unwrap();
            assert_eq!(row["workspace"], root.to_str().unwrap());
            if let Some(previous) = identities.insert(root.to_path_buf(), actor["agent"]["id"].clone()) {
                assert_eq!(previous, actor["agent"]["id"], "Returning to a checkout preserves its own identity");
            }
            broker.end(&terminal);
        }
        assert_ne!(identities[&fixture.main], identities[&fixture.linked]);
    }

    #[test]
    fn native_project_identity_mode_and_busy_coordinator_handoff_preserve_tasks() {
        let fixture = Fixture::new();
        let owner = RunningBroker::start(&fixture.directory.path().join("initial-project.sqlite")).unwrap();
        let broker = &owner.broker;
        let register = |root: &Path, name: &str| {
            let terminal = id();
            let capability = broker.prepare(&terminal).unwrap();
            broker.activate_project(&terminal, "fixture", root.to_str().unwrap(), true).unwrap();
            let mut request = Request { protocol_major:PROTOCOL_MAJOR, terminal, capability, run:None,
                directory:Some(root.to_str().unwrap().into()), defer_initial_ready:false, native_activity:None,
                operation:Operation::AgentRegister { name:name.into() } };
            let actor = call(broker, &request, request.operation.clone()).unwrap();
            request.run = actor["run"].as_str().map(str::to_owned);
            (request, actor["agent"].clone())
        };
        let (lead, lead_agent) = register(&fixture.main, "lead");
        let (worker, worker_agent) = register(&fixture.linked, "worker");
        let (other, other_agent) = register(&fixture.unjoined, "other");
        assert_eq!(lead_agent["project"], worker_agent["project"]);
        let task = call(broker, &lead, assign("worker")).unwrap();
        call(broker, &worker, Operation::TaskStart { task_id:task["id"].as_str().unwrap().into(),
            revision:1, expected_version:None, request_id:id() }).unwrap();
        for _ in 0..17 {
            let mut operation = assign("worker");
            if let Operation::TaskAssign { description, .. } = &mut operation {
                *description = "x".repeat(512);
            }
            call(broker, &lead, operation).unwrap();
        }
        let select = |request: &Request, actor: &Value, root: &Path| {
            broker.control(root.to_str().unwrap(), &ControllerOperation::WorktreeCoordinator {
                root:root.to_str().unwrap().into(), agent:actor["id"].as_str().unwrap().into(),
                run:request.run.clone().unwrap(), request_id:id(),
            }).unwrap();
        };
        select(&lead, &lead_agent, &fixture.main);
        select(&worker, &worker_agent, &fixture.linked);
        let empty = fixture.directory.path().join("empty checkout");
        git(&fixture.main, &["worktree", "add", "-b", "empty", empty.to_str().unwrap()]);
        let empty_root = crate::project_root(&empty).unwrap();
        let empty_panel = || broker.operator_panel(&PanelQuery { project:empty_root.clone(), ..Default::default() }).unwrap();
        assert_eq!(empty_panel()["project"], lead_agent["project"]);
        assert_eq!(empty_panel()["worktree_mode"], true);
        assert_eq!(empty_panel()["coordinator_online"], true);
        assert!(broker.store().unwrap().store.worktree_binding(&empty_root).unwrap().is_none());
        let plain = fixture.directory.path().join("plain project");
        std::fs::create_dir(&plain).unwrap();
        let (plain_request, plain_agent) = register(&plain, "plain");
        let plain_root = crate::project_root(&plain).unwrap();
        assert_eq!(plain_agent["project"], plain_root);
        let plain_panel = broker.operator_panel(&PanelQuery { project:plain_root.clone(),
            terminal:Some(plain_request.terminal.clone()), ..Default::default() }).unwrap();
        assert_eq!(plain_panel["project"], plain_root);
        assert_eq!(plain_panel["worktree_mode"], false);
        assert_eq!(plain_panel["worktree_available"], false);
        assert_eq!(plain_panel["coordinator_online"], false);
        assert_eq!(plain_panel["agents"].as_array().unwrap().len(), 1);
        assert!(call(broker, &plain_request, Operation::AgentSend { content: None, expects_result: false, to:"lead".into(),
            body:"Separate project".into(), subject:None, thread_id:None, reply_to:None,
            task_id:None, request_id:id() }).is_err());
        let unavailable = broker.operator_panel(&PanelQuery { project:plain_root,
            worktree:true, ..Default::default() }).unwrap();
        assert_eq!(unavailable["worktree_mode"], false);
        assert_eq!(unavailable["worktree_available"], false);
        assert_eq!(empty_panel()["coordinator_online"], true);
        assert_eq!(call(broker, &worker, Operation::AgentRegister { name:String::new() }).unwrap()["agent"], worker_agent);
        let inbox = call(broker, &worker, Operation::AgentInbox { cursor:None, limit:None }).unwrap();
        assert!(inbox["messages"].as_array().unwrap().iter().any(|message|
            message["subject"] == "Coordinator handoff"
                && message["body"].as_str().unwrap().contains("18 unfinished project tasks")
                && message["body"].as_str().unwrap().len() <= crate::MAX_TEXT));
        let mode = |worktree| broker.control(fixture.main.to_str().unwrap(), &ControllerOperation::WorktreeMode { worktree, request_id:id() }).unwrap();
        let send = || call(broker, &lead, Operation::AgentSend { content: None, expects_result: false, to:"other".into(), body:"Check".into(),
            subject:None, thread_id:None, reply_to:None, task_id:None, request_id:id() });
        assert!(send().is_err());
        mode(false);
        assert_eq!(empty_panel()["project"], lead_agent["project"]);
        assert_eq!(empty_panel()["worktree_mode"], false);
        assert_eq!(empty_panel()["coordinator_online"], true);
        assert!(send().is_ok());
        assert!(call(broker, &other, assign("lead")).is_ok());
        mode(true);
        assert!(send().is_err());
        let panel = broker.operator_panel(&PanelQuery { project:fixture.main.to_str().unwrap().into(), ..Default::default() }).unwrap();
        assert_eq!(panel["agents"].as_array().unwrap().len(), 3);
        assert_eq!(panel["coordinator_online"], true);
        assert_eq!(call(broker, &worker, Operation::TaskGet { task_id:task["id"].as_str().unwrap().into() }).unwrap()["assignee"], worker_agent["id"]);
        select(&other, &other_agent, &fixture.unjoined);
        call(broker, &worker, Operation::TaskSubmit { task_id:task["id"].as_str().unwrap().into(),
            revision:1, result:"Ready".into(), evidence:"Checked".into(), evidence_ids:vec![],
            attempt_id:None, expected_version:None, request_id:id() }).unwrap();
        let updates = call(broker, &other, Operation::AgentInbox { cursor:None, limit:None }).unwrap();
        assert!(updates["messages"].as_array().unwrap().iter().any(|message|
            message["subject"] == "Task update" && message["body"]["task_id"] == task["id"]));
        select(&worker, &worker_agent, &fixture.linked);
        broker.end(&worker.terminal);
        let (replacement, replacement_agent) = register(&fixture.linked, "worker");
        assert_eq!(replacement_agent["id"], worker_agent["id"]);
        let panel = broker.operator_panel(&PanelQuery { project:fixture.main.to_str().unwrap().into(), worktree:true, ..Default::default() }).unwrap();
        assert_eq!(panel["coordinator_online"], false);
        assert!(call(broker, &replacement, assign("lead")).is_err());
    }

    #[test]
    fn native_project_participation_isolates_checkouts_and_preserves_coordinator_selection() {
        let fixture = Fixture::new();
        let owner = RunningBroker::start(&fixture.directory.path().join("native-project.sqlite")).unwrap();
        let broker = &owner.broker;
        let main = fixture.main.to_str().unwrap();
        let lead = client(broker, &fixture.main, "lead");
        let left = client(broker, &fixture.linked, "left");
        let right = client(broker, &fixture.linked, "right");
        let other = client(broker, &fixture.unjoined, "other");
        let select = |request: &Request, root: &Path| {
            let actor = call(broker, request, Operation::AgentRegister { name: "".into() }).unwrap();
            broker.control(main, &ControllerOperation::WorktreeCoordinator {
                root: root.to_str().unwrap().into(), agent: actor["agent"]["id"].as_str().unwrap().into(),
                run: request.run.clone().unwrap(), request_id: id(),
            }).unwrap()
        };
        let send = |request: &Request, to: &str| call(broker, request, Operation::AgentSend { content: None, expects_result: false,
            to: to.into(), body: "Scope check".into(), subject: None, thread_id: None,
            reply_to: None, task_id: None, request_id: id(),
        });
        let original = select(&lead, &fixture.main);
        assert_eq!(broker.run(&lead.terminal), lead.run);
        assert!(send(&left, "right").is_ok());
        assert!(send(&left, "lead").is_ok());
        assert!(send(&lead, "other").is_ok());
        assert!(send(&left, "other").is_err());
        let visible = call(broker, &left, Operation::AgentList).unwrap();
        assert!(visible.as_array().unwrap().iter().any(|agent| agent["name"] == "right"));
        assert!(!visible.as_array().unwrap().iter().any(|agent| agent["name"] == "other"));
        assert!(!broker.peers(&left.terminal).iter().any(|peer| peer.name == "other"));
        let later = client(broker, &fixture.linked, "later");
        assert!(send(&later, "right").is_ok());
        assert!(send(&later, "other").is_err());
        let changed = select(&left, &fixture.linked);
        assert_eq!(broker.run(&left.terminal), left.run);
        assert!(send(&lead, "right").is_err());
        assert!(send(&left, "other").is_ok());
        assert!(send(&lead, "left").is_ok());
        let inbox = call(broker, &lead, Operation::AgentInbox { cursor:None, limit:None }).unwrap();
        assert!(inbox["messages"].as_array().unwrap().iter().any(|message|
            message["subject"] == "Coordinator changed" && message["body"].as_str().unwrap().contains("left")));
        // Changing pane, checkout, subdirectory or project only changes the read projection.
        std::fs::create_dir(fixture.linked.join("nested")).unwrap();
        let nested = crate::project_root(&fixture.linked.join("nested")).unwrap();
        for (root, terminal) in [(main.to_owned(), Some(lead.terminal.clone())),
            (nested, Some(right.terminal.clone())),
            (fixture.unjoined.to_str().unwrap().into(), None)] {
            let panel = broker.operator_panel(&PanelQuery { project:root, terminal,
                worktree:true, ..Default::default() }).unwrap();
            assert_eq!(panel["coordinator_online"], true);
            let role = panel["roles"].as_array().unwrap().iter().find(|role| role["role"] == "coordinator").unwrap();
            assert_eq!(role["agent"], changed["agent"]["id"]);
        }
        let unrelated = broker.operator_panel(&PanelQuery { project: fixture.directory.path().to_str().unwrap().into(),
            worktree:true, ..Default::default() }).unwrap();
        assert_eq!(unrelated["worktree_mode"], false);
        assert_eq!(broker.operator_panel(&PanelQuery { project:main.into(), worktree:true,
            ..Default::default() }).unwrap()["coordinator_online"], true);
        broker.end(&left.terminal);
        assert_eq!(broker.operator_panel(&PanelQuery { project:main.into(), worktree:true,
            ..Default::default() }).unwrap()["coordinator_online"], false);
        // A prior selection receipt cannot elect the original Coordinator again.
        assert_ne!(original["agent"]["id"], changed["agent"]["id"]);
        assert!(send(&right, "later").is_ok());
        drop(owner);
        let reopened = RunningBroker::start(&fixture.directory.path().join("native-project.sqlite")).unwrap();
        assert_eq!(reopened.broker.operator_panel(&PanelQuery { project:main.into(), worktree:true,
            ..Default::default() }).unwrap()["coordinator_online"], false);
    }

    #[test]
    fn automatic_participation_disambiguates_names_without_reclaiming_other_checkouts() {
        let fixture = Fixture::new();
        let owner = RunningBroker::start(&fixture.directory.path().join("names.sqlite")).unwrap();
        let broker = &owner.broker;
        let lead = client(broker, &fixture.main, "lead");
        let first = client(broker, &fixture.linked, "worker");
        let second = client(broker, &fixture.unjoined, "worker");
        let lead_id = call(broker, &lead, Operation::AgentRegister { name:"".into() }).unwrap()["agent"]["id"].as_str().unwrap().to_owned();
        broker.control(fixture.main.to_str().unwrap(), &ControllerOperation::WorktreeCoordinator {
            root:fixture.main.to_str().unwrap().into(), agent:lead_id, run:lead.run.clone().unwrap(), request_id:id(),
        }).unwrap();
        let first_agent = call(broker, &first, Operation::AgentRegister { name:"".into() }).unwrap();
        let second_agent = call(broker, &second, Operation::AgentRegister { name:"".into() }).unwrap();
        assert_ne!(first_agent["agent"]["name"], second_agent["agent"]["name"]);
        assert_eq!(first_agent["agent"]["project"], second_agent["agent"]["project"]);
        assert_eq!(broker.run(&first.terminal), first.run);
        assert_eq!(broker.run(&second.terminal), second.run);
        let later = client(broker, &fixture.main, "worker");
        let later_agent = call(broker, &later, Operation::AgentRegister { name:"".into() }).unwrap();
        assert_ne!(first_agent["agent"]["name"], later_agent["agent"]["name"]);
        assert_ne!(second_agent["agent"]["name"], later_agent["agent"]["name"]);
        assert_eq!(first_agent["agent"]["project"], later_agent["agent"]["project"]);
        broker.expire_epoch(&lead.terminal);
        let panel = broker.operator_panel(&PanelQuery { project:fixture.main.to_str().unwrap().into(),
            worktree:true, ..Default::default() }).unwrap();
        assert_eq!(panel["coordinator_online"], false);
        assert!(panel["roles"].as_array().unwrap().iter()
            .any(|role| role["role"] == "coordinator" && !role["run"].is_null()));
    }

    #[test]
    fn online_project_agents_are_filtered_before_pagination() {
        let directory = tempfile::tempdir().unwrap();
        let canonical = directory.path().canonicalize().unwrap();
        let root = canonical.to_str().unwrap();
        let owner = RunningBroker::start(&directory.path().join("online.sqlite")).unwrap();
        let broker = &owner.broker;
        for index in 0..55 {
            let request = client(broker, directory.path(), &format!("a-offline-{index:02}"));
            broker.end(&request.terminal);
        }
        let online = client(broker, directory.path(), "z-online");
        let panel = broker.operator_panel(&PanelQuery { project:root.into(), ..Default::default() }).unwrap();
        assert_eq!(panel["agents"].as_array().unwrap().len(), 1);
        assert_eq!(panel["agents"][0]["agent"]["name"], "z-online");
        assert!(panel["agent_cursor"].is_null());
        assert_eq!(panel["participant_names"].as_object().unwrap().len(), 56);
        broker.end(&online.terminal);
        assert!(broker.operator_panel(&PanelQuery { project:root.into(), ..Default::default() })
            .unwrap()["agents"].as_array().unwrap().is_empty());
        assert_eq!(broker.store().unwrap().store.agents(root).unwrap().len(), 56);
    }

    #[test]
    fn enrollment_requires_the_native_runs_captured_checkout_authority() {
        let fixture = Fixture::new();
        let owner =
            RunningBroker::start(&fixture.directory.path().join("uncaptured.sqlite")).unwrap();
        let request = client(&owner.broker, &fixture.main, "uncaptured");
        let actor = call(
            &owner.broker,
            &request,
            Operation::AgentRegister { name: "".into() },
        )
        .unwrap();
        owner
            .broker
            .store()
            .unwrap()
            .terminals
            .get_mut(&request.terminal)
            .unwrap()
            .physical_root = None;
        assert!(owner
            .broker
            .control(
                fixture.main.to_str().unwrap(),
                &ControllerOperation::WorktreeCoordinator {
                    root: fixture.main.to_str().unwrap().into(),
                    agent: actor["agent"]["id"].as_str().unwrap().into(),
                    run: request.run.unwrap(),
                    request_id: id(),
                }
            )
            .is_err());
        assert!(owner
            .broker
            .worktree_domain(fixture.main.to_str().unwrap())
            .unwrap()
            .is_none());
    }

    #[test]
    fn project_queries_omit_the_new_mode_field_for_older_companions() {
        let private = serde_json::to_value(PanelQuery::default()).unwrap();
        assert!(private.get("worktree").is_none());
        let team = serde_json::to_value(PanelQuery {
            worktree: true,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(team["worktree"], true);
        assert!(
            !serde_json::from_value::<PanelQuery>(private)
                .unwrap()
                .worktree
        );
        assert!(serde_json::from_value::<PanelQuery>(team).unwrap().worktree);
    }

    #[test]
    fn explicit_reselection_fences_old_coordinator_and_receipt_replay_cannot_restore_it() {
        let fixture = Fixture::new();
        let owner =
            RunningBroker::start(&fixture.directory.path().join("reselection.sqlite")).unwrap();
        let broker = &owner.broker;
        let first = client(broker, &fixture.main, "first");
        let replacement = client(broker, &fixture.main, "replacement");
        let selection = |request: &Request| {
            let actor = call(
                broker,
                request,
                Operation::AgentRegister {
                    name: String::new(),
                },
            )
            .unwrap();
            ControllerOperation::WorktreeCoordinator {
                root: fixture.main.to_str().unwrap().into(),
                agent: actor["agent"]["id"].as_str().unwrap().into(),
                run: request.run.clone().unwrap(),
                request_id: id(),
            }
        };
        let first_selection = selection(&first);
        broker
            .control(fixture.main.to_str().unwrap(), &first_selection)
            .unwrap();
        broker.readiness(&first.terminal, true);
        std::thread::sleep(Duration::from_millis(800));
        let stale_wake = broker
            .wakeups()
            .into_iter()
            .find(|wake| wake.terminal == first.terminal)
            .unwrap();
        assert!(broker.claim_wake(&stale_wake));
        broker
            .control(fixture.main.to_str().unwrap(), &selection(&replacement))
            .unwrap();
        assert!(call(broker, &first, assign("replacement")).is_err());
        assert!(call(
            broker,
            &first,
            Operation::AgentInbox {
                cursor: None,
                limit: None
            }
        )
        .is_ok());
        let replay = broker
            .control(fixture.main.to_str().unwrap(), &first_selection)
            .unwrap();
        let panel = broker
            .operator_panel(&PanelQuery {
                project: fixture.main.to_str().unwrap().into(),
                worktree: true,
                ..Default::default()
            })
            .unwrap();
        assert_ne!(panel["roles"][0]["agent"], replay["agent"]["id"]);
        assert_eq!(panel["coordinator_online"], true);
        assert!(!broker.wake_valid(&stale_wake));
        broker.end(&replacement.terminal);
        assert!(!broker
            .operator_panel(&PanelQuery {
                project: fixture.main.to_str().unwrap().into(),
                worktree: true,
                ..Default::default()
            })
            .unwrap()["coordinator_online"]
            .as_bool()
            .unwrap());
    }

    #[test]
    fn worktree_projection_does_not_adopt_private_tasks_before_coordinator_selection() {
        let fixture = Fixture::new();
        let owner =
            RunningBroker::start(&fixture.directory.path().join("project-mode.sqlite")).unwrap();
        let broker = &owner.broker;
        let issuer = client(broker, &fixture.main, "issuer");
        let worker = client(broker, &fixture.main, "worker");
        call(broker, &issuer, assign("worker")).unwrap();
        let project = PanelQuery {
            project: fixture.main.to_str().unwrap().into(),
            ..Default::default()
        };
        assert_eq!(
            broker.operator_panel(&project).unwrap()["tasks"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let team = broker
            .operator_panel(&PanelQuery {
                worktree: true,
                ..project
            })
            .unwrap();
        assert!(team["tasks"].as_array().unwrap().is_empty());
        assert_eq!(team["candidates"].as_array().unwrap().len(), 2);
        let actor = call(
            broker,
            &worker,
            Operation::AgentRegister {
                name: String::new(),
            },
        )
        .unwrap();
        assert!(broker
            .control(
                fixture.main.to_str().unwrap(),
                &ControllerOperation::WorktreeCoordinator {
                    root: fixture.main.to_str().unwrap().into(),
                    agent: actor["agent"]["id"].as_str().unwrap().into(),
                    run: worker.run.unwrap(),
                    request_id: id(),
                }
            )
            .is_err());
        let non_git = broker
            .operator_panel(&PanelQuery {
                project: fixture.directory.path().to_str().unwrap().into(),
                worktree: true,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(non_git["worktree_available"], false);
        assert!(non_git["tasks"].as_array().unwrap().is_empty());
    }

    #[test]
    fn explicit_coordinator_workers_peer_review_and_integration_preserve_project_history() {
        let fixture = Fixture::new();
        let owner = RunningBroker::start(&fixture.directory.path().join("roles.sqlite")).unwrap();
        let broker = &owner.broker;
        let coordinator = client(broker, &fixture.main, "coordinator");
        let worker = client(broker, &fixture.linked, "worker");
        let reviewer = client(broker, &fixture.unjoined, "reviewer");
        let private_project = fixture.main.to_str().unwrap();
        let old_id = call(
            broker,
            &coordinator,
            Operation::AgentRegister { name: "".into() },
        )
        .unwrap()["agent"]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let select = |request: &Request, root: &Path, lead: bool| {
            let agent = call(
                broker,
                request,
                Operation::AgentRegister { name: "".into() },
            )
            .unwrap()["agent"]["id"]
                .as_str()
                .unwrap()
                .to_owned();
            let root = root.to_str().unwrap().into();
            let run = request.run.clone().unwrap();
            let request_id = id();
            if lead {
                ControllerOperation::WorktreeCoordinator {
                    root,
                    agent,
                    run,
                    request_id,
                }
            } else {
                ControllerOperation::WorktreeWorker {
                    root,
                    agent,
                    run,
                    request_id,
                }
            }
        };
        assert!(broker
            .control(private_project, &select(&worker, &fixture.linked, false))
            .is_err());
        broker
            .control(private_project, &select(&coordinator, &fixture.main, true))
            .unwrap();
        assert!(broker
            .control(private_project, &select(&coordinator, &fixture.main, false))
            .is_err());
        broker
            .control(private_project, &select(&worker, &fixture.linked, false))
            .unwrap();
        broker
            .control(
                private_project,
                &select(&reviewer, &fixture.unjoined, false),
            )
            .unwrap();
        assert_eq!(broker.run(&coordinator.terminal), coordinator.run);
        assert!(call(broker, &worker, assign("reviewer")).is_err());
        let mut assignment = assign("worker");
        if let Operation::TaskAssign { reviewer, .. } = &mut assignment {
            *reviewer = Some("reviewer".into());
        }
        let task = call(broker, &coordinator, assignment).unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned();
        call(
            broker,
            &worker,
            Operation::TaskStart {
                task_id: task.clone(),
                revision: 1,
                expected_version: None,
                request_id: id(),
            },
        )
        .unwrap();
        assert!(call(
            broker,
            &coordinator,
            Operation::TaskIntegrate {
                task_id: task.clone(),
                commit: None,
                request_id: id()
            }
        )
        .is_err());
        call(
            broker,
            &worker,
            Operation::TaskSubmit {
                task_id: task.clone(),
                revision: 1,
                result: "Ready".into(),
                evidence: "Checks passed".into(),
                evidence_ids: vec![],
                attempt_id: None,
                expected_version: None,
                request_id: id(),
            },
        )
        .unwrap();
        let review = Operation::TaskReview {
            task_id: task.clone(),
            revision: 1,
            accepted: true,
            feedback: "Peer checked".into(),
            expected_version: None,
            request_id: id(),
        };
        assert!(call(broker, &worker, review.clone()).is_err());
        call(broker, &reviewer, review).unwrap();
        assert!(call(
            broker,
            &reviewer,
            Operation::TaskIntegrate {
                task_id: task.clone(),
                commit: None,
                request_id: id()
            }
        )
        .is_err());
        let commit = git(&fixture.main, &["rev-parse", "HEAD"]).trim().to_owned();
        call(
            broker,
            &coordinator,
            Operation::TaskIntegrate {
                task_id: task.clone(),
                commit: Some(commit),
                request_id: id(),
            },
        )
        .unwrap();
        let project = broker
            .operator_panel(&PanelQuery {
                project: private_project.into(),
                terminal: Some(coordinator.terminal.clone()),
                ..Default::default()
            })
            .unwrap();
        assert!(project["agents"].as_array().unwrap().iter().all(|row| row["online"] == true));
        assert_ne!(project["agents"][0]["agent"]["id"], old_id);
        assert_eq!(project["tasks"].as_array().unwrap().len(), 1);
        let team = broker
            .operator_panel(&PanelQuery {
                project: private_project.into(),
                worktree: true,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(team["roles"].as_array().unwrap().len(), 3);
        assert_eq!(team["coordinator_online"], true);
        let scope = team["project"].as_str().unwrap();
        let created = broker
            .control(
                scope,
                &ControllerOperation::WorktreeCreate {
                    root: private_project.into(),
                    name: "extra".into(),
                    base: "main".into(),
                    request_id: id(),
                },
            )
            .unwrap();
        assert_eq!(created["agent_started"], false);
        assert_eq!(
            broker.worktree_candidates(private_project).unwrap().len(),
            3
        );
        broker.end(&coordinator.terminal);
        broker
            .activate(&coordinator.terminal, "codex", private_project, false)
            .unwrap();
        let restarted = Request {
            run: broker.run(&coordinator.terminal),
            ..coordinator.clone()
        };
        let restarted_agent = call(
            broker,
            &restarted,
            Operation::AgentRegister { name: "".into() },
        )
        .unwrap();
        assert_eq!(restarted_agent["agent"]["project"], private_project);
        assert_eq!(
            broker
                .operator_panel(&PanelQuery {
                    project: private_project.into(),
                    worktree: true,
                    ..Default::default()
                })
                .unwrap()["coordinator_online"],
            false
        );
        broker.set_programs(Some(HashSet::new()));
        let disabled = broker
            .operator_panel(&PanelQuery {
                project: private_project.into(),
                worktree: true,
                ..Default::default()
            })
            .unwrap();
        assert!(disabled["roles"]
            .as_array()
            .unwrap()
            .iter()
            .all(|role| role["run"].is_null()));
        broker.set_programs(None);
        broker
            .activate(
                &worker.terminal,
                "codex",
                fixture.linked.to_str().unwrap(),
                false,
            )
            .unwrap();
        let restarted_worker = Request {
            run: broker.run(&worker.terminal),
            ..worker.clone()
        };
        let private_worker = call(
            broker,
            &restarted_worker,
            Operation::AgentRegister { name: "".into() },
        )
        .unwrap();
        assert_eq!(
            private_worker["agent"]["project"],
            fixture.linked.to_str().unwrap()
        );
    }

    #[test]
    fn existing_remote_endpoint_forwards_scope_and_native_lifecycle_without_restarting() {
        let fixture = Fixture::new();
        let source = RunningBroker::start(&fixture.directory.path().join("source.sqlite")).unwrap();
        let team = RunningBroker::start(&fixture.directory.path().join("team.sqlite")).unwrap();
        let request = client(&source.broker, &fixture.main, "coordinator");
        let agent = call(
            &source.broker,
            &request,
            Operation::AgentRegister { name: "".into() },
        )
        .unwrap()["agent"]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let operation = ControllerOperation::WorktreeCoordinator {
            root: fixture.main.to_str().unwrap().into(),
            agent,
            run: request.run.clone().unwrap(),
            request_id: id(),
        };
        team.broker
            .enroll_worktree(&source.broker, &operation)
            .unwrap();
        assert_eq!(source.broker.run(&request.terminal), request.run);
        source.broker.input_bytes(&request.terminal, b"draft");
        let panel = team
            .broker
            .operator_panel(&PanelQuery {
                project: fixture.main.to_str().unwrap().into(),
                worktree: true,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(panel["agents"][0]["draft"], "present");
        call(
            &source.broker,
            &request,
            Operation::AgentInbox {
                cursor: None,
                limit: None,
            },
        )
        .unwrap();
        source.broker.end(&request.terminal);
        source
            .broker
            .revoke_remote_run(&request.terminal, request.run.as_deref());
        assert!(source.broker.forwarded(&request.terminal).is_none());
        let next_run = id();
        let capability = source
            .broker
            .prepare_remote_run(&request.terminal, "codex", &fixture.main, &next_run)
            .unwrap();
        let restarted = Request {
            capability,
            run: Some(next_run.clone()),
            ..request.clone()
        };
        let restarted_agent = call(
            &source.broker,
            &restarted,
            Operation::AgentRegister { name: "".into() },
        )
        .unwrap();
        assert!(restarted_agent["agent"]["project"].as_str().unwrap().starts_with("space:"));
        let restarted_panel = source.broker.operator_panel(&PanelQuery {
            project:fixture.main.to_str().unwrap().into(), worktree:true,
            terminal:Some(request.terminal.clone()), ..Default::default()
        }).unwrap();
        assert_eq!(restarted_panel["coordinator_online"], false);
        assert!(restarted_panel["roles"].as_array().unwrap().iter().any(|role|
            role["agent"] == restarted_agent["agent"]["id"] && role["role"] == "worker"));
        source
            .broker
            .revoke_remote_run(&request.terminal, request.run.as_deref());
        assert_eq!(source.broker.run(&request.terminal), Some(next_run));
        assert!(call(
            &source.broker,
            &request,
            Operation::AgentInbox {
                cursor: None,
                limit: None
            }
        )
        .is_err());
        assert_eq!(
            team.broker
                .operator_panel(&PanelQuery {
                    project: fixture.main.to_str().unwrap().into(),
                    worktree: true,
                    ..Default::default()
                })
                .unwrap()["coordinator_online"],
            false
        );
    }
}
