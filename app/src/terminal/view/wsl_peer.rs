//! Native WSL terminal delivery; Linux owns run identity and MCP, Windows owns UI guards.
use super::*;
use crate::agent_communication::wsl_settings::Target;
use crate::terminal::cli_agent_sessions::{CLIAgentInputState, CLIAgentSession,
    CLIAgentSessionContext, CLIAgentSessionStatus};
use warp_agent_bus::{
    companion::TaskCommand,
    readiness::{Activity, Draft},
    ssh_remote::{HostClient, SshProfile},
    transport::Wake,
    wsl_setup::{NativeAction, NativeRequest, NativeState},
};

type Client = Arc<tokio::sync::Mutex<HostClient>>;

#[derive(Clone, PartialEq, Eq)]
struct Context {
    session: SessionId,
    target: Target,
    shell_pid: u32,
    root: String,
}

#[derive(Default)]
pub(super) struct State {
    context: Option<Context>,
    client: Option<Client>,
    busy: bool,
    attempted: Option<std::time::Instant>,
    draft: Draft,
    epoch: u64,
    submitted: bool,
    cancelled: bool,
    observed_run: Option<String>,
    unavailable: bool,
    configuration_generation: u64,
}

impl State {
    fn select(&mut self, context: Option<Context>) {
        if context == self.context {
            return;
        }
        self.context = context;
        self.client = None;
        self.busy = false;
        self.attempted = None;
        self.draft.clear();
        self.epoch = self.epoch.wrapping_add(1);
        self.submitted = false;
        self.cancelled = false;
        self.unavailable = false;
    }
    fn select_configuration(&mut self, generation: u64) {
        if self.configuration_generation == generation { return; }
        self.configuration_generation = generation;
        self.client = None;
        self.busy = false;
        self.attempted = None;
        self.unavailable = false;
        self.epoch = self.epoch.wrapping_add(1);
    }
    pub(super) fn user_input(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        self.epoch = self.epoch.wrapping_add(1);
        match self.draft.input(bytes) {
            Some(Activity::Working) => {
                self.submitted = true;
                self.cancelled = false;
            }
            Some(Activity::Cancelled) => {
                self.submitted = false;
                self.cancelled = true;
            }
            _ => {
                self.submitted = false;
            }
        }
    }
}

async fn request(
    client: &Client,
    shell_pid: u32,
    action: NativeAction,
) -> anyhow::Result<Option<NativeState>> {
    tokio::time::timeout(Duration::from_secs(10), async {
        let value = client
            .lock()
            .await
            .project_tasks(
                &TaskCommand::GuestNative(NativeRequest { shell_pid, action }),
                0,
            )
            .await
            .map_err(|_| anyhow::anyhow!("WSL native delivery unavailable"))?;
        let value = value
            .get("value")
            .ok_or_else(|| anyhow::anyhow!("Invalid WSL native response"))?;
        Ok(serde_json::from_value(value.clone())?)
    })
    .await?
}

impl TerminalView {
    pub(super) fn schedule_wsl_integration(&mut self, command: &str, ctx: &mut ViewContext<Self>) -> bool {
        if !crate::terminal::warpify::settings::WarpifySettings::is_interactive_wsl_command(command) { return false; }
        let Some(session) = self.active_block_session_id().and_then(|id| self.sessions_model().as_ref(ctx).get(id)) else { return false; };
        if session.is_wsl() || session.ssh_arguments().is_some() || session.is_legacy_ssh_session()
            || matches!(session.session_type(), crate::terminal::model::session::SessionType::WarpifiedRemote { .. }) { return false; }
        let program = command.split_whitespace().next().unwrap();
        if session.alias_value(program).is_some() || session.abbreviation_value(program).is_some()
            || session.function_names().any(|name| name.eq_ignore_ascii_case(program)) { return false; }
        self.wsl_peer.select_configuration(crate::agent_communication::AgentCommunication::as_ref(ctx).wsl.generation);
        let epoch = self.wsl_peer.epoch;
        let block = self.model.lock().block_list().active_block_id().clone();
        let future = ctx.spawn_abortable(Timer::after(Duration::from_millis(AUTO_WARPIFY_DELAY)), move |view, _, ctx| {
            // User input, cancellation or a replaced block invalidates delayed bootstrap.
            if view.wsl_peer.epoch != epoch || view.model.lock().block_list().active_block_id() != &block { return; }
            if view.active_block_session_id().and_then(|id| view.sessions_model().as_ref(ctx).get(id)).is_some_and(|session| session.is_wsl()) { return; }
            view.trigger_subshell_bootstrap(None, false, ctx);
        }, |_, _| ());
        self.warpify_state.add_auto_warpify_abort_handle(future);
        true
    }

