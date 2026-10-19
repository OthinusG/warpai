//! Local coordination for independently authenticated third-party CLI agents.
mod codex_launch;
pub(crate) mod ssh_settings;
#[cfg(windows)]
pub(crate) mod wsl_settings;
mod legacy_remote_credentials;
pub(crate) mod panel;
pub(crate) mod setup;
use crate::terminal::{
    cli_agent_sessions::{
        CLIAgentSessionStatus, CLIAgentSessionsModel, CLIAgentSessionsModelEvent,
    },
    CLIAgent, TerminalView,
};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
use warp_agent_bus::{
    project_root,
    transport::{Broker, RunningBroker, CAPABILITY, ENDPOINT, TERMINAL},
};
use warpui::r#async::Timer;
use warpui::{Entity, EntityId, ModelContext, SingletonEntity, ViewHandle, WeakViewHandle};
use warpui_extras::secure_storage::AppContextExt;

pub(crate) static BROKER: OnceLock<Broker> = OnceLock::new();
static VIEWS: OnceLock<Mutex<HashMap<EntityId, (String, WeakViewHandle<TerminalView>)>>> =
    OnceLock::new();
static NATIVE_EVIDENCE: OnceLock<Mutex<HashMap<EntityId, (Instant, Instant)>>> = OnceLock::new();
pub(crate) struct AgentCommunication {
    _server: Option<RunningBroker>,
    pub(crate) preferences: setup::Preferences,
    pub(crate) available: Vec<setup::Available>,
    pub(crate) busy: bool,
    pub(crate) status: String,
    pub(crate) ssh: ssh_settings::SshSettings,
    #[cfg(windows)]
    pub(crate) wsl: wsl_settings::WslSettings,
    preferences_path: PathBuf,
    pending: Option<std::sync::mpsc::Receiver<(setup::Preferences, Vec<setup::Available>, String)>>,
    notified: HashMap<String, String>,
    sampled: HashMap<String, warp_agent_bus::transport::ScreenContext>,
    poll_in_flight: bool,
    work_revision: Option<usize>,
    peer_work: Vec<warp_agent_bus::transport::Wake>,
    wakes: Vec<warp_agent_bus::transport::Wake>,
    observed: HashSet<String>,
    remote_sampled: HashMap<EntityId, (CLIAgent, Option<String>, Instant, Instant)>,
}
impl Entity for AgentCommunication {
    type Event = ();
}
impl SingletonEntity for AgentCommunication {}
impl AgentCommunication {
    #[cfg(test)]
    pub(crate) fn mock() -> Self {
        Self {
            _server: None,
            preferences: Default::default(),
            available: Vec::new(),
            busy: false,
            status: String::new(),
            ssh: Default::default(),
            #[cfg(windows)]
            wsl: Default::default(),
            preferences_path: PathBuf::new(),
            pending: None,
            notified: HashMap::new(),
            sampled: HashMap::new(),
            poll_in_flight: false,
            work_revision: None,
            peer_work: Vec::new(),
            wakes: Vec::new(),
            observed: HashSet::new(),
            remote_sampled: HashMap::new(),
        }
    }

