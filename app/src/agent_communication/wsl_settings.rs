//! Windows selection is persisted independently; discovery executes only in the selected guest.
use super::{setup, AgentCommunication};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::{Arc, Mutex}, time::{Duration, Instant}};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use warpui::{AppContext, ModelContext, SingletonEntity};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Target {
    pub distribution: String,
    pub user: String,
    pub home: String,
}

impl Target {
    pub(crate) fn from_session(session: &crate::terminal::model::session::Session) -> Option<Self> {
        if session.ssh_arguments().is_some() || session.is_legacy_ssh_session() {
            return None;
        }
        Self::new(
            session.wsl_distro_name()?,
            session.user(),
            session.home_dir()?,
        )
    }

    fn new(distribution: &str, user: &str, home: &str) -> Option<Self> {
        let target = Self {
            distribution: distribution.into(),
            user: user.into(),
            home: home.into(),
        };
        if ![target.distribution.as_str(), target.user.as_str()]
            .into_iter()
            .all(|value| {
                !value.is_empty()
                    && value.len() <= 256
                    && !value.starts_with('-')
                    && !value.chars().any(char::is_control)
            })
            || warp_agent_bus::installation::wsl_companion_path(&target.home).is_err()
        {
            return None;
        }
        Some(target)
    }

    pub fn label(&self) -> String {
        format!("{} · {}", self.distribution, self.user)
    }
}

pub(crate) fn logged_in_views(app: &AppContext) -> Vec<warpui::ViewHandle<crate::terminal::TerminalView>> {
    let mut views = Vec::new();
    for window in app.window_ids() {
        let live_terminals: std::collections::HashSet<_> = app
            .views_of_type::<crate::workspace::Workspace>(window)
            .unwrap_or_default()
            .into_iter()
            .flat_map(|workspace| {
                workspace
                    .as_ref(app)
                    .list_tab_pane_groups(app)
                    .into_iter()
                    .flat_map(|group| group.terminal_ids)
            })
            .collect();
        for view in app
            .views_of_type::<crate::terminal::TerminalView>(window)
            .unwrap_or_default()
        {
            if !live_terminals.contains(&view.id()) {
                continue;
            }
            views.push(view);
        }
    }
    views
}

fn logged_in_targets(app: &AppContext) -> Vec<Target> {
    let mut targets = std::collections::BTreeMap::new();
    for view in logged_in_views(app) {
            let view = view.as_ref(app);
            let Some(session) = view
                .active_block_session_id()
                .and_then(|id| view.sessions_model().as_ref(app).get(id))
            else {
                continue;
            };
            let Some(target) = Target::from_session(&session) else {
                continue;
            };
            if !crate::terminal::wsl::WslInfo::as_ref(app)
                .distributions()
                .any(|distribution| {
                    distribution.name == target.distribution
                        && distribution.supports_communication()
                })
            {
                continue;
            }
            targets
                .entry((target.distribution.clone(), target.user.clone()))
                .or_insert(target);
    }
    targets.into_values().collect()
}

#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Preferences {
    pub enabled: bool,
    pub distribution: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
}

#[derive(Default)]
pub(crate) struct WslSettings {
    pub preferences: Preferences,
    pub agents: Vec<(String, String)>,
    pub available: Vec<setup::Available>,
    pub guest_preferences: setup::Preferences,
    pub account: Option<String>,
    pub status: String,
    pub busy: bool,
    pub companion_installed: Option<bool>,
    pub targets: Vec<Target>,
    pub installation: Option<Arc<Mutex<InstallationProgress>>>,
    pub(crate) generation: u64,
    path: PathBuf,
}

pub(crate) struct InstallationProgress {
    pub account: String,
    pub started: Instant,
    pub elapsed: Option<Duration>,
    pub phases: Vec<&'static str>,
    pub transferred: usize,
    pub total: usize,
    pub result: Option<String>,
}