    fn clear_observed_wsl_agent(&mut self, ctx: &mut ViewContext<Self>) {
        let Some(run) = self.wsl_peer.observed_run.take() else { return; };
        let id = self.view_id;
        CLIAgentSessionsModel::handle(ctx).update(ctx, |model, ctx| {
            if model.session(id).is_some_and(|session| session.listener.is_none()
                && session.session_context.session_id.as_deref() == Some(run.as_str())) {
                model.remove_session(id, ctx);
            }
        });
    }

    fn observe_wsl_agent(&mut self, context: &Context, state: Option<&NativeState>, ctx: &mut ViewContext<Self>) {
        let Some(state) = state else { self.clear_observed_wsl_agent(ctx); return; };
        if self.wsl_peer.observed_run.as_ref().is_some_and(|run| run != &state.run) {
            self.clear_observed_wsl_agent(ctx);
        }
        // Guest process identity proves participation even before any queued wake exists.
        if CLIAgentSessionsModel::as_ref(ctx).session(self.view_id).is_none() {
            let agent = enum_iterator::all::<CLIAgent>()
                .find(|agent| agent.command_prefix() == state.program).unwrap_or(CLIAgent::Unknown);
            let id = self.view_id;
            CLIAgentSessionsModel::handle(ctx).update(ctx, |model, ctx| model.set_session(id, CLIAgentSession {
                agent, status: CLIAgentSessionStatus::InProgress,
                session_context: CLIAgentSessionContext { cwd:Some(context.root.clone()),
                    session_id:Some(state.run.clone()), ..Default::default() },
                input_state:CLIAgentInputState::Closed, should_auto_toggle_input:false,
                listener:None, plugin_version:None, remote_host:Some(format!("wsl:{}@{}",
                    context.target.user, context.target.distribution)), draft_text:None,
                custom_command_prefix:(agent == CLIAgent::Unknown).then(|| state.program.clone()),
            }, ctx));
            self.wsl_peer.observed_run = Some(state.run.clone());
            ctx.notify();
        }
    }

    pub(super) fn record_wsl_peer_input(&mut self, bytes: &[u8], ctx: &AppContext) {
        let context = self.wsl_peer_context(ctx);
        self.wsl_peer.select(context);
        self.wsl_peer.user_input(bytes);
    }
    fn wsl_peer_context(&self, ctx: &AppContext) -> Option<Context> {
        if !self.is_long_running_and_user_controlled() {
            return None;
        }
        let session_id = self.active_block_session_id()?;
        let session = self.sessions_model().as_ref(ctx).get(session_id)?;
        let target = Target::from_session(&session)?;
        let shell_pid = session.wsl_shell_pid()?;
        let root = self
            .model
            .lock()
            .active_block_metadata()
            .current_working_directory()?
            .to_owned();
        Some(Context {
            session: session_id,
            target,
            shell_pid,
            root,
        })
    }

    fn wsl_peer_valid(&self, context: &Context, epoch: u64, ctx: &AppContext) -> bool {
        let settings = &crate::agent_communication::AgentCommunication::as_ref(ctx).wsl;
        settings.preferences.enabled
            && settings.targets.contains(&context.target)
            && self.wsl_peer.epoch == epoch
            && self.wsl_peer.draft.is_empty()
            && !self.wsl_peer.cancelled
            && self.wsl_peer_context(ctx).as_ref() == Some(context)
            && self.peer_input_is_empty(ctx)
    }

