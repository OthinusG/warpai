//! SSH account selection and remote-owned native MCP preferences.
use super::{setup, AgentCommunication};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};
use warp_agent_bus::{ssh_files::SshConnection, ssh_remote::SshProfile};
use warpui::{AppContext, ModelContext, SingletonEntity};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Target {
    pub key: String,
    pub label: String,
    pub profile: SshProfile,
    pub connection: SshConnection,
    pub search_paths: Vec<String>,
}

impl Target {
    pub(crate) fn from_session(session: &crate::terminal::model::session::Session,
        directory: &str, app: &AppContext) -> Option<Self> {
        if crate::remote_server::selected_session::session_connection(session)?.is_wsl() { return None; }
        let (profile, connection) = crate::remote_server::selected_session::confirmed_profile(session, directory, app)?;
        if connection.is_wsl() { return None; }
        let separator = if profile.remote_shell == warp_agent_bus::ssh_remote::RemoteShell::PowerShell { ';' } else { ':' };
        let search_paths = session.path().as_deref().unwrap_or("").split(separator)
            .filter(|path| if separator == ':' { path.starts_with('/') } else {
                let bytes = path.as_bytes();
                path.starts_with(r"\\") || bytes.len() >= 3 && bytes[1] == b':' && matches!(bytes[2], b'\\' | b'/')
            }).take(128).map(str::to_owned).collect();
        Some(Self { key: connection.scope_key(),
            label: format!("{} · {}", session.hostname(), session.user()),
            profile, connection, search_paths })
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Preferences {
    pub enabled: bool,
    pub selected: Option<String>,
}

#[derive(Default)]
pub(crate) struct SshSettings {
    pub preferences: Preferences,
    pub targets: Vec<Target>,
    pub available: Vec<setup::Available>,
    pub remote_preferences: setup::Preferences,
    pub account_id: Option<String>,
    verified_service: Option<String>,
    pub status: String,
    pub busy: bool,
    pub generation: u64,
    #[cfg(debug_assertions)]
    pub capture_fixture: bool,
    path: PathBuf,
}

impl SshSettings {
    pub fn load(path: PathBuf) -> Self {
        let preferences = std::fs::read(&path).ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default();
        Self { preferences, path, ..Default::default() }
    }

    pub fn selected_target(&self) -> Option<&Target> {
        self.targets.iter().find(|target| Some(&target.key) == self.preferences.selected.as_ref())
    }
}

pub(crate) fn logged_in_views(app: &AppContext) -> Vec<warpui::ViewHandle<crate::terminal::TerminalView>> {
    let mut views = Vec::new();
    for window in app.window_ids() {
        let live: std::collections::HashSet<_> = app.views_of_type::<crate::workspace::Workspace>(window)
            .unwrap_or_default().into_iter().flat_map(|workspace|
                workspace.as_ref(app).list_tab_pane_groups(app).into_iter()
                    .flat_map(|group| group.terminal_ids)).collect();
        views.extend(app.views_of_type::<crate::terminal::TerminalView>(window).unwrap_or_default()
            .into_iter().filter(|view| live.contains(&view.id())));
    }
    views
}

impl AgentCommunication {
    pub(crate) fn refresh_ssh_targets(&mut self, ctx: &mut ModelContext<Self>) {
        #[cfg(debug_assertions)]
        if self.ssh.capture_fixture { return; }
        let mut targets = BTreeMap::new();
        for handle in logged_in_views(ctx) {
            let view = handle.as_ref(ctx);
            if view.has_pending_ssh_command() { continue; }
            let Some(session) = view.active_block_session_id()
                .and_then(|id| view.sessions_model().as_ref(ctx).get(id)) else { continue; };
            let Some(directory) = view.pwd() else { continue; };
            if let Some(target) = Target::from_session(&session, &directory, ctx) {
                targets.entry(target.key.clone()).or_insert(target);
            }
        }
        let targets: Vec<_> = targets.into_values().collect();
        if self.ssh.targets == targets { return; }
        self.ssh.targets = targets;
        if self.ssh.selected_target().is_none() {
            self.ssh.preferences.selected = self.ssh.targets.first().map(|target| target.key.clone());
            self.ssh.available.clear();
            self.ssh.remote_preferences = Default::default();
            self.ssh.account_id = None;
            self.ssh.verified_service = None;
            self.ssh.status.clear();
            self.ssh.generation += 1;
            if self.ssh.preferences.enabled && self.ssh.selected_target().is_some() {
                let generation = self.ssh.generation;
                let selected = self.ssh.preferences.selected.clone();
                // One scan on account discovery; failed scans wait for explicit retry.
                ctx.spawn(async {}, move |model, _, ctx| {
                    if model.ssh.preferences.enabled && !model.ssh.busy
                        && model.ssh.generation == generation
                        && model.ssh.preferences.selected == selected {
                        model.configure_ssh(warp_agent_bus::wsl_setup::Action::Rescan, ctx);
                    }
                });
            }
        }
        ctx.notify();
    }

    pub(crate) fn select_ssh_target(&mut self, key: Option<String>, ctx: &mut ModelContext<Self>) {
        if self.ssh.busy { return; }
        if key.as_ref().is_some_and(|key| !self.ssh.targets.iter().any(|target| &target.key == key)) { return; }
        self.ssh.preferences.selected = key;
        self.ssh.generation += 1;
        self.ssh.available.clear();
        self.ssh.remote_preferences = Default::default();
        self.ssh.account_id = None;
        self.ssh.verified_service = None;
        self.ssh.status.clear();
        self.save_ssh_preferences();
        if self.ssh.preferences.enabled { self.configure_ssh(warp_agent_bus::wsl_setup::Action::Rescan, ctx); }
        ctx.notify();
    }

    fn save_ssh_preferences(&mut self) {
        if self.ssh.path.as_os_str().is_empty() { return; }
        let result = serde_json::to_vec_pretty(&self.ssh.preferences)
            .map_err(anyhow::Error::from).and_then(|bytes| setup::atomic_write(&self.ssh.path, &bytes));
        if result.is_err() { self.ssh.status = "Could not save SSH Remote preferences.".into(); }
    }

    pub(crate) fn configure_ssh(&mut self, action: warp_agent_bus::wsl_setup::Action,
        ctx: &mut ModelContext<Self>) {
        use warp_agent_bus::wsl_setup::Action;
        #[cfg(debug_assertions)]
        if self.ssh.capture_fixture { return; }
        if self.ssh.busy { return; }
        self.refresh_ssh_targets(ctx);
        if let Action::Enable(enabled) = action {
            self.ssh.preferences.enabled = enabled;
            warp_agent_bus::ssh_files::set_ssh_enabled(enabled);
            self.ssh.generation += 1;
            self.save_ssh_preferences();
        } else {
            self.ssh.generation += 1;
        }
        let selected = self.ssh.selected_target().cloned();
        // The master switch revokes all currently reachable accounts, once per verified service.
        let targets = if matches!(action, Action::Enable(false)) {
            self.ssh.targets.clone()
        } else { selected.clone().into_iter().collect() };
        if targets.is_empty() {
            self.ssh.status = "No logged-in SSH accounts. Connect in a Warpai terminal first.".into();
            ctx.notify();
            return;
        }
        if !self.ssh.preferences.enabled && !matches!(action, Action::Enable(false) | Action::RemoveAll) { return; }
        let generation = self.ssh.generation;
        self.ssh.busy = true;
        self.ssh.status = "Updating MCP settings on the selected SSH account…".into();
        let selected_key = selected.map(|target| target.key);
        let expected = self.ssh.verified_service.clone().zip(self.ssh.account_id.clone());
        let expected_key = selected_key.clone();
        ctx.notify();
        let disabling = matches!(action, Action::Enable(false));
        ctx.spawn(async move {
            let mut services = std::collections::HashSet::new();
            let mut results = Vec::new();
            for target in targets {
                let result = async {
                    let mut client = if matches!(action, Action::Enable(false) | Action::RemoveAll) {
                        target.connection.connect_for_mcp_cleanup(&target.profile).await
                    } else { target.connection.connect(&target.profile).await }.map_err(anyhow::Error::new)?;
                    let service = (client.fence().ok_or_else(|| anyhow::anyhow!("Missing SSH identity"))?.service_id.clone(), client.account_id.clone());
                    if Some(&target.key) == expected_key.as_ref() {
                        anyhow::ensure!(expected.as_ref().is_none_or(|identity| identity == &service), "SSH account identity changed; select the account again");
                    }
                    if !services.insert(service.clone()) { return Ok(None); }
                    setup_on_client(&mut client, &target, action.clone(), generation, false).await.map(|response| Some((response, service.0)))
                }.await;
                results.push((target.key, result));
            }
            results
        }, move |model, results, ctx| {
            model.ssh.busy = false;
            if model.ssh.generation != generation || model.ssh.preferences.selected != selected_key {
                model.ssh.status = "SSH selection changed. Rescan the original account to verify its MCP settings.".into();
                ctx.notify();
                return;
            }
            let mut failed = false;
            for (key, result) in results {
                match result {
                    Ok(Some((response, service))) if Some(&key) == selected_key.as_ref() => {
                        model.ssh.remote_preferences = response.preferences;
                        model.ssh.available = response.available;
                        model.ssh.account_id = Some(response.account_id);
                        model.ssh.verified_service = Some(service);
                        model.ssh.status = response.status;
                    }
                    Ok(_) => (),
                    Err(error) => {
                        failed = true;
                        model.ssh.status = if error.downcast_ref::<warp_agent_bus::ssh_remote::ConnectionError>()
                            == Some(&warp_agent_bus::ssh_remote::ConnectionError::FeatureUnavailable) {
                            "Update Warpai Companion on this SSH account, then rescan.".into()
                        } else {
                            "SSH MCP setup could not be confirmed. Check the connection, then rescan before retrying.".into()
                        };
                    }
                }
            }
            if failed && disabling {
                model.ssh.status = "SSH Remote is off. MCP cleanup could not be confirmed for every account; reconnect and rescan before retrying.".into();
            }
            ctx.notify();
        });
    }
}

pub(crate) async fn setup_on_target(target: &Target, action: warp_agent_bus::wsl_setup::Action,
    generation: u64) -> anyhow::Result<warp_agent_bus::ssh_setup::Response> {
    let mut client = target.connection.connect(&target.profile).await.map_err(anyhow::Error::new)?;
    setup_on_client(&mut client, target, action, generation, true).await
}

async fn setup_on_client(client: &mut warp_agent_bus::ssh_remote::HostClient, target: &Target,
    action: warp_agent_bus::wsl_setup::Action, generation: u64, codex_only: bool) -> anyhow::Result<warp_agent_bus::ssh_setup::Response> {
    let commands = enum_iterator::all::<crate::terminal::CLIAgent>().filter(|agent| !codex_only || agent.command_prefix() == "codex").flat_map(|agent|
        agent.command_prefixes().iter().map(move |name| (agent.command_prefix().to_owned(), (*name).to_owned()))).collect();
    let value = client.project_tasks(&warp_agent_bus::companion::TaskCommand::SshSetup(
        warp_agent_bus::ssh_setup::Request { action, commands, search_paths: target.search_paths.clone() }
    ), generation).await.map_err(anyhow::Error::new)?;
    if value.get("error").is_some() { anyhow::bail!("SSH MCP setup failed"); }
    let response: warp_agent_bus::ssh_setup::Response = serde_json::from_value(value["value"].clone())?;
    anyhow::ensure!(response.account_id == client.account_id, "SSH account changed");
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_master_is_off_by_default_and_preferences_do_not_include_transport_arguments() {
        let settings = SshSettings::default();
        assert!(!settings.preferences.enabled);
        let value = serde_json::to_value(&settings.preferences).unwrap();
        assert_eq!(value, serde_json::json!({"enabled":false,"selected":null}));
    }

    #[test]
    fn ssh_master_toggles_without_changing_local_or_wsl_preferences() {
        warpui::App::test((), |mut app| async move {
            crate::test_util::terminal::initialize_app_for_terminal_view(&mut app);
            AgentCommunication::handle(&mut app).update(&mut app, |model, ctx| {
                model.ssh.path = PathBuf::new();
                model.preferences.enabled = true;
                #[cfg(windows)]
                let wsl_enabled = model.wsl.preferences.enabled;
                model.configure_ssh(warp_agent_bus::wsl_setup::Action::Enable(true), ctx);
                assert!(model.ssh.preferences.enabled);
                assert!(model.ssh.targets.is_empty());
                model.configure_ssh(warp_agent_bus::wsl_setup::Action::Enable(false), ctx);
                assert!(!model.ssh.preferences.enabled);
                assert!(model.preferences.enabled);
                #[cfg(windows)]
                assert_eq!(model.wsl.preferences.enabled, wsl_enabled);
            });
        });
    }

    #[test]
    fn stale_account_selection_never_resolves_another_target() {
        let mut settings = SshSettings::default();
        settings.preferences.selected = Some("old-connection".into());
        settings.targets.push(Target {
            key: "new-connection".into(), label: "fixture · user".into(),
            profile: SshProfile { target: "fixture".into(), config_file: None,
                remote_root: "/project".into(), companion_path: "/companion".into(),
                remote_shell: warp_agent_bus::ssh_remote::RemoteShell::Posix },
            connection: SshConnection::Native { arguments: vec!["fixture".into()], session: "new".into() },
            search_paths: vec![],
        });
        assert!(settings.selected_target().is_none());
        settings.preferences.selected = Some("new-connection".into());
        assert_eq!(settings.selected_target().unwrap().key, "new-connection");
    }
}