impl InstallationProgress {
    fn new(target: &Target) -> Self {
        Self { account: target.label(), started: Instant::now(), elapsed: None,
            phases: vec!["Preparing Companion installation…"], transferred: 0, total: 0, result: None }
    }
    fn phase(&mut self, phase: &'static str) {
        if self.phases.last().copied() != Some(phase) && self.phases.len() < 10 {
            self.phases.push(phase);
        }
    }
    fn guest_phase(line: &str) -> Option<&'static str> {
        match line {
            "warpai-install:identity" => Some("Verifying the WSL account…"),
            "warpai-install:extract" => Some("Extracting the Companion bundle…"),
            "warpai-install:install" => Some("Installing Companion and its Git runtime…"),
            "warpai-install:verify" => Some("Verifying the installed Companion…"),
            "warpai-install:cleanup" => Some("Removing guest MCP configuration…"),
            "warpai-install:stop" => Some("Waiting for Companion connections to close…"),
            _ => None,
        }
    }
}

impl WslSettings {
    pub fn selected_target(&self) -> Option<&Target> {
        self.targets.iter().find(|target| {
            self.preferences.distribution.as_deref() == Some(target.distribution.as_str())
                && self.preferences.user.as_deref() == Some(target.user.as_str())
        })
    }
    pub fn load(path: PathBuf) -> Self {
        let preferences = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self {
            preferences,
            path,
            ..Default::default()
        }
    }
}

impl AgentCommunication {
    pub(crate) fn refresh_wsl_targets(&mut self, ctx: &mut ModelContext<Self>) {
        crate::terminal::wsl::WslInfo::handle(ctx).update(ctx, |model, ctx| model.refresh(ctx));
        let targets = logged_in_targets(ctx);
        if self.wsl.targets == targets {
            return;
        }
        self.wsl.targets = targets;
        if self.wsl.selected_target().is_none() {
            if self.wsl.busy {
                if let Some(progress) = &self.wsl.installation {
                    let mut progress = progress.lock().unwrap();
                    if progress.result.is_none() {
                        progress.elapsed = Some(progress.started.elapsed());
                        progress.result = Some("The WSL account disconnected. Installation may be partial; reconnect and rescan.".into());
                    }
                }
            }
            self.wsl.generation = self.wsl.generation.wrapping_add(1);
            self.wsl.busy = false;
            self.wsl.agents.clear();
            self.wsl.available.clear();
            self.wsl.account = None;
            self.wsl.companion_installed = None;
            self.wsl.status =
                "Log into WSL in a Warpai terminal before configuring communication.".into();
        }
        ctx.notify();
    }

    pub(crate) fn configure_wsl(
        &mut self,
        enabled: Option<bool>,
        target: Option<Target>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.configure_wsl_inner(enabled, target, None, None, ctx);
    }

    pub(crate) fn select_wsl_agent(&mut self, command: String, ctx: &mut ModelContext<Self>) {
        if self.wsl.busy || !self.wsl.preferences.enabled {
            return;
        }
        self.configure_wsl_inner(
            None,
            None,
            Some(warp_agent_bus::wsl_setup::Action::Select(command)),
            None,
            ctx,
        );
    }

    pub(crate) fn maintain_wsl(&mut self, remove_all: bool, ctx: &mut ModelContext<Self>) {
        if self.wsl.busy || !self.wsl.preferences.enabled || self.wsl.selected_target().is_none() {
            return;
        }
        let action = if remove_all {
            warp_agent_bus::wsl_setup::Action::RemoveAll
        } else {
            warp_agent_bus::wsl_setup::Action::Rescan
        };
        self.configure_wsl_inner(None, None, Some(action), None, ctx);
    }

    pub(crate) fn install_wsl_companion(&mut self, ctx: &mut ModelContext<Self>) {
        self.change_wsl_companion(self.wsl.companion_installed != Some(true), ctx);
    }

    pub(crate) fn change_wsl_companion(&mut self, install: bool, ctx: &mut ModelContext<Self>) {
        if !self.wsl.preferences.enabled || self.wsl.selected_target().is_none() {
            return;
        }
        self.configure_wsl_inner(
            None,
            None,
            None,
            Some(install),
            ctx,
        );
    }