    pub(crate) fn poll_wsl_peer_work(
        &mut self,
        enabled: bool,
        targets: &[Target],
        ctx: &mut ViewContext<Self>,
    ) {
        let context = self.wsl_peer_context(ctx);
        self.wsl_peer.select(context.clone());
        let configuration_generation = crate::agent_communication::AgentCommunication::as_ref(ctx).wsl.generation;
        self.wsl_peer.select_configuration(configuration_generation);
        let Some(context) = context else {
            self.clear_observed_wsl_agent(ctx);
            return;
        };
        if !enabled || !targets.contains(&context.target) {
            self.wsl_peer.client = None;
            return;
        }
        if crate::agent_communication::AgentCommunication::as_ref(ctx).wsl.busy { return; }
        if self.wsl_peer.unavailable || self.wsl_peer.busy
            || self
                .wsl_peer
                .attempted
                .is_some_and(|time| time.elapsed() < Duration::from_secs(1))
        {
            return;
        }
        let (draft, blocked) = self.peer_input_guard(ctx);
        let native_ready = CLIAgentSessionsModel::as_ref(ctx)
            .session(self.view_id)
            .filter(|session| session.listener.is_some())
            .map(|session| {
                matches!(
                    session.status,
                    crate::terminal::cli_agent_sessions::CLIAgentSessionStatus::Success
                )
            });
        let epoch = self.wsl_peer.epoch;
        let action = NativeAction::Observe {
            draft: draft || !self.wsl_peer.draft.is_empty(),
            blocked,
            input_epoch: epoch,
            submitted: self.wsl_peer.submitted,
            cancelled: self.wsl_peer.cancelled,
            native_ready,
        };
        self.wsl_peer.busy = true;
        self.wsl_peer.attempted = Some(std::time::Instant::now());
        let cached = self.wsl_peer.client.clone();
        let selected = context.clone();
        ctx.spawn(async move {
            let client = match cached {
                Some(client) => client,
                None => {
                    let profile = SshProfile {
                        target: "wsl".into(), config_file: None, remote_root: selected.root.clone(),
                        companion_path: warp_agent_bus::installation::wsl_companion_path(&selected.target.home)
                            .map_err(|_| anyhow::anyhow!("Invalid Linux home"))?,
                        remote_shell: warp_agent_bus::ssh_remote::RemoteShell::Posix,
                    };
                    Arc::new(tokio::sync::Mutex::new(HostClient::connect_wsl(&profile,
                        &selected.target.distribution, &selected.target.user).await
                        .map_err(|_| anyhow::anyhow!("WSL service unavailable"))?))
                }
            };
            let state = request(&client, selected.shell_pid, action).await?;
            let state = match state {
                Some(state) if state.wake.is_some() => request(&client, selected.shell_pid,
                    NativeAction::Claim { wake: state.wake.unwrap() }).await?,
                state => state,
            };
            Ok::<_, anyhow::Error>((client, state))
        }, move |view, result, ctx| {
            if view.wsl_peer.context.as_ref() != Some(&context) || view.wsl_peer.configuration_generation != configuration_generation { return; }
            let (client, state) = match result {
                Ok(value) => value,
                Err(_) => { view.wsl_peer.client = None; view.wsl_peer.busy = false; view.wsl_peer.unavailable = true; return; }
            };
            view.wsl_peer.client = Some(client.clone());
            view.observe_wsl_agent(&context, state.as_ref(), ctx);
            let Some(state) = state else { view.wsl_peer.busy = false; return; };
            let Some(wake) = state.wake.filter(|_| state.valid) else { view.wsl_peer.busy = false; return; };
            let agent = CLIAgentSessionsModel::as_ref(ctx).session(view.view_id).map(|session| session.agent)
                .filter(|agent| agent.command_prefix() == state.program || *agent == CLIAgent::Unknown);
            if uuid::Uuid::parse_str(&wake.message_id).is_err() || agent.is_none()
                || !view.wsl_peer_valid(&context, epoch, ctx) {
                view.finish_wsl_wake(client, context, wake, false, ctx);
                return;
            }
            let text = format!("Warpai peer work is waiting (message {}). Call warp_agent_inbox, process this message or task, and acknowledge it through MCP. Before ending your turn, call warp_agent_ready.", wake.message_id);
            let delay = view.stage_wsl_peer_text(agent.unwrap(), text, ctx);
            let connection = client.clone();
            let claimed = wake.clone();
            let shell_pid = context.shell_pid;
            ctx.spawn(async move {
                Timer::after(delay).await;
                request(&connection, shell_pid, NativeAction::Validate { wake: claimed }).await
            }, move |view, result, ctx| {
                let valid = result.ok().flatten().is_some_and(|state| state.valid && state.run == wake.run);
                let submitted = valid && view.wsl_peer_valid(&context, epoch, ctx);
                if submitted { view.write_to_pty(b"\r".to_vec(), ctx); }
                view.finish_wsl_wake(client, context, wake, submitted, ctx);
            });
        });
    }