    pub(crate) fn new(ctx: &mut ModelContext<Self>) -> Self {
        let directory =
            warp_core::paths::secure_state_dir().unwrap_or_else(warp_core::paths::state_dir);
        let server = std::fs::create_dir_all(&directory)
            .ok()
            .and_then(|_| RunningBroker::start(&directory.join("agent-communication.sqlite")).ok());
        if let Some(server) = &server {
            server
                .broker
                .set_programs(Some(std::collections::HashSet::new()));
            let _ = BROKER.set(server.broker.clone());
        }
        VIEWS.get_or_init(|| Mutex::new(HashMap::new()));
        NATIVE_EVIDENCE.get_or_init(|| Mutex::new(HashMap::new()));
        ctx.subscribe_to_model(&CLIAgentSessionsModel::handle(ctx), |model, event, ctx| {
            let Some(broker) = BROKER.get() else {
                return;
            };
            let Some(views) = VIEWS.get() else {
                return;
            };
            let Ok(views) = views.lock() else {
                return;
            };
            let Some((terminal, view)) = views.get(&event.terminal_view_id()) else {
                return;
            };
            match event {
                CLIAgentSessionsModelEvent::Started { agent, .. } => {
                    // Managed custom CLIs can participate through their verified native MCP client.
                    let program = if *agent == CLIAgent::Unknown {
                        "custom"
                    } else {
                        agent.command_prefix()
                    };
                    if !model.preferences.programs().contains(program) {
                        return;
                    }
                    if let Some(view) = view.upgrade(ctx) {
                        if view.as_ref(ctx).active_block_session_id()
                            .and_then(|id| view.as_ref(ctx).sessions_model().as_ref(ctx).get(id))
                            .is_some_and(|session| {
                                #[cfg(windows)]
                                if session.is_wsl() { return true; }
                                session.ssh_arguments().is_some()
                            }) { return; }
                        let terminal_model = view.as_ref(ctx).model.lock();
                        if let Some(directory) =
                            terminal_model.active_block_metadata().current_working_directory()
                        {
                            if let Ok(project) = project_root(Path::new(directory)) {
                                let command = terminal_model.block_list().active_block().command_to_string();
                                let view = view.as_ref(ctx);
                                let command = view.codex_original_command(command.trim());
                                let unshadowed = shlex::split(command)
                                    .and_then(|words| words.first().cloned())
                                    .is_some_and(|program| view.active_block_session_id()
                                        .and_then(|session| view.sessions_model().as_ref(ctx).get(session))
                                        .is_some_and(|shell| shell.alias_value(&program).is_none()
                                            && shell.abbreviation_value(&program).is_none()
                                            && !shell.function_names().any(|name| name == program.as_str())));
                                let launch_options = model.preferences.selected.values()
                                    .find(|entry| entry.active && entry.program == program)
                                    .map(|entry| &entry.launch_options);
                                let initial_prompt = unshadowed && agent.accepts_peer_prompt(command) && (agent.starts_at_empty_prompt(command, &launch_options.cloned().unwrap_or_default())
                                    || (*agent == CLIAgent::Unknown
                                        && model.preferences.selected.get(command)
                                            .is_some_and(|entry| entry.active)));
                                let project = if *agent == CLIAgent::Codex {
                                    launch_options.and_then(|options| shlex::split(command).map(|words| options.codex_working_directory(&words[1..], Path::new(directory))))
                                        .and_then(|directory| project_root(&directory).ok()).unwrap_or(project)
                                } else { project };
                                drop(terminal_model);
                                if let Err(error) = broker.activate_project(terminal, program, &project, initial_prompt) {
                                    let code = error.downcast_ref::<warp_agent_bus::DomainError>()
                                        .map(|error| error.code.as_str()).unwrap_or("coordinator_unavailable");
                                    log::warn!("Could not activate local agent communication: {code}");
                                }
                            }
                        }
                    }
                }
                CLIAgentSessionsModelEvent::Ended { .. } => broker.end(terminal),
                CLIAgentSessionsModelEvent::StatusChanged { status, .. } => {
                    if matches!(status, CLIAgentSessionStatus::Blocked { .. }) {
                        broker.activity(terminal, warp_agent_bus::readiness::Activity::WaitingApproval);
                        return;
                    }
                    // Opaque OSC notifications also announce approvals; they cannot prove idle.
                    if view.upgrade(ctx).is_some_and(|view| {
                        CLIAgentSessionsModel::as_ref(ctx)
                            .session(view.id())
                            .is_some_and(|session| {
                                crate::terminal::cli_agent_sessions::listener::agent_supports_rich_status(&session.agent)
                            })
                    }) {
                        match status {
                            CLIAgentSessionStatus::Success | CLIAgentSessionStatus::Idle => broker.readiness(terminal, true),
                            CLIAgentSessionStatus::InProgress => broker.readiness(terminal, false),
                            _ => {},
                        }
                    }
                }
                _ => {}
            }
        });
        if server.is_none() {
            log::warn!("Local agent communication is unavailable");
        }
        Self::schedule(ctx);
        let preferences_path = directory.join("agent-communication-settings.json");
        let mut preferences: setup::Preferences = match std::fs::read(&preferences_path) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => setup::Preferences::default(),
        };
        for profile in &mut preferences.legacy_remote_profiles {
            if profile.cleanup_pending {
                profile.cleanup_pending = legacy_remote_credentials::remove(
                    ctx.secure_storage(),
                    profile.coordinator,
                    profile.device,
                )
                .is_err();
            }
        }
        let mut model = Self {
            _server: server,
            preferences,
            available: vec![],
            busy: false,
            status: String::new(),
            ssh: ssh_settings::SshSettings::load(directory.join("ssh-communication-settings.json")),
            #[cfg(windows)]
            wsl: wsl_settings::WslSettings::load(directory.join("wsl-communication-settings.json")),
            preferences_path,
            pending: None,
            notified: HashMap::new(),
            sampled: HashMap::new(),
            poll_in_flight: false,
            work_revision: None,
            peer_work: Vec::new(),
            wakes: Vec::new(),
            observed: HashSet::new(),
            remote_sampled: HashMap::new(),
        };
        warp_agent_bus::ssh_files::set_ssh_enabled(model.ssh.preferences.enabled);
        model.configure(None, None, ctx);
        model
    }
    fn policy(&mut self) {
        if let Some(broker) = BROKER.get() {
            broker.set_programs(Some(self.preferences.programs()));
        }
    }
    pub(crate) fn configure(
        &mut self,
        enabled: Option<bool>,
        command: Option<String>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.configure_inner(enabled, command, false, ctx);
    }
    pub(crate) fn uninstall_all(&mut self, ctx: &mut ModelContext<Self>) {
        self.configure_inner(Some(false), None, true, ctx);
    }
    fn configure_inner(
        &mut self,
        enabled: Option<bool>,
        command: Option<String>,
        uninstall_all: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        if self.busy {
            return;
        }
        let mut preferences = self.preferences.clone();
        if let Some(enabled) = enabled {
            preferences.enabled = enabled;
            // Revocation precedes disk IO and cannot be undone by a failed cleanup.
            if !enabled {
                self.preferences.enabled = false;
                self.policy();
            }
        }
        if uninstall_all {
            preferences.enabled = false;
            self.preferences.enabled = false;
            self.policy();
        }
        for (name, entry) in &mut preferences.selected {
            if !preferences.enabled || command.as_ref() == Some(name) {
                entry.active = false;
            }
        }
        self.preferences = preferences.clone();
        self.policy();
        self.busy = true;
        self.status = if uninstall_all {
            "Removing Warpai MCP configuration from installed agents…".into()
        } else {
            "Updating native MCP configuration…".into()
        };
        let path = self.preferences_path.clone();
        let mut commands: Vec<(String, String)> = enum_iterator::all::<CLIAgent>()
            .flat_map(|agent| {
                agent
                    .command_prefixes()
                    .iter()
                    .map(move |command| (agent.command_prefix().to_owned(), (*command).to_owned()))
            })
            .collect();
        // Custom toolbar patterns can supply literal executable prefixes, without shell evaluation.
        for (pattern, vendor) in crate::settings::AISettings::as_ref(ctx)
            .cli_agent_footer_enabled_commands
            .iter()
        {
            let command = pattern
                .trim_start_matches('^')
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('$');
            if !command.is_empty()
                && command
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
            {
                let program =
                    serde_json::from_value::<CLIAgent>(serde_json::Value::String(vendor.clone()))
                        .ok()
                        .filter(|agent| *agent != CLIAgent::Unknown)
                        .map(|agent| agent.command_prefix().to_owned())
                        .unwrap_or_else(|| "custom".into());
                if !commands.iter().any(|(_, known)| known == command) {
                    commands.push((program, command.to_owned()));
                }
            }
        }
        let search_paths: Vec<PathBuf> = VIEWS
            .get()
            .and_then(|views| views.lock().ok())
            .map(|views| {
                views
                    .values()
                    .filter_map(|(_, view)| view.upgrade(ctx))
                    .flat_map(|view| {
                        let view = view.as_ref(ctx);
                        let Some(session) = view.active_block_session_id() else {
                            return vec![];
                        };
                        if !view.session_is_local(session, ctx) {
                            return vec![];
                        }
                        let sessions = view.sessions_model().as_ref(ctx);
                        let shell = sessions.get(session);
                        sessions
                            .env_var_for_session(session, "PATH")
                            .into_iter()
                            .chain(shell.as_ref().and_then(|session| session.path().as_deref()))
                            .flat_map(|path| std::env::split_paths(std::ffi::OsStr::new(path)))
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
            .unwrap_or_default();
        let (sender, receiver) = std::sync::mpsc::channel();
        commands.sort_by_key(|(_, command)| !preferences.selected.contains_key(command));
        self.pending = Some(receiver);
        std::thread::spawn(move || {
            let mut errors = Vec::new();
            let mut available = if preferences.enabled || uninstall_all {
                let bridge = if preferences.enabled {
                    setup::companion()
                } else {
                    setup::companion_path()
                };
                match bridge {
                    Ok(bridge) => setup::discover(commands, &bridge, search_paths),
                    Err(_) => {
                        errors.push(
                            "Could not determine the bundled communication bridge path".into(),
                        );
                        vec![]
                    }
                }
            } else {
                vec![]
            };
            if uninstall_all {
                for row in &available {
                    if let Some(mut entry) = row.installed.clone() {
                        entry.active = false;
                        preferences
                            .selected
                            .entry(row.command.clone())
                            .or_insert(entry);
                    } else {
                        errors.push(format!(
                            "{}: no safe MCP cleanup adapter is available for this installed agent version; configuration was left unchanged",
                            row.command
                        ));
                    }
                }
            }
            if setup::save_preferences(&path, &preferences).is_err() {
                preferences.enabled = false;
                let _ = sender.send((
                    preferences,
                    available,
                    "Could not save communication preferences; communication remains disabled"
                        .into(),
                ));
                return;
            }
            let was_selected = command
                .as_ref()
                .is_some_and(|name| preferences.selected.contains_key(name));
            let remove: Vec<String> = if !preferences.enabled {
                preferences.selected.keys().cloned().collect()
            } else {
                preferences
                    .selected
                    .iter()
                    .filter(|(name, entry)| !entry.active || command.as_ref() == Some(*name))
                    .map(|(name, _)| name.clone())
                    .collect()
            };
            for name in remove {
                let entry = &preferences.selected[&name];
                let cleanup = if uninstall_all {
                    setup::uninstall(entry)
                } else {
                    setup::configure(entry, false, true)
                };
                match cleanup {
                    Ok(()) => {
                        preferences.selected.remove(&name);
                    }
                    Err(error) => errors.push(format!(
                        "{name}: cleanup failed; configuration was preserved. Retry cleanup from Settings. {error:#}"
                    )),
                }
            }
            if preferences.enabled {
                if let Some(name) = &command {
                    if !was_selected
                        && !errors
                            .iter()
                            .any(|error| error.starts_with(&format!("{name}:")))
                    {
                        if let Some(entry) = available
                            .iter()
                            .find(|row| &row.command == name)
                            .and_then(|row| row.installed.clone())
                        {
                            if let Err(error) = setup::configure(&entry, false, false) {
                                errors.push(format!(
                                    "{name}: setup preflight failed; existing configuration was preserved. {error:#}"
                                ));
                            } else {
                                let mut pending_entry = entry.clone();
                                pending_entry.active = false;
                                preferences.selected.insert(name.clone(), pending_entry);
                                // Persist cleanup intent before invoking a vendor command or replacing its file.
                                if let Err(error) = setup::save_preferences(&path, &preferences) {
                                    errors.push(format!(
                                        "{name}: could not save setup intent; configuration was not changed. {error:#}"
                                    ));
                                } else if let Err(error) = setup::configure(&entry, true, false) {
                                    errors.push(format!(
                                        "{name}: setup failed; cleanup remains pending. {error:#}"
                                    ));
                                } else {
                                    preferences.selected.insert(name.clone(), entry);
                                }
                            }
                        }
                    }
                }
            }
            for row in &mut available {
                if let (Some(selected), Some(discovered)) = (
                    preferences.selected.get_mut(&row.command),
                    row.installed.as_ref(),
                ) {
                    selected.launch_options = discovered.launch_options.clone();
                    selected.program = discovered.program.clone();
                }
                if preferences.selected.contains_key(&row.command) {
                    row.status = if preferences.selected[&row.command].active {
                        if matches!(preferences.selected[&row.command].adapter, setup::Adapter::Vibe(_)) {
                            "Configured — open a new terminal pane, then start Vibe to load its native MCP environment"
                        } else {
                            "Configured — open a new terminal pane, then start this agent"
                        }
                    } else {
                        "Disabled — cleanup pending; uncheck or retry cleanup"
                    }
                    .into();
                }
            }
            if setup::save_preferences(&path, &preferences).is_err() {
                preferences.enabled = false;
                errors.push(
                    "Could not save communication preferences; communication remains disabled"
                        .into(),
                );
            }
            let status = if errors.is_empty() {
                if uninstall_all {
                    "Communication MCP cleanup completed for all discovered agents.".into()
                } else if !preferences.enabled {
                    "Communication is off.".into()
                } else if preferences.selected.is_empty() {
                    "Select installed agents to configure communication.".into()
                } else {
                    "Open a new terminal pane and start the selected agents. Agents in the same project communicate automatically.".into()
                }
            } else {
                errors.join("\n")
            };
            let _ = sender.send((preferences, available, status));
        });
        ctx.notify();
    }
    fn schedule(ctx: &mut ModelContext<Self>) {
        ctx.spawn(Timer::after(Duration::from_millis(300)), |model, _, ctx| {
            warp_agent_bus::ssh_files::set_ssh_enabled(model.ssh.preferences.enabled);
            model.refresh_ssh_targets(ctx);
            #[cfg(windows)]
            {
                model.refresh_wsl_targets(ctx);
            }
            for view in ssh_settings::logged_in_views(ctx) {
                view.update(ctx, |view, ctx| view.poll_remote_peer_work(model, ctx));
            }
            if let Some(result) = model
                .pending
                .as_ref()
                .and_then(|receiver| receiver.try_recv().ok())
            {
                model.preferences = result.0;
                model.available = result.1;
                model.status = result.2;
                model.busy = false;
                model.pending = None;
                model.policy();
                ctx.notify();
            }
            if let (Some(broker), Some(views)) = (BROKER.get(), VIEWS.get()) {
                let bindings: Vec<_> = views
                    .lock()
                    .map(|mut views| {
                        views.retain(|_, (terminal, view)| {
                            let alive = view.upgrade(ctx).is_some();
                            if !alive {
                                broker.end(terminal);
                            }
                            alive
                        });
                        views.values().cloned().collect()
                    })
                    .unwrap_or_default();
                let pending = model.peer_work.clone();
                let mut screens = Vec::new();
                let mut remote_screens = Vec::new();
                model.sampled.retain(|terminal, _| bindings.iter().any(|(id, _)| id == terminal));
                model.remote_sampled.retain(|id, _| bindings.iter().any(|(_, view)| view.id() == *id));
                if let Some(evidence) = NATIVE_EVIDENCE.get() {
                    if let Ok(mut evidence) = evidence.lock() {
                        evidence.retain(|id, _| bindings.iter().any(|(_, view)| view.id() == *id));
                    }
                }
                for (terminal, view) in &bindings {
                    if let Some(view) = view.upgrade(ctx) {
                        if !model.poll_in_flight { broker.screen_context(terminal).and_then(|context| {
                            // Coalesce redraws and keystrokes; unchanged screens need no extraction.
                            if model.sampled.get(terminal).is_some_and(|previous| context.same_screen(previous)) {
                                return None;
                            }
                            if context.input_quiet() < Duration::from_millis(200) { return None; }
                            let native = view.as_ref(ctx);
                            let title = native.terminal_title_text();
                            screens.push((terminal.clone(), context, native.model.clone(), title));
                            Some(())
                        }); }
                        if !model.poll_in_flight && broker.screen_context(terminal).is_none() {
                            let native = view.as_ref(ctx);
                            if let Some(session) = CLIAgentSessionsModel::as_ref(ctx).session(view.id()).filter(|session| session.is_remote()) {
                                #[cfg(windows)]
                                let is_wsl = session.remote_host.as_deref().is_some_and(|host| host.starts_with("wsl:"));
                                #[cfg(not(windows))]
                                let is_wsl = false;
                                let evidence = NATIVE_EVIDENCE.get().and_then(|evidence| evidence.lock().ok())
                                    .and_then(|evidence| evidence.get(&view.id()).copied());
                                if let Some((input, output)) = evidence.filter(|(input, _)| !is_wsl && input.elapsed() >= Duration::from_millis(200)) {
                                    let key = (session.agent, session.session_context.session_id.clone(), input, output);
                                    if model.remote_sampled.get(&view.id()) != Some(&key) {
                                        remote_screens.push((view.id(), key, native.model.clone(), native.terminal_title_text()));
                                    }
                                }
                            }
                        }
                        let observed = model.observed.remove(terminal);
                        if let Some(activity) = broker.terminal_activity(terminal) {
                            let id = view.id();
                            CLIAgentSessionsModel::handle(ctx).update(ctx, |sessions, ctx| {
                                sessions.observe_activity(id, activity, observed, ctx);
                                sessions.observe_pending_peer_work(id, pending.iter().any(|work| work.terminal == *terminal
                                    && broker.run(terminal).as_deref() == Some(work.run.as_str())), ctx);
                            });
                        }
                        let (draft, blocked) = view.as_ref(ctx).peer_input_guard(ctx);
                        broker.composer_guard(terminal, draft, blocked);
                    }
                }
                model.notified.retain(|terminal, message| {
                    pending.iter().any(|work| &work.terminal == terminal && &work.message_id == message)
                });
                for work in pending {
                    if model.notified.get(&work.terminal) == Some(&work.message_id) {
                        continue;
                    }
                    if let Some((_, view)) = bindings.iter().find(|(terminal, _)| terminal == &work.terminal) {
                        if let Some(view) = view.upgrade(ctx) {
                            view.update(ctx, |_, ctx| {
                                ctx.emit(crate::terminal::view::Event::ShowToast {
                                    message: warpui::localization::text("Peer work received. It will start when this agent is ready.").into(),
                                    flavor: crate::view_components::ToastFlavor::Default,
                                });
                            });
                            model.notified.insert(work.terminal, work.message_id);
                        }
                    }
                }
                for wake in std::mem::take(&mut model.wakes) {
                    if let Some((_, view)) = bindings
                        .iter()
                        .find(|(terminal, _)| terminal == &wake.terminal)
                    {
                        if let Some(view) = view.upgrade(ctx) {
                            let broker = broker.clone();
                            view.update(ctx, |view, ctx| {
                                view.wake_for_peer_work(broker, wake, ctx)
                            });
                        }
                    }
                }
                if !model.poll_in_flight && (!screens.is_empty() || !remote_screens.is_empty()
                    || !model.peer_work.is_empty()
                    || broker.work_revision().is_none_or(|revision| model.work_revision != Some(revision))) {
                    model.poll_in_flight = true;
                    let broker = broker.clone();
                    ctx.spawn(async move {
                        tokio::task::spawn_blocking(move || {
                            let mut sampled = Vec::new();
                            let mut remote_sampled = Vec::new();
                            for (terminal, context, native, title) in screens {
                                let quiet = context.quiet();
                                let Some(native) = native.try_lock() else { continue; };
                                let (screen, cursor) = native.agent_screen();
                                let progress = native.agent_progress().to_owned();
                                drop(native);
                                let activity = warp_agent_bus::screen_detection::activity_with_progress(
                                    &context.program, &screen, &title, &progress, quiet, cursor);
                                let observed = activity.is_some_and(|activity| broker.observe_screen(&terminal, &context, activity));
                                if observed || activity.is_none() && quiet >= Duration::from_millis(700) {
                                    sampled.push((terminal, context, observed));
                                }
                            }
                            for (view, key, native, title) in remote_screens {
                                let quiet = key.2.elapsed().min(key.3.elapsed());
                                let Some(native) = native.try_lock() else { continue; };
                                let (screen, cursor) = native.agent_screen();
                                let progress = native.agent_progress().to_owned();
                                drop(native);
                                let activity = warp_agent_bus::screen_detection::activity_with_progress(
                                    key.0.command_prefix(), &screen, &title, &progress, quiet, cursor);
                                if activity.is_some() || quiet >= Duration::from_millis(700) {
                                    remote_sampled.push((view, key, activity));
                                }
                            }
                            let revision = broker.work_revision();
                            (sampled, remote_sampled, broker.pending_work(), broker.wakeups(), revision)
                        }).await
                    }, |model, result, ctx| {
                        model.poll_in_flight = false;
                        if let Ok((sampled, remote_sampled, pending, wakes, revision)) = result {
                            model.work_revision = revision;
                            model.peer_work = pending;
                            model.wakes = wakes;
                            for (terminal, context, observed) in sampled {
                                if observed { model.observed.insert(terminal.clone()); }
                                model.sampled.insert(terminal, context);
                            }
                            for (view, key, activity) in remote_sampled {
                                let valid = CLIAgentSessionsModel::as_ref(ctx).session(view).is_some_and(|session|
                                    session.is_remote() && session.agent == key.0 && session.session_context.session_id == key.1)
                                    && NATIVE_EVIDENCE.get().and_then(|evidence| evidence.lock().ok())
                                        .is_some_and(|evidence| evidence.get(&view) == Some(&(key.2, key.3)));
                                if !valid { continue; }
                                model.remote_sampled.insert(view, key);
                                if let Some(activity) = activity {
                                    CLIAgentSessionsModel::handle(ctx).update(ctx, |sessions, ctx|
                                        sessions.observe_activity(view, activity, true, ctx));
                                }
                            }
                        }
                    });
                }
            }
            Self::schedule(ctx);
        });
    }
}
/// Tests and remote terminals without the native singleton receive no capabilities.
pub(crate) fn prepare(
    env: &mut HashMap<OsString, OsString>,
    directory: Option<&Path>,
    workspace: Option<&warp_agent_bus::WorkspaceBinding>,
    ctx: &warpui::AppContext,
) -> Option<String> {
    let broker = BROKER.get()?;
    let terminal = env.get(&OsString::from(TERMINAL))?.to_str()?.to_owned();
    let settings = AgentCommunication::as_ref(ctx);
    let capability = if let Some(workspace) = workspace {
        if !settings.preferences.enabled
            || directory
                .and_then(|root| project_root(root).ok())
                .as_deref()
                != Some(workspace.root.as_str())
        {
            return None;
        }
        broker.prepare_bound_workspace(&terminal, workspace).ok()?
    } else {
        broker.prepare(&terminal).ok()?
    };
    env.insert(ENDPOINT.into(), broker.endpoint.clone().into());
    env.insert(CAPABILITY.into(), capability.into());
    if settings.preferences.enabled {
        if let Some(installed) = settings
            .preferences
            .selected
            .values()
            .find(|entry| entry.active && matches!(entry.adapter, setup::Adapter::Vibe(_)))
        {
            if setup::vibe_environment(installed, env, directory).is_err() {
                log::warn!("Could not prepare native Vibe communication environment; existing settings were preserved");
            }
        }
    }
    if let Ok(executable) = std::env::current_exe() {
        let companion = executable.with_file_name(if cfg!(windows) {
            "warpai-agent.exe"
        } else {
            "warpai-agent"
        });
        env.insert("WARP_AGENT_BIN".into(), companion.clone().into_os_string());
    }
    Some(terminal)
}
pub(crate) fn accepts_peer_prompt(
    agent: &CLIAgent,
    command: &str,
    ctx: &warpui::AppContext,
) -> bool {
    if !agent.accepts_peer_prompt(command) {
        return false;
    }
    if *agent != CLIAgent::Codex {
        return true;
    }
    let Some(words) = shlex::split(command) else {
        return false;
    };
    let fallback = warp_agent_bus::launch::LaunchOptions::from_help("codex", "");
    if warp_agent_bus::session::codex_accepts_peer_prompt(&words[1..], &fallback) {
        return true;
    }
    AgentCommunication::as_ref(ctx)
        .preferences
        .selected
        .values()
        .find(|entry| entry.active && entry.program == "codex")
        .is_some_and(|entry| {
            warp_agent_bus::session::codex_accepts_peer_prompt(&words[1..], &entry.launch_options)
        })
}

pub(crate) fn bind(view: &ViewHandle<TerminalView>, terminal: Option<String>) {
    if let (Some(terminal), Some(views)) = (terminal, VIEWS.get()) {
        if let Ok(mut views) = views.lock() {
            views.insert(view.id(), (terminal, view.downgrade()));
            if let Some(evidence) = NATIVE_EVIDENCE.get() {
                if let Ok(mut evidence) = evidence.lock() {
                    let now = Instant::now(); evidence.insert(view.id(), (now, now));
                }
            }
        }
    }
}

fn terminal_for_view(view: EntityId) -> Option<String> {
    VIEWS
        .get()?
        .lock()
        .ok()?
        .get(&view)
        .map(|(terminal, _)| terminal.clone())
}
pub(crate) fn user_input(view: EntityId, bytes: &[u8]) {
    if !bytes.is_empty() {
        if let Some(evidence) = NATIVE_EVIDENCE.get() {
            if let Ok(mut evidence) = evidence.lock() {
                if let Some((input, _)) = evidence.get_mut(&view) { *input = Instant::now(); }
            }
        }
    }
    if let (Some(broker), Some(terminal)) = (BROKER.get(), terminal_for_view(view)) {
        broker.input_bytes(&terminal, bytes);
    }
}
pub(crate) fn output(view: EntityId) {
    if let Some(evidence) = NATIVE_EVIDENCE.get() {
        if let Ok(mut evidence) = evidence.lock() {
            if let Some((_, output)) = evidence.get_mut(&view) { *output = Instant::now(); }
        }
    }
    if let (Some(broker), Some(terminal)) = (BROKER.get(), terminal_for_view(view)) {
        broker.output(&terminal);
    }
}

pub(crate) fn readiness(view: EntityId, ready: bool) {
    if let (Some(broker), Some(terminal)) = (BROKER.get(), terminal_for_view(view)) {
        broker.readiness(&terminal, ready);
    }
}