    fn configure_wsl_inner(
        &mut self,
        enabled: Option<bool>,
        target: Option<Target>,
        action: Option<warp_agent_bus::wsl_setup::Action>,
        install: Option<bool>,
        ctx: &mut ModelContext<Self>,
    ) {
        if self.wsl.busy {
            return;
        }
        self.refresh_wsl_targets(ctx);
        if target
            .as_ref()
            .is_some_and(|target| !self.wsl.targets.contains(target))
        {
            return;
        }
        let settings = &mut self.wsl;
        settings.generation = settings.generation.wrapping_add(1);
        let generation = settings.generation;
        if let Some(enabled) = enabled {
            settings.preferences.enabled = enabled;
        }
        if let Some(target) = target {
            settings.preferences.distribution = Some(target.distribution);
            settings.preferences.user = Some(target.user);
        }
        settings.agents.clear();
        settings.available.clear();
        settings.account = None;
        settings.companion_installed = None;
        settings.busy = false;
        settings.status.clear();
        let save = serde_json::to_vec_pretty(&settings.preferences)
            .map_err(anyhow::Error::from)
            .and_then(|bytes| setup::atomic_write(&settings.path, &bytes));
        if save.is_err() {
            settings.status = "Could not save WSL communication settings.".into();
            ctx.notify();
            return;
        }
        let Some(target) = settings.selected_target().cloned() else {
            settings.status = "Log into WSL in a Warpai terminal, then select its account.".into();
            ctx.notify();
            return;
        };
        let commands: Vec<(String, String)> = enum_iterator::all::<crate::terminal::CLIAgent>()
            .flat_map(|agent| {
                agent
                    .command_prefixes()
                    .iter()
                    .map(move |name| (agent.command_prefix().to_owned(), (*name).to_owned()))
            })
            .collect();
        let action = if install.is_some() {
            warp_agent_bus::wsl_setup::Action::Rescan
        } else {
            action.unwrap_or(warp_agent_bus::wsl_setup::Action::Enable(
                settings.preferences.enabled,
            ))
        };
        settings.busy = true;
        settings.status = "Checking agents in the selected WSL distribution…".into();
        let progress = install.map(|_| Arc::new(Mutex::new(InstallationProgress::new(&target))));
        if let Some(progress) = &progress { settings.installation = Some(progress.clone()); }
        let selected = target.clone();
        ctx.spawn(async move {
            if let Some(install) = install {
                if install { install_companion_with_progress(&target, progress.as_ref()).await?; }
                else {
                    if let Some(progress) = &progress { progress.lock().unwrap().phase("Removing Companion from the selected WSL account…"); }
                    uninstall_companion(&target, progress.as_ref()).await?;
                    return Ok((None, None));
                }
            }
            if let Some(progress) = &progress {
                progress.lock().unwrap().phase("Checking Companion service and agent configuration…");
            }
            match guest_setup(&target, warp_agent_bus::wsl_setup::Request { action, commands: commands.clone() }).await {
                Ok(response) => Ok((Some(response), None)),
                Err(_) if install == Some(true) => Err(anyhow::anyhow!("Companion service verification failed. Rescan this account before retrying.")),
                Err(error) if error.is::<MissingCompanion>() => {
                    let commands: Vec<_> = commands.into_iter().map(|(_, command)| command).collect();
                    discover(&target, &commands).await.map(|fallback| (None, Some(fallback)))
                }
                Err(error) => Err(error)
            }
        }, move |model, result, ctx| {
            model.refresh_wsl_targets(ctx);
            let settings = &mut model.wsl;
            if settings.generation != generation { return; }
            if !settings.targets.contains(&selected) {
                settings.busy = false;
                if install.is_some() {
                    if let Some(progress) = &settings.installation {
                        let mut progress = progress.lock().unwrap();
                        progress.elapsed = Some(progress.started.elapsed());
                        progress.result = Some("The WSL account disconnected. Installation may be partial; reconnect and rescan.".into());
                    }
                }
                ctx.notify(); return;
            }
            settings.busy = false;
            match result {
                Ok((Some(response), _)) if response.account == selected.user => {
                    settings.companion_installed = Some(true);
                    settings.account = Some(response.account);
                    settings.available = response.available;
                    settings.guest_preferences = response.preferences;
                    settings.status = response.status;
                }
                Ok((_, Some((account, agents)))) if account == selected.user => {
                    settings.companion_installed = Some(false);
                    settings.account = Some(account);
                    settings.agents = agents;
                    settings.status = if settings.agents.is_empty() { "No installed CLI agents found in this WSL account." }
                        else { "Install or update WSL Companion to configure guest MCP." }.into();
                }
                Ok((None, None)) if install == Some(false) => {
                    settings.companion_installed = Some(false);
                    settings.guest_preferences = Default::default();
                    settings.status = "WSL Companion removed from this account. SSH and project data were preserved.".into();
                }
                Ok(_) => settings.status = "WSL MCP setup unavailable.".into(),
                Err(error) => {
                    settings.status = "WSL setup did not complete. Its result may be partial; rescan this account before retrying changes.".into();
                    if install.is_some() {
                        if let Some(progress) = &settings.installation {
                            progress.lock().unwrap().result = Some(error.to_string());
                        }
                    }
                },
            }
            if install.is_some() {
                if let Some(progress) = &settings.installation {
                    let mut progress = progress.lock().unwrap();
                    progress.elapsed = Some(progress.started.elapsed());
                    if progress.result.is_none() { progress.result = Some(settings.status.clone()); }
                }
            }
            ctx.notify();
        });
        ctx.notify();
    }
}