    fn finish_wsl_wake(
        &mut self,
        client: Client,
        context: Context,
        wake: Wake,
        submitted: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        let shell_pid = context.shell_pid;
        let configuration_generation = self.wsl_peer.configuration_generation;
        let selected_client = client.clone();
        ctx.spawn(async move { request(&client, shell_pid, NativeAction::Finish { wake, submitted }).await },
            move |view, result, _| {
                if view.wsl_peer.context.as_ref() == Some(&context)
                    && view.wsl_peer.configuration_generation == configuration_generation
                    && view.wsl_peer.client.as_ref().is_some_and(|client| Arc::ptr_eq(client, &selected_client)) {
                    view.wsl_peer.busy = false;
                    if result.is_err() { view.wsl_peer.client = None; view.wsl_peer.unavailable = true; }
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unavailable_service_requires_configuration_retry_without_losing_drafts() {
        let mut state = State::default();
        state.user_input(b"unsent draft");
        let epoch = state.epoch;
        state.unavailable = true;
        state.select_configuration(0);
        assert!(state.unavailable);
        assert_eq!(state.epoch, epoch);
        state.select_configuration(1);
        assert!(!state.unavailable);
        assert!(!state.draft.is_empty());
        assert_ne!(state.epoch, epoch);
    }
    #[test]
    fn guest_observations_detect_agents_without_wakes_and_keep_terminals_independent() {
        use crate::test_util::{add_window_with_terminal, terminal::initialize_app_for_terminal_view};
        warpui::App::test((), |mut app| async move {
            initialize_app_for_terminal_view(&mut app);
            let first = add_window_with_terminal(&mut app, None);
            let second = add_window_with_terminal(&mut app, None);
            let context = Context { session:1_u64.into(), target:Target {
                distribution:"Ubuntu Test".into(), user:"guest".into(), home:"/home/guest".into()
            }, shell_pid:123, root:"/home/guest/project".into() };
            let state = NativeState { run:"first-run".into(), program:"codex".into(), wake:None, valid:false };
            first.update(&mut app, |view, ctx| view.observe_wsl_agent(&context, Some(&state), ctx));
            let other = NativeState { run:"second-run".into(), program:"claude".into(), wake:None, valid:false };
            second.update(&mut app, |view, ctx| view.observe_wsl_agent(&context, Some(&other), ctx));
            assert_eq!(CLIAgentSessionsModel::as_ref(&app).session(first.id()).unwrap().agent, CLIAgent::Codex);
            assert_eq!(CLIAgentSessionsModel::as_ref(&app).session(second.id()).unwrap().agent, CLIAgent::Claude);
            first.update(&mut app, |view, ctx| {
                let mut saved = CLIAgentSessionsModel::as_ref(ctx).session(view.view_id).unwrap().clone();
                assert_eq!(saved.remote_host.as_deref(), Some("wsl:guest@Ubuntu Test"));
                assert_eq!(saved.session_context.cwd.as_deref(), Some(context.root.as_str()));
                saved.draft_text = Some("unsent prompt".into());
                CLIAgentSessionsModel::handle(ctx).update(ctx, |model, ctx| model.set_session(view.view_id, saved, ctx));
                view.observe_wsl_agent(&context, Some(&state), ctx);
                assert_eq!(CLIAgentSessionsModel::as_ref(ctx).session(view.view_id).unwrap().draft_text.as_deref(), Some("unsent prompt"));
                view.observe_wsl_agent(&context, None, ctx);
            });
            assert!(CLIAgentSessionsModel::as_ref(&app).session(first.id()).is_none());
            assert_eq!(CLIAgentSessionsModel::as_ref(&app).session(second.id()).unwrap().session_context.session_id.as_deref(), Some("second-run"));
        });
    }

    #[test]
    fn manual_input_and_cancel_fence_native_delivery() {
        let mut state = State::default();
        state.user_input("几个字".as_bytes());
        assert!(!state.draft.is_empty());
        assert_eq!(state.epoch, 1);
        state.user_input(b"\x7f\x7f\x7f");
        assert!(state.draft.is_empty());
        state.user_input(b"\x03");
        assert!(state.cancelled);
        assert!(!state.draft.is_empty());
        state.user_input(b"continue\r");
        assert!(state.submitted && !state.cancelled && state.draft.is_empty());
        assert_eq!(state.epoch, 4);
    }
}