fn wsl_command(target: &Target) -> Result<tokio::process::Command> {
    ensure!(
        [target.distribution.as_str(), target.user.as_str()]
            .into_iter()
            .all(|value| !value.is_empty()
                && value.len() <= 256
                && !value.starts_with('-')
                && !value.chars().any(char::is_control)),
        "Invalid confirmed WSL account"
    );
    let mut command = tokio::process::Command::new("wsl.exe");
    warp_agent_bus::session::without_terminal_binding(&mut command);
    command.env_remove("VIBE_MCP_SERVERS");
    command.args([
        "--distribution",
        &target.distribution,
        "--user",
        &target.user,
    ]);
    use std::os::windows::process::CommandExt;
    command.as_std_mut().creation_flags(0x08000000);
    Ok(command)
}

#[cfg(test)]
async fn install_companion(target: &Target) -> Result<()> {
    install_companion_with_progress(target, None).await
}

async fn install_companion_with_progress(target: &Target, progress: Option<&Arc<Mutex<InstallationProgress>>>) -> Result<()> {
    if let Some(progress) = progress { progress.lock().unwrap().phase("Verifying the bundled archive checksum…"); }
    let payload = bundled_companion_payload().await?;
    const SCRIPT: &str = "set -eu; echo warpai-install:identity; test \"$(id -un)\" = \"$1\"; test \"$HOME\" = \"$2\"; test \"$(uname -m)\" = x86_64; directory=$(mktemp -d); trap 'rm -rf \"$directory\"' EXIT HUP INT TERM; echo warpai-install:extract; tar -xz -C \"$directory\"; echo warpai-install:install; sh \"$directory/install-unix.sh\" >/dev/null 2>&1; echo warpai-install:verify; \"$2/.config/.warpai/wsl/bin/warpai-wsl-companion\" --check-runtime >/dev/null 2>&1";
    run_installation(target, SCRIPT, Some(payload), progress).await
}

async fn bundled_companion_payload() -> Result<Vec<u8>> {
    use sha2::Digest;
    let resources = std::env::current_exe()?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Windows application directory unavailable"))?
        .join("resources");
    #[cfg(test)]
    let resources = std::env::var_os("WARP_TEST_WSL_BUNDLE").map(PathBuf::from).unwrap_or(resources);
    let path = resources.join("wsl-companion.tar.gz");
    let mut file = tokio::fs::File::open(&path)
        .await.context("Bundled WSL Companion is missing or unreadable. Repair the Windows installation.")?
        .take(128 * 1024 * 1024 + 1);
    let mut payload = Vec::new();
    file.read_to_end(&mut payload).await?;
    ensure!(
        payload.len() <= 128 * 1024 * 1024,
        "WSL payload limit exceeded"
    );
    let digest = tokio::fs::read_to_string(resources.join("wsl-companion.tar.gz.sha256")).await
        .context("Bundled WSL Companion checksum file is missing or unreadable.")?;
    ensure!(
        format!("{:x}", sha2::Sha256::digest(&payload)) == digest.trim(),
        "WSL payload checksum mismatch"
    );
    Ok(payload)
}

async fn uninstall_companion(target: &Target, progress: Option<&Arc<Mutex<InstallationProgress>>>) -> Result<()> {
    // Leave guest settings/history and the original SSH installation intact.
    // Recovery uses the trusted bundle, never the potentially damaged installed binary.
    const SCRIPT: &str = "set -eu; test \"$(id -un)\" = \"$1\"; test \"$HOME\" = \"$2\"; directory=$(mktemp -d); trap 'rm -rf \"$directory\"' EXIT HUP INT TERM; tar -xz -C \"$directory\"; printf '%s' '{\"action\":{\"action\":\"remove_all\"},\"commands\":[]}' | \"$directory/warpai-wsl-companion\" recover-uninstall";
    run_installation(target, SCRIPT, Some(bundled_companion_payload().await?), progress).await
}

async fn run_installation(target: &Target, script: &str, payload: Option<Vec<u8>>, progress: Option<&Arc<Mutex<InstallationProgress>>>) -> Result<()> {
    static MUTATIONS: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    let _mutation = MUTATIONS.get_or_init(|| tokio::sync::Mutex::new(())).try_lock()
        .map_err(|_| anyhow::anyhow!("Another WSL installation is active"))?;
    let mut command = wsl_command(target)?;
    command
        .args([
            "--exec",
            "sh",
            "-c",
            script,
            "warpai-wsl-installation",
            &target.user,
            &target.home,
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    let mut child = command.spawn().context("Could not start the selected WSL distribution.")?;
    let output = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Missing installation output"))?;
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Missing installation input"))?;
    tokio::time::timeout(Duration::from_secs(180), async {
        let transfer = async {
            if let Some(payload) = payload {
                if let Some(progress) = progress { progress.lock().unwrap().total = payload.len(); }
                for chunk in payload.chunks(64 * 1024) {
                    input.write_all(chunk).await.context("Could not transfer the Companion archive into WSL.")?;
                    if let Some(progress) = progress { progress.lock().unwrap().transferred += chunk.len(); }
                }
            }
            input.shutdown().await?;
            drop(input);
            let status = child.wait().await?;
            ensure!(status.success(), "{}", warpui::localization::format_text("WSL installer exited with status {status}. Inspect the last phase and rescan before retrying.", &[("status", &status.code().map_or_else(|| "unknown".into(), |code| code.to_string()))]));
            Ok::<_, anyhow::Error>(())
        };
        let observe = async {
            let mut lines = tokio::io::BufReader::new(output.take(8192)).lines();
            while let Some(line) = lines.next_line().await? {
                if let Some(phase) = InstallationProgress::guest_phase(&line) {
                    if let Some(progress) = progress { progress.lock().unwrap().phase(phase); }
                }
            }
            Ok::<_, anyhow::Error>(())
        };
        tokio::try_join!(transfer, observe)?;
        Ok(())
    }).await.context("WSL installation timed out after 180 seconds. Changes may be partial; rescan before retrying.")?
}

#[derive(Debug)]
struct MissingCompanion;
impl std::fmt::Display for MissingCompanion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WSL Companion is not installed")
    }
}
impl std::error::Error for MissingCompanion {}

pub(super) async fn guest_setup(
    target: &Target,
    request: warp_agent_bus::wsl_setup::Request,
) -> Result<warp_agent_bus::wsl_setup::Response> {
    const SCRIPT: &str =
        "test \"$(id -un)\" = \"$2\" || exit 126; test -x \"$1\" || exit 127; exec \"$1\" setup";
    let companion = warp_agent_bus::installation::wsl_companion_path(&target.home)
        .map_err(|_| anyhow::anyhow!("Invalid confirmed Linux home"))?;
    let mut command = wsl_command(target)?;
    command
        .args([
            "--exec",
            "sh",
            "-lc",
            SCRIPT,
            "warpai-wsl-setup",
            &companion,
            &target.user,
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    let mut child = command.spawn()?;
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Missing guest input"))?;
    let mut output = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Missing guest output"))?
        .take(512 * 1024 + 1);
    tokio::time::timeout(Duration::from_secs(180), async {
        let bytes = serde_json::to_vec(&request)?;
        ensure!(bytes.len() <= 65536, "Setup request limit exceeded");
        input.write_all(&bytes).await?;
        input.shutdown().await?;
        drop(input);
        let mut bytes = Vec::new();
        output.read_to_end(&mut bytes).await?;
        let status = child.wait().await?;
        if status.code() == Some(127) {
            return Err(MissingCompanion.into());
        }
        ensure!(
            bytes.len() <= 512 * 1024 && status.success(),
            "Guest setup failed"
        );
        let response: warp_agent_bus::wsl_setup::Response = serde_json::from_slice(&bytes)?;
        ensure!(response.account == target.user, "WSL account changed");
        Ok(response)
    })
    .await?
}

async fn discover(target: &Target, commands: &[String]) -> Result<(String, Vec<(String, String)>)> {
    // Arguments stay literal; the fixed login-shell script resolves the guest's own PATH.
    const SCRIPT: &str = "set -eu; printf '%s\\n' \"$(id -un)\"; for name do path=$(command -v \"$name\" 2>/dev/null) || continue; case $path in /*) printf '%s\\t%s\\n' \"$name\" \"$path\";; esac; done";
    ensure!(
        commands.iter().all(|name| !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))),
        "Invalid Agent command"
    );
    let mut command = wsl_command(target)?;
    command
        .args(["--exec", "sh", "-lc", SCRIPT, "warpai-wsl-discovery"])
        .args(commands)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    let mut child = command.spawn()?;
    let mut output = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Missing guest output"))?
        .take(65537);
    tokio::time::timeout(Duration::from_secs(20), async {
        let mut bytes = Vec::new();
        output.read_to_end(&mut bytes).await?;
        ensure!(bytes.len() <= 65536, "Guest output limit exceeded");
        ensure!(child.wait().await?.success(), "Guest discovery failed");
        let result = parse(&bytes, commands)?;
        ensure!(result.0 == target.user, "WSL account changed");
        Ok(result)
    })
    .await?
}

fn parse(bytes: &[u8], commands: &[String]) -> Result<(String, Vec<(String, String)>)> {
    let text = std::str::from_utf8(bytes)?;
    let mut lines = text.lines();
    let account = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing account"))?;
    ensure!(
        !account.is_empty() && account.len() <= 256 && !account.chars().any(char::is_control),
        "Invalid account"
    );
    let mut agents = Vec::new();
    for line in lines {
        let (name, path) = line
            .split_once('\t')
            .ok_or_else(|| anyhow::anyhow!("Invalid guest entry"))?;
        ensure!(
            commands.iter().any(|command| command == name)
                && path.starts_with('/')
                && path.len() <= 4096
                && !path.chars().any(char::is_control),
            "Invalid guest executable"
        );
        if !agents.iter().any(|(command, _)| command == name) {
            agents.push((name.into(), path.into()));
        }
    }
    Ok((account.into(), agents))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installation_progress_accepts_only_fixed_phases_and_bounds_history() {
        let target = Target::new("Ubuntu Test", "guest", "/home/guest").unwrap();
        let mut progress = InstallationProgress::new(&target);
        for line in ["token=hidden", "warpai-install:unknown", "warpai-install:identity\n"] {
            assert!(InstallationProgress::guest_phase(line).is_none());
        }
        let phase = InstallationProgress::guest_phase("warpai-install:install").unwrap();
        progress.phase(phase);
        progress.phase(phase);
        assert_eq!(progress.phases.len(), 2);
        for _ in 0..20 {
            progress.phase("Verifying the installed Companion…");
            progress.phase(phase);
        }
        assert_eq!(progress.phases.len(), 10);
        assert_eq!(progress.account, "Ubuntu Test · guest");
    }
    #[tokio::test]
    #[ignore = "requires the bundled payload and disposable Windows WSL guest"]
    async fn bundled_wsl_install_reinstall_uninstall_preserves_ssh() {
        assert_eq!(std::env::var("GITHUB_ACTIONS").as_deref(), Ok("true"));
        let target = Target::new(&std::env::var("WARP_TEST_WSL_DISTRIBUTION").unwrap(),
            "warpai-other", "/home/warpai-other").unwrap();
        let mut fixture = wsl_command(&target).unwrap();
        let status = fixture.args(["--exec", "sh", "-lc",
            "set -eu; mkdir -p \"$HOME/.config/.warpai/bin\"; printf '%s' 'Owned SSH Companion fixture' > \"$HOME/.config/.warpai/bin/warpai-companion\""]).status().await.unwrap();
        assert!(status.success());
        let progress = Arc::new(Mutex::new(InstallationProgress::new(&target)));
        install_companion_with_progress(&target, Some(&progress)).await.unwrap();
        {
            let progress = progress.lock().unwrap();
            assert!(progress.total > 0);
            assert_eq!(progress.transferred, progress.total);
            assert!(progress.phases.contains(&"Verifying the installed Companion…"));
        }
        install_companion(&target).await.unwrap();
        let response = guest_setup(&target, warp_agent_bus::wsl_setup::Request {
            action: warp_agent_bus::wsl_setup::Action::Rescan, commands: vec![],
        }).await.unwrap();
        assert_eq!(response.account, target.user);
        uninstall_companion(&target, None).await.unwrap();
        uninstall_companion(&target, None).await.unwrap();
        assert!(guest_setup(&target, warp_agent_bus::wsl_setup::Request {
            action: warp_agent_bus::wsl_setup::Action::Rescan, commands: vec![],
        }).await.err().unwrap().is::<MissingCompanion>());
        let mut verify = wsl_command(&target).unwrap();
        assert!(verify.args(["--exec", "sh", "-lc",
            "set -eu; test \"$(cat \"$HOME/.config/.warpai/bin/warpai-companion\")\" = 'Owned SSH Companion fixture'; test -z \"$(find \"$HOME/.config/.warpai/wsl/bin\" -mindepth 1 -print -quit)\""]).status().await.unwrap().success());
        install_companion(&target).await.unwrap();
        let mut damage = wsl_command(&target).unwrap();
        assert!(damage.args(["--exec", "sh", "-c",
            "set -eu; directory=\"$HOME/.config/.warpai/wsl/bin\"; printf broken > \"$directory/warpai-wsl-companion\"; for path in \"$directory\"/companion-runtime-*/SHA256SUMS; do printf broken > \"$path\"; done"]).status().await.unwrap().success());
        assert!(guest_setup(&target, warp_agent_bus::wsl_setup::Request {
            action: warp_agent_bus::wsl_setup::Action::Rescan, commands: vec![],
        }).await.is_err());
        uninstall_companion(&target, None).await.unwrap();
        install_companion(&target).await.unwrap();
        assert_eq!(guest_setup(&target, warp_agent_bus::wsl_setup::Request {
            action: warp_agent_bus::wsl_setup::Action::Rescan, commands: vec![],
        }).await.unwrap().account, target.user);
        let mut missing = wsl_command(&target).unwrap();
        assert!(missing.args(["--exec", "sh", "-c", "rm -f \"$HOME/.config/.warpai/wsl/bin/warpai-wsl-companion\""]).status().await.unwrap().success());
        uninstall_companion(&target, None).await.unwrap();
        install_companion(&target).await.unwrap();
        uninstall_companion(&target, None).await.unwrap();
        let mut preserved = wsl_command(&target).unwrap();
        assert!(preserved.args(["--exec", "sh", "-c",
            "set -eu; test \"$(cat \"$HOME/.config/.warpai/bin/warpai-companion\")\" = 'Owned SSH Companion fixture'; test -z \"$(find \"$HOME/.config/.warpai/wsl/bin\" -mindepth 1 -print -quit)\""]).status().await.unwrap().success());
        let mut linked = wsl_command(&target).unwrap();
        assert!(linked.args(["--exec", "sh", "-c",
            "set -eu; ln -s \"$HOME/.config/.warpai/bin/warpai-companion\" \"$HOME/.config/.warpai/wsl/bin/warpai-wsl-companion\""]).status().await.unwrap().success());
        assert!(uninstall_companion(&target, None).await.is_err());
        let mut retained = wsl_command(&target).unwrap();
        assert!(retained.args(["--exec", "sh", "-c",
            "set -eu; test \"$(cat \"$HOME/.config/.warpai/bin/warpai-companion\")\" = 'Owned SSH Companion fixture'; rm \"$HOME/.config/.warpai/wsl/bin/warpai-wsl-companion\"; printf preserve > \"$HOME/.config/.warpai/wsl/bin/unrelated.txt\""]).status().await.unwrap().success());
        uninstall_companion(&target, None).await.unwrap();
        let mut unrelated = wsl_command(&target).unwrap();
        assert!(unrelated.args(["--exec", "sh", "-c",
            "set -eu; test \"$(cat \"$HOME/.config/.warpai/wsl/bin/unrelated.txt\")\" = preserve; rm \"$HOME/.config/.warpai/wsl/bin/unrelated.txt\""]).status().await.unwrap().success());
    }
    #[test]
    fn guest_discovery_rejects_unrequested_commands_and_invalid_paths() {
        let commands = vec!["codex".into(), "claude".into()];
        let (account, agents) = parse(
            "guest\ncodex\t/home/guest/My Tools/多语言/codex\n".as_bytes(),
            &commands,
        )
        .unwrap();
        assert_eq!(account, "guest");
        assert_eq!(agents.len(), 1);
        for invalid in [
            "",
            "guest\nunknown\t/bin/unknown\n",
            "guest\ncodex\trelative\n",
            "guest\ncodex\t/bin/codex\tbad\n",
        ] {
            assert!(parse(invalid.as_bytes(), &commands).is_err());
        }
    }
    #[test]
    fn wsl_selection_persists_without_local_preferences() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("wsl.json");
        setup::atomic_write(
            &path,
            br#"{"enabled":true,"distribution":"Custom Linux","user":"guest"}"#,
        )
        .unwrap();
        let mut settings = WslSettings::load(path);
        assert!(settings.preferences.enabled);
        assert_eq!(
            settings.preferences.distribution.as_deref(),
            Some("Custom Linux")
        );
        assert!(settings.agents.is_empty());
        assert!(
            settings.selected_target().is_none(),
            "Saved selection cannot enroll an unlogged guest"
        );
        settings.targets.push(Target {
            distribution: "Custom Linux".into(),
            user: "other".into(),
            home: "/home/other".into(),
        });
        assert!(
            settings.selected_target().is_none(),
            "Another user cannot satisfy the saved account"
        );
        settings.targets.push(Target {
            distribution: "Custom Linux".into(),
            user: "guest".into(),
            home: "/home/guest".into(),
        });
        assert_eq!(settings.selected_target().unwrap().user, "guest");
        settings.targets.clear();
        assert!(settings.preferences.enabled);
        assert_eq!(settings.preferences.user.as_deref(), Some("guest"));
        assert!(settings.selected_target().is_none());
    }

    #[test]
    fn configuration_pins_the_logged_in_account_in_literal_wsl_arguments() {
        let target = Target {
            distribution: "Custom $(literal) 多语言".into(),
            user: "guest".into(),
            home: "/home/guest".into(),
        };
        let command = wsl_command(&target).unwrap();
        let args: Vec<_> = command
            .as_std()
            .get_args()
            .map(|argument| argument.to_str().unwrap())
            .collect();
        assert_eq!(
            args,
            [
                "--distribution",
                "Custom $(literal) 多语言",
                "--user",
                "guest"
            ]
        );
        let mut invalid = target;
        invalid.user = "-root".into();
        assert!(wsl_command(&invalid).is_err());
    }

    #[test]
    fn logged_in_target_requires_confirmed_guest_home_and_excludes_nested_ssh() {
        use crate::terminal::model::session::{
            command_executor::NoOpCommandExecutor, Session, SessionInfo,
        };
        let mut info = SessionInfo::new_for_test()
            .with_user("guest".into())
            .with_home_dir("/home/guest".into());
        let create = |info| Session::new(info, std::sync::Arc::new(NoOpCommandExecutor::new()));
        assert!(Target::from_session(&create(info.clone())).is_none());
        info.wsl_name = Some("Custom Linux".into());
        // Session::home_dir uses a platform-wide test override; validate Linux metadata directly.
        let target = Target::new("Custom Linux", "guest", "/home/guest").unwrap();
        assert_eq!(target.label(), "Custom Linux · guest");
        assert!(Target::new("Custom Linux", "guest", "C:\\Users\\guest").is_none());
        assert!(Target::new("Custom Linux", "", "/home/guest").is_none());
        assert!(
            Target::from_session(&create(info.with_ssh_socket_path("/tmp/nested-ssh".into())))
                .is_none()
        );
    }
}
