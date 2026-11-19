//! System OpenSSH metadata and a clean, strictly verified project control channel.
use std::{path::PathBuf, process::Stdio, time::Duration};

use base64::Engine;
use remote_protocol::{
    managed::{valid_account_fence, valid_fence, PROTOCOL_MAJOR, TERMINAL_SCREEN_SCHEMA,
        TERMINAL_SNAPSHOT_MAX_BYTES, TERMINAL_SNAPSHOT_PAGE_BYTES},
    proto::*,
    protocol::{read_message_with_limit, write_message_with_limit, MAX_MANAGED_MESSAGE_SIZE},
};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RemoteShell {
    Posix,
    PowerShell,
}

/// Target metadata only; authentication and jump routing stay in system SSH config.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SshProfile {
    pub target: String,
    pub config_file: Option<PathBuf>,
    pub remote_root: String,
    pub companion_path: String,
    pub remote_shell: RemoteShell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionError {
    InvalidProfile,
    SshUnavailable,
    SshAuthenticationUnavailable,
    ConnectionLost,
    IncompatibleVersion,
    StaleAttachment,
    DeliveryUnknown,
    SnapshotExpired,
    CompanionUnavailable,
    FeatureUnavailable,
    InvalidInput,
    Conflict,
    CapacityExceeded,
    PermissionDenied,
    NotFound,
}

/// A digest-checked screen checkpoint and its attachment-scoped cut.
pub struct TerminalCheckpoint {
    pub header: TerminalSnapshotPage,
    pub data: Vec<u8>,
}

pub enum TerminalCheckpointTail {
    Output(TerminalState),
    Gap { available_start: u64, available_end: u64 },
}

fn target(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && !value.starts_with('-')
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-:[]".contains(&b))
}

fn text(value: &str, limit: usize) -> bool {
    !value.trim().is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}

/// OpenSSH expands its own account's home, including when invoked inside WSL.
pub(crate) fn valid_control_socket(socket: &std::path::Path, wsl: bool) -> bool {
    let Some(path) = socket.to_str().filter(|path| text(path, 4096)) else {
        return false;
    };
    socket.is_absolute()
        || (wsl && path.starts_with('/'))
        || path.strip_prefix("~/.ssh/").is_some_and(|name| {
            !name.is_empty() && name != "." && name != ".."
                && name.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}

impl SshProfile {
    pub fn validate(&self) -> Result<(), ConnectionError> {
        if !target(&self.target)
            || self
                .config_file
                .as_ref()
                .is_some_and(|p| p.to_str().is_none_or(|v| !text(v, 4096)))
            || !text(&self.remote_root, 4096)
            || !text(&self.companion_path, 4096)
            || match self.remote_shell {
                RemoteShell::Posix => {
                    !self.remote_root.starts_with('/') || !self.companion_path.starts_with('/')
                }
                RemoteShell::PowerShell => {
                    !windows_absolute(&self.remote_root) || !windows_absolute(&self.companion_path)
                }
            }
        {
            return Err(ConnectionError::InvalidProfile);
        }
        Ok(())
    }

    pub(crate) fn ssh_command(&self) -> Result<Command, ConnectionError> {
        self.validate()?;
        let mut command = Command::new("ssh");
        crate::session::without_terminal_binding(&mut command);
        #[cfg(windows)]
        command.creation_flags(windows::Win32::System::Threading::CREATE_NO_WINDOW.0);
        command.env_remove("VIBE_MCP_SERVERS");
        // Keep system authentication agent/config; never forward native terminal capabilities.
        command.args([
            "-o",
            "ForwardAgent=no",
            "-o",
            "ForwardX11=no",
            "-o",
            "PermitLocalCommand=no",
            "-o",
            "ConnectTimeout=10",
            "-o",
            "ServerAliveInterval=15",
            "-o",
            "ServerAliveCountMax=2",
        ]);
        command.args([
            "-T",
            "-o",
            "BatchMode=yes",
            "-o",
            "StrictHostKeyChecking=yes",
        ]);
        if let Some(config) = &self.config_file {
            command.arg("-F").arg(config);
        }
        command.kill_on_drop(true);
        Ok(command)
    }

    /// SSH joins remote argv into a shell program; quote for that separate boundary.
    fn companion_command(&self) -> Result<String, ConnectionError> {
        self.companion_command_args("")
    }

    fn companion_command_args(&self, arguments: &str) -> Result<String, ConnectionError> {
        let path = &self.companion_path;
        Ok(match self.remote_shell {
            RemoteShell::Posix => {
                let quoted = format!("'{}'", path.replace('\'', "'\\''"));
                format!("exec {quoted}{arguments}")
            }
            RemoteShell::PowerShell => {
                let script = format!("& '{}'{arguments}", path.replace('\'', "''"));
                let utf16: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
                format!(
                    "powershell.exe -NoLogo -NoProfile -NonInteractive -EncodedCommand {}",
                    base64::engine::general_purpose::STANDARD.encode(utf16)
                )
            }
        })
    }

    /// Opt-in helper bridge: the installed companion joins the account guardian.
    fn guardian_command(&self, route: &str) -> Result<String, ConnectionError> {
        if !matches!(route, "ssh" | "wsl") {
            return Err(ConnectionError::InvalidProfile);
        }
        // The guardian is a POSIX account service; Windows hosts stay unsupported.
        if self.remote_shell != RemoteShell::Posix {
            return Err(ConnectionError::FeatureUnavailable);
        }
        self.companion_command_args(&format!(" --guardian-stdio {route}"))
    }

    fn arguments_command(&self, arguments: &[String]) -> Result<Command, ConnectionError> {
        if arguments.is_empty()
            || arguments.len() > 128
            || arguments.iter().any(|arg| !text(arg, 4096))
        {
            return Err(ConnectionError::InvalidProfile);
        }
        let mut command = self.ssh_command()?;
        command.args(["-o", "ClearAllForwardings=yes"]);
        command.args(arguments);
        Ok(command)
    }

    fn session_command(
        &self,
        socket: &std::path::Path,
        wsl: Option<&str>,
    ) -> Result<Command, ConnectionError> {
        if !valid_control_socket(socket, wsl.is_some()) {
            return Err(ConnectionError::InvalidProfile);
        }
        let native = self.ssh_command()?;
        let mut command = if let Some(distribution) = wsl {
            if !text(distribution, 256) {
                return Err(ConnectionError::InvalidProfile);
            }
            let mut command = Command::new("wsl");
            crate::session::without_terminal_binding(&mut command);
        #[cfg(windows)]
        command.creation_flags(windows::Win32::System::Threading::CREATE_NO_WINDOW.0);
            command.env_remove("VIBE_MCP_SERVERS");
            command.args(["--distribution", distribution, "--exec", "ssh"]);
            command.args(native.as_std().get_args());
            command.kill_on_drop(true);
            command
        } else {
            native
        };
        // Reuse only the established session. A closed master must never reconnect elsewhere.
        command
            .arg("-S")
            .arg(socket)
            .args(["-o", "ControlMaster=no", "-o", "ProxyCommand=false"]);
        command.arg("--").arg("warpai-session");
        Ok(command)
    }
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for ConnectionError {}

fn windows_absolute(path: &str) -> bool {
    let bytes = path.as_bytes();
    (bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/'))
        || path.starts_with("\\\\")
}

/// Managed channel over either a spawned companion process or a verified Unix socket.
enum HostWriter {
    Child(ChildStdin),
    #[cfg(unix)]
    Unix(tokio::io::WriteHalf<tokio::net::UnixStream>),
}

impl tokio::io::AsyncWrite for HostWriter {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
        buffer: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Self::Child(writer) => std::pin::Pin::new(writer).poll_write(context, buffer),
            #[cfg(unix)]
            Self::Unix(writer) => std::pin::Pin::new(writer).poll_write(context, buffer),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Child(writer) => std::pin::Pin::new(writer).poll_flush(context),
            #[cfg(unix)]
            Self::Unix(writer) => std::pin::Pin::new(writer).poll_flush(context),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Child(writer) => std::pin::Pin::new(writer).poll_shutdown(context),
            #[cfg(unix)]
            Self::Unix(writer) => std::pin::Pin::new(writer).poll_shutdown(context),
        }
    }
}

enum HostReader {
    Child(ChildStdout),
    #[cfg(unix)]
    Unix(tokio::io::ReadHalf<tokio::net::UnixStream>),
}

impl tokio::io::AsyncRead for HostReader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
        buffer: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Child(reader) => std::pin::Pin::new(reader).poll_read(context, buffer),
            #[cfg(unix)]
            Self::Unix(reader) => std::pin::Pin::new(reader).poll_read(context, buffer),
        }
    }
}

/// Mutable access serializes requests. Any timeout/malformed response closes the channel.
pub struct HostClient {
    child: Option<Child>,
    input: Compat<HostWriter>,
    output: Compat<HostReader>,
    fence: Option<ManagedFence>,
    account_fence: Option<AccountFence>,
    current_guarded_session: Option<CurrentGuardedSession>,
    pub canonical_root: String,
    pub account_id: String,
    pub root_identity: String,
    alive: bool,
    capabilities: Vec<String>,
    access_epoch: Option<u64>,
}

        fn requires_handoff(command: &crate::companion::TaskCommand) -> bool {
            use crate::{companion::TaskCommand, Operation};
            match command {
                TaskCommand::Scoped { command, .. } => requires_handoff(command),
                TaskCommand::Operator(Operation::TaskSubmit { handoff, .. }
                    | Operation::TaskProgress { handoff, .. } | Operation::TaskFail { handoff, .. }
                    | Operation::TaskFinishCancel { handoff, .. }) => handoff.is_some(),
                TaskCommand::Operator(Operation::AgentReply { handoff_ref, .. }) => handoff_ref.is_some(),
                TaskCommand::Operator(Operation::ContextGet { handoff_part, .. }) => handoff_part.is_some(),
                TaskCommand::Operator(Operation::TaskHistory { handoff_only, .. }) => *handoff_only,
                TaskCommand::Operator(Operation::ContextPublish { content:crate::context::ContextContent::Handoff { .. }, .. }) => true,
                _ => false,
            }
        }
impl HostClient {
    /// Use the selected guest account without an interactive shell or TCP listener.
    #[cfg(windows)]
    pub async fn connect_wsl(profile: &SshProfile, distribution: &str, user: &str) -> Result<Self, ConnectionError> {
        Self::probe_installed(profile, Self::wsl_command(profile, distribution, user)?).await?;
        let mut command = Self::wsl_command(profile, distribution, user)?;
        command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
        let child = command.spawn().map_err(|_| ConnectionError::CompanionUnavailable)?;
        let mut client = Self::open(child, &profile.remote_root).await?;
        if !client.account_id.starts_with("uid:")
            || !client.capabilities.iter().any(|value| value == "project_file_chunks") {
            client.disconnect();
            return Err(ConnectionError::IncompatibleVersion);
        }
        Ok(client)
    }

    /// Opt-in account guardian inside the selected WSL distribution.
    #[cfg(windows)]
    pub async fn connect_wsl_guarded(
        profile: &SshProfile,
        distribution: &str,
        user: &str,
        route: &str,
    ) -> Result<Self, ConnectionError> {
        if route != "wsl" {
            return Err(ConnectionError::InvalidProfile);
        }
        Self::probe_installed(profile, Self::wsl_command(profile, distribution, user)?).await?;
        let mut command = Self::wsl_guardian_command(profile, distribution, user)?;
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let child = command
            .spawn()
            .map_err(|_| ConnectionError::CompanionUnavailable)?;
        let mut client = Self::open(child, &profile.remote_root).await?;
        if !client.account_id.starts_with("uid:") {
            client.disconnect();
            return Err(ConnectionError::IncompatibleVersion);
        }
        client.require_account_guardian()?;
        Ok(client)
    }

    #[cfg(any(windows, test))]
    fn wsl_guardian_command(
        profile: &SshProfile,
        distribution: &str,
        user: &str,
    ) -> Result<Command, ConnectionError> {
        let mut command = Self::wsl_command(profile, distribution, user)?;
        command.args(["--guardian-stdio", "wsl"]);
        Ok(command)
    }

    #[cfg(any(windows, test))]
    pub(crate) fn wsl_command(profile: &SshProfile, distribution: &str, user: &str) -> Result<Command, ConnectionError> {
        profile.validate()?;
        if profile.remote_shell != RemoteShell::Posix
            || !text(distribution, 256) || distribution.starts_with('-')
            || !text(user, 256) || user.starts_with('-') {
            return Err(ConnectionError::InvalidProfile);
        }
        let mut command = Command::new("wsl.exe");
        crate::session::without_terminal_binding(&mut command);
        #[cfg(windows)]
        command.creation_flags(windows::Win32::System::Threading::CREATE_NO_WINDOW.0);
        command.env_remove("VIBE_MCP_SERVERS");
        command.args(["--distribution", distribution, "--user", user, "--exec", &profile.companion_path]);
        command.kill_on_drop(true);
        Ok(command)
    }

    pub async fn connect(profile: &SshProfile) -> Result<Self, ConnectionError> {
        let mut command = profile.ssh_command()?;
        command
            .arg("--")
            .arg(&profile.target)
            .arg(profile.companion_command()?)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let child = command
            .spawn()
            .map_err(|_| ConnectionError::SshUnavailable)?;
        Self::open(child, &profile.remote_root).await
    }

    /// Opt-in account guardian: the installed helper bridges to retained runs.
    pub async fn connect_guarded(profile: &SshProfile, route: &str) -> Result<Self, ConnectionError> {
        let mut command = profile.ssh_command()?;
        command
            .arg("--")
            .arg(&profile.target)
            .arg(profile.guardian_command(route)?)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let child = command
            .spawn()
            .map_err(|_| ConnectionError::SshUnavailable)?;
        let mut client = Self::open(child, &profile.remote_root).await?;
        client.require_account_guardian()?;
        Ok(client)
    }

    /// Attach to the selected terminal's already authenticated SSH session.
    pub async fn connect_session(
        profile: &SshProfile,
        socket: &std::path::Path,
        wsl: Option<&str>,
    ) -> Result<Self, ConnectionError> {
        Self::probe_installed(profile, profile.session_command(socket, wsl)?).await?;
        Self::open_command(
            profile,
            profile.session_command(socket, wsl)?,
            profile.companion_command()?,
        )
        .await
    }

    /// Opt-in account guardian over the established session.
    pub async fn connect_session_guarded(
        profile: &SshProfile,
        socket: &std::path::Path,
        wsl: Option<&str>,
        route: &str,
    ) -> Result<Self, ConnectionError> {
        Self::probe_installed(profile, profile.session_command(socket, wsl)?).await?;
        let mut client = Self::open_command(
            profile,
            profile.session_command(socket, wsl)?,
            profile.guardian_command(route)?,
        )
        .await?;
        client.require_account_guardian()?;
        Ok(client)
    }

    /// Native Windows uses the user's original system SSH transport arguments.
    pub async fn connect_arguments(
        profile: &SshProfile,
        arguments: &[String],
    ) -> Result<Self, ConnectionError> {
        Self::probe_installed(profile, profile.arguments_command(arguments)?).await?;
        Self::open_command(
            profile,
            profile.arguments_command(arguments)?,
            profile.companion_command()?,
        )
        .await
    }

    /// Opt-in account guardian through native Windows SSH arguments.
    pub async fn connect_arguments_guarded(
        profile: &SshProfile,
        arguments: &[String],
        route: &str,
    ) -> Result<Self, ConnectionError> {
        Self::probe_installed(profile, profile.arguments_command(arguments)?).await?;
        let mut client = Self::open_command(
            profile,
            profile.arguments_command(arguments)?,
            profile.guardian_command(route)?,
        )
        .await?;
        client.require_account_guardian()?;
        Ok(client)
    }

    /// A guarded connection must be served by the account guardian, never a legacy scope.
    fn require_account_guardian(&mut self) -> Result<(), ConnectionError> {
        let valid = self.account_fence.is_some()
            && self
                .capabilities
                .iter()
                .any(|capability| capability == "guarded_remote_sessions_v1")
            && self
                .capabilities
                .iter()
                .any(|capability| capability == "account_catalog");
        if !valid {
            self.close();
            return Err(ConnectionError::IncompatibleVersion);
        }
        Ok(())
    }

    async fn probe_installed(
        profile: &SshProfile,
        mut probe: Command,
    ) -> Result<(), ConnectionError> {
        use tokio::io::AsyncReadExt;
        #[cfg(windows)]
        let version_argument = if probe.as_std().get_program() == std::ffi::OsStr::new("wsl.exe") {
            "--version".to_owned()
        } else {
            profile.companion_command_args(" --version")?
        };
        #[cfg(not(windows))]
        let version_argument = profile.companion_command_args(" --version")?;
        let mut child = probe
            .arg(version_argument)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| ConnectionError::SshUnavailable)?;
        let mut output = child
            .stdout
            .take()
            .ok_or(ConnectionError::CompanionUnavailable)?;
        tokio::time::timeout(std::time::Duration::from_secs(15), async {
            let mut bytes = Vec::new();
            (&mut output)
                .take(257)
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| ConnectionError::CompanionUnavailable)?;
            if bytes.len() > 256 {
                return Err(ConnectionError::IncompatibleVersion);
            }
            let status = child
                .wait()
                .await
                .map_err(|_| ConnectionError::CompanionUnavailable)?;
            if !status.success() {
                return Err(if status.code() == Some(255) {
                    ConnectionError::SshAuthenticationUnavailable
                } else {
                    ConnectionError::CompanionUnavailable
                });
            }
            crate::installation::verify_version(&bytes)
        })
        .await
        .map_err(|_| ConnectionError::CompanionUnavailable)?
    }

    async fn open_command(
        profile: &SshProfile,
        mut command: Command,
        remote_command: String,
    ) -> Result<Self, ConnectionError> {
        let child = command
            .arg(remote_command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| ConnectionError::SshUnavailable)?;
        Self::open(child, &profile.remote_root).await
    }

    /// The companion CLI joins the same private account service as SSH control.
    pub async fn connect_local(root: &str) -> Result<Self, ConnectionError> {
        let executable =
            std::env::current_exe().map_err(|_| ConnectionError::CompanionUnavailable)?;
        Self::connect_companion(&executable, root).await
    }

    /// Connect to an explicitly provisioned companion through the same private protocol.
    pub async fn connect_companion(
        executable: &std::path::Path,
        root: &str,
    ) -> Result<Self, ConnectionError> {
        if !executable.is_absolute() {
            return Err(ConnectionError::InvalidProfile);
        }
        let mut command = Command::new(executable);
        crate::session::without_terminal_binding(&mut command);
        #[cfg(windows)]
        command.creation_flags(windows::Win32::System::Threading::CREATE_NO_WINDOW.0);
        command.env_remove("VIBE_MCP_SERVERS");
        let child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| ConnectionError::CompanionUnavailable)?;
        Self::open(child, root).await
    }

    async fn open(mut child: Child, root: &str) -> Result<Self, ConnectionError> {
        let input = HostWriter::Child(
            child
                .stdin
                .take()
                .ok_or(ConnectionError::ConnectionLost)?,
        )
        .compat_write();
        let output = HostReader::Child(
            child
                .stdout
                .take()
                .ok_or(ConnectionError::ConnectionLost)?,
        )
        .compat();
        Self::handshake(Some(child), input, output, Some(root)).await
    }

    /// Direct account-guardian socket for the local CLI; no process or shell involved.
    #[cfg(unix)]
    pub async fn connect_unix(
        stream: tokio::net::UnixStream,
        root: Option<&str>,
    ) -> Result<Self, ConnectionError> {
        let (reader, writer) = tokio::io::split(stream);
        Self::handshake(
            None,
            HostWriter::Unix(writer).compat_write(),
            HostReader::Unix(reader).compat(),
            root,
        )
        .await
    }

    async fn handshake(
        child: Option<Child>,
        input: Compat<HostWriter>,
        output: Compat<HostReader>,
        root: Option<&str>,
    ) -> Result<Self, ConnectionError> {
        let mut client = Self {
            child,
            input,
            output,
            fence: None,
            account_fence: None,
            current_guarded_session: None,
            canonical_root: String::new(),
            account_id: String::new(),
            root_identity: String::new(),
            alive: true,
            capabilities: Vec::new(),
            access_epoch: None,
        };
        let initialized = client
            .request(managed_request::Operation::Initialize(ManagedInitialize {
                protocol_major: PROTOCOL_MAJOR,
            }))
            .await?;
        let managed_response::Result::Initialized(initialized) = initialized else {
            return Err(ConnectionError::IncompatibleVersion);
        };
        let Some(fence) = initialized.fence else {
            return Err(ConnectionError::IncompatibleVersion);
        };
        if initialized.protocol_major != PROTOCOL_MAJOR
            || !valid_fence(&fence, false)
            || !text(&initialized.account_id, 256)
        {
            return Err(ConnectionError::IncompatibleVersion);
        }
        if initialized
            .account_fence
            .as_ref()
            .is_some_and(|fence| !valid_account_fence(fence))
        {
            return Err(ConnectionError::IncompatibleVersion);
        }
        client.fence = Some(fence.clone());
        if let Some(root) = root {
            client.open_project(root).await?;
        }
        client.account_id = initialized.account_id;
        client.capabilities = initialized.capabilities;
        client.current_guarded_session = initialized.current_guarded_session.filter(|session| {
            initialized.account_fence.is_some()
                && Uuid::parse_str(&session.session_id).is_ok_and(|id| !id.is_nil())
                && Uuid::parse_str(&session.run_id).is_ok_and(|id| !id.is_nil())
        });
        client.account_fence = initialized.account_fence;
        Ok(client)
    }

    pub(crate) async fn open_project(&mut self, root: &str) -> Result<(), ConnectionError> {
        let fence = self.fence.clone().ok_or(ConnectionError::StaleAttachment)?;
        self.open_project_request(
            managed_request::Operation::ProjectOpen(ProjectOpen {
                fence: Some(fence),
                root: root.into(),
            }),
            None,
        ).await
    }

    #[cfg(unix)]
    pub(crate) async fn open_guarded_project(&mut self, row: &AccountCatalogRow) -> Result<(), ConnectionError> {
        let fence = self.fence.clone().ok_or(ConnectionError::StaleAttachment)?;
        self.open_project_request(
            managed_request::Operation::GuardedProjectOpen(GuardedProjectOpen {
                fence: Some(fence),
                session_id: row.session_id.clone(),
                run_id: row.run_id.clone(),
            }),
            Some(&row.project_id),
        ).await
    }

    async fn open_project_request(
        &mut self,
        operation: managed_request::Operation,
        expected_project: Option<&str>,
    ) -> Result<(), ConnectionError> {
        let fence = self.fence.clone().ok_or(ConnectionError::StaleAttachment)?;
        let opened = self.request(operation).await?;
        let managed_response::Result::ProjectOpened(opened) = opened else {
            return Err(ConnectionError::CompanionUnavailable);
        };
        let Some(project_fence) = opened.fence else {
            return Err(ConnectionError::StaleAttachment);
        };
        if !valid_fence(&project_fence, true)
            || project_fence.service_id != fence.service_id
            || project_fence.service_boot_id != fence.service_boot_id
            || project_fence.connection_id != fence.connection_id
            || expected_project.is_some_and(|project| project != project_fence.project_id)
            || !text(&opened.canonical_root, 4096)
            || opened.root_identity.len() != 64
            || !opened.root_identity.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(ConnectionError::StaleAttachment);
        }
        self.fence = Some(project_fence);
        self.canonical_root = opened.canonical_root;
        self.root_identity = opened.root_identity;
        Ok(())
    }

    pub(crate) fn restrict_ssh_access(&mut self, epoch: Option<u64>) -> Result<(), ConnectionError> {
        if let Err(error) = crate::ssh_files::check_ssh_access(epoch) { self.close(); return Err(error); }
        self.access_epoch = epoch;
        Ok(())
    }

    pub fn fence(&self) -> Option<&ManagedFence> {
        self.fence.as_ref()
    }

    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    fn terminal_scope(&self, fence: &Option<ManagedFence>) -> Result<(), ConnectionError> {
        if !self.capabilities.iter().any(|c| c == "managed_agent") {
            return Err(ConnectionError::FeatureUnavailable);
        }
        if !self.alive || fence.is_none() || fence != &self.fence {
            return Err(ConnectionError::StaleAttachment);
        }
        Ok(())
    }

    /// Domain failure metadata is returned as `error`; lost writes retain the original intent.
    pub(crate) async fn guest_mcp_bind(&mut self, pid: u32, program: String) -> Result<GuestMcpBound, ConnectionError> {
        match self.request(managed_request::Operation::GuestMcpBind(GuestMcpBind {
            fence: self.fence.clone(), agent_pid: pid, program,
        })).await? {
            managed_response::Result::GuestMcpBound(binding) if binding.fence == self.fence => Ok(binding),
            _ => Err(ConnectionError::StaleAttachment),
        }
    }

    /// Domain failure metadata is returned as `error`; lost writes retain the original intent.
    pub async fn project_tasks(
        &mut self,
        command: &crate::companion::TaskCommand,
        query_generation: u64,
    ) -> Result<serde_json::Value, ConnectionError> {
        if !self.capabilities.iter().any(|c| c == "project_tasks") {
            return Err(ConnectionError::FeatureUnavailable);
        }
        fn requires_context(command: &crate::companion::TaskCommand) -> bool {
            use crate::companion::TaskCommand;
            match command {
                TaskCommand::Scoped { command, .. } => requires_context(command),
                TaskCommand::Operator(operation) => match operation {
                    crate::Operation::ContextPublish { .. } | crate::Operation::ContextGet { .. } | crate::Operation::ContextList { .. } => true,
                    crate::Operation::AgentSend { content, .. } => content.is_some(),
                    crate::Operation::TaskAssign { context_refs, .. } | crate::Operation::TaskCreatePool { context_refs, .. } => !context_refs.is_empty(),
                    _ => false,
                },
                _ => false,
            }
        }
        if requires_context(command) && (!self.capabilities.iter().any(|c| c == "context_resources")
            || !self.capabilities.iter().any(|c| c == "structured_task_messages")) {
            return Err(ConnectionError::FeatureUnavailable);
        }
        if requires_handoff(command) && !self.capabilities.iter().any(|c| c == "handoff_briefs_v1") {
            return Err(ConnectionError::FeatureUnavailable);
        }
        fn required_project_feature(command: &crate::companion::TaskCommand) -> Option<&'static str> {
            use crate::{companion::TaskCommand, Operation};
            match command {
                TaskCommand::Scoped { command, .. } => required_project_feature(command),
                TaskCommand::SshSetup(_) => Some("native_mcp_setup"),
                TaskCommand::Operator(Operation::ExecutionThreadGet { .. }) => Some("execution_threads"),
                TaskCommand::Operator(Operation::ProjectBootstrap { .. } | Operation::ProjectMemoryUpdate { .. }) => Some("project_memory"),
                TaskCommand::Operator(Operation::SupervisionIndex { .. }) => Some("project_supervision"),
                TaskCommand::Operator(Operation::TaskFollowupReconcile { .. }) => Some("followup_reconcile"),
                _ => None,
            }
        }
        if required_project_feature(command).is_some_and(|feature| !self.capabilities.iter().any(|value| value == feature)) {
            return Err(ConnectionError::FeatureUnavailable);
        }
        if !self.alive || self.fence.as_ref().is_none_or(|f| f.project_id.is_empty()) {
            return Err(ConnectionError::StaleAttachment);
        }
        let command_json =
            serde_json::to_vec(command).map_err(|_| ConnectionError::InvalidInput)?;
        if command_json.len() > 64 * 1024 {
            return Err(ConnectionError::CapacityExceeded);
        }
        let result = self
            .request(managed_request::Operation::ProjectTasks(
                ProjectTasksRequest {
                    fence: self.fence.clone(),
                    query_generation,
                    command_json,
                },
            ))
            .await?;
        if let managed_response::Result::ProjectTasks(result) = result {
            if result.fence == self.fence && result.query_generation == query_generation {
                if let Some(value) = crate::companion::decode_task_result(&result.result_json) {
                    let panel_matches = !matches!(command, crate::companion::TaskCommand::Panel(_))
                        || value.get("error").is_some()
                        || value["value"]["project"].as_str()
                            == self.fence.as_ref().map(|f| f.project_id.as_str());
                    if panel_matches {
                        return Ok(value);
                    }
                }
            }
        }
        self.close();
        Err(ConnectionError::StaleAttachment)
    }

    /// File control includes bounded WSL chunks and never admits legacy command execution.
    pub async fn project_files(
        &mut self,
        mut query: ProjectFilesRequest,
    ) -> Result<ProjectFilesResult, ConnectionError> {
        if !self.capabilities.iter().any(|c| c == "project_files") {
            return Err(ConnectionError::FeatureUnavailable);
        }
        let action = ProjectFileAction::try_from(query.action).map_err(|_| ConnectionError::InvalidInput)?;
        let capability = match action {
            ProjectFileAction::ProjectGitStatus | ProjectFileAction::ProjectGitDiff
            | ProjectFileAction::ProjectGitPrepareBase | ProjectFileAction::ProjectGitRoot
            | ProjectFileAction::ProjectGitBranches => Some("project_git_review"),
            ProjectFileAction::ProjectFileReadChunk | ProjectFileAction::ProjectFileWriteChunk => Some("project_file_chunks"),
            _ => None,
        };
        if capability.is_some_and(|required| !self.capabilities.iter().any(|c| c == required)) {
            return Err(ConnectionError::FeatureUnavailable);
        }
        query.fence = self.fence.clone();
        let generation = query.query_generation;
        let reply = self
            .request(managed_request::Operation::ProjectFiles(query))
            .await?;
        if let managed_response::Result::ProjectFiles(reply) = reply {
            if reply.fence == self.fence
                && reply.query_generation == generation
                && reply
                    .snapshot
                    .as_ref()
                    .is_none_or(|snapshot| snapshot.repo_path == self.canonical_root)
            {
                return Ok(*reply);
            }
        }
        self.close();
        Err(ConnectionError::StaleAttachment)
    }

    pub async fn terminal_launch(
        &mut self,
        request: TerminalLaunch,
    ) -> Result<TerminalState, ConnectionError> {
        if request.working_directory.is_some()
            && !self.capabilities.iter().any(|cap| cap == "managed_launch_cwd") {
            return Err(ConnectionError::FeatureUnavailable);
        }
        if request.agent_program.is_some()
            && !self.capabilities.iter().any(|cap| cap == "project_mcp")
        {
            return Err(ConnectionError::FeatureUnavailable);
        }
        if (request.guarded || request.label.is_some() || request.session_kind.is_some())
            && !self
                .capabilities
                .iter()
                .any(|cap| cap == "guarded_remote_sessions_v1")
        {
            return Err(ConnectionError::FeatureUnavailable);
        }
        self.terminal_scope(&request.fence)?;
        let session = request.session_id.clone();
        let intent_guarded = request.guarded;
        let intent_label = request.label.clone();
        let result = self
            .request(managed_request::Operation::TerminalLaunch(request))
            .await?;
        if let managed_response::Result::TerminalState(state) = result {
            if !self.valid_terminal(&state) || state.session_id != session {
                self.close();
                return Err(ConnectionError::StaleAttachment);
            }
            let policy_matches = if intent_guarded {
                state.guarded == Some(true)
                    && !state.attachment_id.is_empty()
                    && state.attachment_generation > 0
            } else {
                state.guarded != Some(true)
                    && state.attachment_id.is_empty()
                    && state.attachment_generation == 0
            };
            if policy_matches && state.label == intent_label {
                return Ok(state);
            }
        }
        self.close();
        Err(ConnectionError::IncompatibleVersion)
    }

    pub async fn terminal_control(
        &mut self,
        request: TerminalControl,
    ) -> Result<TerminalState, ConnectionError> {
        let guarded = request.takeover
            || request.attachment_generation != 0
            || !request.attachment_id.is_empty()
            || matches!(
                TerminalAction::try_from(request.action),
                Ok(TerminalAction::TerminalAttach | TerminalAction::TerminalDetach)
            );
        if guarded
            && !self
                .capabilities
                .iter()
                .any(|cap| cap == "guarded_remote_sessions_v1")
        {
            return Err(ConnectionError::FeatureUnavailable);
        }
        self.terminal_scope(&request.fence)?;
        let session = request.session_id.clone();
        let run = request.run_id.clone();
        let action = TerminalAction::try_from(request.action).ok();
        let input_sequence = request.input_sequence;
        let expected_attachment = (request.attachment_id.clone(), request.attachment_generation);
        let result = self
            .request(managed_request::Operation::TerminalControl(request))
            .await?;
        if let managed_response::Result::TerminalState(state) = result {
            if !self.valid_terminal(&state) || (guarded && state.guarded != Some(true)) {
                self.close();
                return Err(ConnectionError::StaleAttachment);
            }
            // A stale session or attachment belongs to one terminal; keep the
            // multiplexed connection alive for its other sessions.
            if state.session_id != session || state.run_id != run {
                return Err(ConnectionError::StaleAttachment);
            }
            if guarded
                && action != Some(TerminalAction::TerminalAttach)
                && (state.attachment_id != expected_attachment.0
                    || state.attachment_generation != expected_attachment.1)
            {
                return Err(ConnectionError::StaleAttachment);
            }
            if guarded
                && action == Some(TerminalAction::TerminalInput)
                && !state
                    .input_receipt
                    .as_ref()
                    .is_some_and(|receipt| receipt.input_sequence == input_sequence)
            {
                return Err(ConnectionError::DeliveryUnknown);
            }
            return Ok(state);
        }
        self.close();
        Err(ConnectionError::StaleAttachment)
    }

    /// Read an immutable screen cut. Page headers and byte counts are checked before assembly.
    pub async fn terminal_checkpoint(&mut self, state: &TerminalState) -> Result<TerminalCheckpoint, ConnectionError> {
        if state.guarded != Some(true)
            || !self.capabilities.iter().any(|capability| capability == "guarded_screen_checkpoint_v1")
        {
            return Err(ConnectionError::FeatureUnavailable);
        }
        if !self.valid_terminal(state) {
            return Err(ConnectionError::StaleAttachment);
        }
        let mut request = TerminalSnapshotRequest {
            fence: self.fence.clone(), session_id: state.session_id.clone(), run_id: state.run_id.clone(),
            attachment_id: state.attachment_id.clone(), attachment_generation: state.attachment_generation,
            schema: TERMINAL_SCREEN_SCHEMA, snapshot_id: String::new(), page: 0,
        };
        let mut header = None;
        let mut data = Vec::new();
        loop {
            let response = self.request(managed_request::Operation::TerminalSnapshot(request.clone())).await?;
            let managed_response::Result::TerminalSnapshot(mut page) = response else {
                return Err(ConnectionError::IncompatibleVersion);
            };
            let total = page.total_bytes as usize;
            let start = (request.page as usize)
                .checked_mul(TERMINAL_SNAPSHOT_PAGE_BYTES)
                .filter(|offset| *offset < total).ok_or(ConnectionError::IncompatibleVersion)?;
            if page.fence != request.fence || page.session_id != request.session_id
                || page.run_id != request.run_id || page.attachment_id != request.attachment_id
                || page.attachment_generation != request.attachment_generation
            {
                return Err(ConnectionError::StaleAttachment);
            }
            if page.schema != request.schema || page.page != request.page
                || !Uuid::parse_str(&page.snapshot_id).is_ok_and(|id| !id.is_nil())
                || !(1..=TERMINAL_SNAPSHOT_MAX_BYTES).contains(&total)
                || page.sha256.len() != 32
                || !(1..=1000).contains(&page.rows) || !(1..=1000).contains(&page.columns)
                || data.len() != start
                || page.data.len() != TERMINAL_SNAPSHOT_PAGE_BYTES.min(total - start)
            {
                return Err(ConnectionError::IncompatibleVersion);
            }
            let body = std::mem::take(&mut page.data);
            page.page = 0;
            if let Some(expected) = &header {
                if expected != &page { return Err(ConnectionError::IncompatibleVersion); }
            } else {
                data.reserve_exact(total);
                request.snapshot_id = page.snapshot_id.clone();
                header = Some(page);
            }
            data.extend(body);
            if data.len() == total { break; }
            request.page += 1;
        }
        let header = header.ok_or(ConnectionError::IncompatibleVersion)?;
        use sha2::{Digest, Sha256};
        if Sha256::digest(&data)[..] != header.sha256 {
            return Err(ConnectionError::IncompatibleVersion);
        }
        let screen = crate::screen_detection::native::NativeScreen::from_checkpoint(&data)
            .ok_or(ConnectionError::IncompatibleVersion)?;
        if screen.size() != (header.rows as u16, header.columns as u16)
            || screen.history_truncated() != header.history_truncated
        {
            return Err(ConnectionError::IncompatibleVersion);
        }
        Ok(TerminalCheckpoint { header, data })
    }

    /// Read a contiguous post-checkpoint tail or return the actual available range.
    pub async fn terminal_checkpoint_tail(&mut self, checkpoint: &TerminalCheckpoint, offset: u64) -> Result<TerminalCheckpointTail, ConnectionError> {
        let header = &checkpoint.header;
        if offset < header.output_cut || header.fence != self.fence {
            return Err(ConnectionError::InvalidInput);
        }
        let state = self.terminal_control(TerminalControl {
            fence: header.fence.clone(), session_id: header.session_id.clone(), run_id: header.run_id.clone(),
            attachment_id: header.attachment_id.clone(), attachment_generation: header.attachment_generation,
            action: TerminalAction::TerminalRead as i32, output_offset: offset, ..Default::default()
        }).await?;
        if state.accepted_resize_sequence != header.resize_sequence {
            return Err(ConnectionError::SnapshotExpired);
        }
        if state.output_truncated {
            return Ok(TerminalCheckpointTail::Gap { available_start: state.output_offset, available_end: state.output_end });
        }
        if state.output_offset != offset { return Err(ConnectionError::InvalidInput); }
        Ok(TerminalCheckpointTail::Output(state))
    }

    pub(crate) fn current_guarded_session(&self) -> Option<&CurrentGuardedSession> {
        self.current_guarded_session.as_ref()
    }

    pub fn account_fence(&self) -> Option<&AccountFence> {
        self.account_fence.as_ref()
    }

    /// Only the account guardian serves the shared catalog and retained runs.
    pub fn is_account_guardian(&self) -> bool {
        self.account_fence.is_some()
            && self
                .capabilities
                .iter()
                .any(|capability| capability == "guarded_remote_sessions_v1")
            && self
                .capabilities
                .iter()
                .any(|capability| capability == "account_catalog")
    }

    fn account_scope(&self) -> Result<AccountFence, ConnectionError> {
        if !self.alive || !self.is_account_guardian() {
            return Err(ConnectionError::FeatureUnavailable);
        }
        self.account_fence
            .clone()
            .ok_or(ConnectionError::FeatureUnavailable)
    }

    pub async fn account_catalog_list(
        &mut self,
        filter: AccountCatalogFilter,
        limit: u32,
        cursor: String,
    ) -> Result<AccountCatalogPage, ConnectionError> {
        let fence = self.account_scope()?;
        let result = self
            .request(managed_request::Operation::AccountCatalogList(
                AccountCatalogList {
                    fence: Some(fence),
                    filter: Some(filter),
                    limit,
                    cursor,
                },
            ))
            .await?;
        match result {
            managed_response::Result::AccountCatalogPage(page) => Ok(page),
            _ => {
                self.close();
                Err(ConnectionError::IncompatibleVersion)
            }
        }
    }

    pub async fn account_catalog_inspect(
        &mut self,
        session_id: String,
        run_id: String,
    ) -> Result<AccountCatalogRow, ConnectionError> {
        let fence = self.account_scope()?;
        let result = self
            .request(managed_request::Operation::AccountCatalogInspect(
                AccountCatalogInspect {
                    fence: Some(fence),
                    session_id,
                    run_id,
                },
            ))
            .await?;
        match result {
            managed_response::Result::AccountCatalogRow(row) => Ok(row),
            _ => {
                self.close();
                Err(ConnectionError::IncompatibleVersion)
            }
        }
    }

    /// Replay is bounded; a `gap` requires a fresh listing before trusting events.
    pub async fn account_catalog_watch(
        &mut self,
        filter: Option<AccountCatalogFilter>,
        cursor: String,
        limit: u32,
    ) -> Result<AccountCatalogEvents, ConnectionError> {
        let fence = self.account_scope()?;
        let result = self
            .request(managed_request::Operation::AccountCatalogWatch(
                AccountCatalogWatch {
                    fence: Some(fence),
                    filter,
                    cursor,
                    limit,
                },
            ))
            .await?;
        match result {
            managed_response::Result::AccountCatalogEvents(events) => Ok(events),
            _ => {
                self.close();
                Err(ConnectionError::IncompatibleVersion)
            }
        }
    }

    /// Receipts report the stop request; task acceptance and review stay separate.
    pub async fn account_stop(
        &mut self,
        target: Option<AccountStopTarget>,
        project_id: String,
    ) -> Result<AccountStopReceipts, ConnectionError> {
        let fence = self.account_scope()?;
        let result = self
            .request(managed_request::Operation::AccountStop(AccountStop {
                fence: Some(fence),
                target,
                project_id,
            }))
            .await?;
        match result {
            managed_response::Result::AccountStopReceipts(receipts) => Ok(receipts),
            _ => {
                self.close();
                Err(ConnectionError::IncompatibleVersion)
            }
        }
    }

    fn valid_terminal(&self, state: &TerminalState) -> bool {
        remote_protocol::managed::valid_terminal_state(state) && state.fence == self.fence
    }

    /// Disconnect stops this connection's owned Agent runs and revokes their MCP bindings.
    pub fn disconnect(&mut self) {
        self.close();
    }

    async fn request(
        &mut self,
        operation: managed_request::Operation,
    ) -> Result<managed_response::Result, ConnectionError> {
        let timeout = crate::ssh_setup::control_timeout(&operation);
        if let Err(error) = crate::ssh_files::check_ssh_access(self.access_epoch) { self.close(); return Err(error); }
        if !self.alive {
            return Err(ConnectionError::ConnectionLost);
        }
        let request_id = Uuid::new_v4().to_string();
        let message = ClientMessage {
            request_id: request_id.clone(),
            message: Some(client_message::Message::Managed(ManagedRequest {
                operation: Some(operation),
            })),
        };
        let response = tokio::time::timeout(timeout, async {
            write_message_with_limit(&mut self.input, &message, MAX_MANAGED_MESSAGE_SIZE).await?;
            read_message_with_limit::<ServerMessage>(&mut self.output, MAX_MANAGED_MESSAGE_SIZE)
                .await
        })
        .await;
        if let Err(error) = crate::ssh_files::check_ssh_access(self.access_epoch) { self.close(); return Err(error); }
        let Ok(Ok(response)) = response else {
            self.close();
            return Err(ConnectionError::ConnectionLost);
        };
        if response.request_id != request_id {
            self.close();
            return Err(ConnectionError::StaleAttachment);
        }
        let Some(server_message::Message::Managed(reply)) = response.message else {
            self.close();
            return Err(ConnectionError::IncompatibleVersion);
        };
        let Some(result) = reply.result else {
            self.close();
            return Err(ConnectionError::IncompatibleVersion);
        };
        match result {
            managed_response::Result::Error(error) => {
                let error = match ManagedErrorCode::try_from(error.code) {
                    Ok(ManagedErrorCode::ManagedIncompatibleVersion) => {
                        ConnectionError::IncompatibleVersion
                    }
                    Ok(ManagedErrorCode::ManagedStaleAttachment) => {
                        ConnectionError::StaleAttachment
                    }
                    Ok(ManagedErrorCode::ManagedFeatureUnavailable) => {
                        ConnectionError::FeatureUnavailable
                    }
                    Ok(ManagedErrorCode::ManagedInvalidInput) => ConnectionError::InvalidInput,
                    Ok(ManagedErrorCode::ManagedPermissionDenied) => {
                        ConnectionError::PermissionDenied
                    }
                    Ok(ManagedErrorCode::ManagedNotFound) => ConnectionError::NotFound,
                    Ok(ManagedErrorCode::ManagedConflict)
                    | Ok(ManagedErrorCode::ManagedInputConflict) => ConnectionError::Conflict,
                    Ok(ManagedErrorCode::ManagedDeliveryUnknown) => {
                        ConnectionError::DeliveryUnknown
                    }
                    Ok(ManagedErrorCode::ManagedSnapshotExpired) => ConnectionError::SnapshotExpired,
                    Ok(ManagedErrorCode::ManagedCapacityExceeded) => {
                        ConnectionError::CapacityExceeded
                    }
                    _ => ConnectionError::CompanionUnavailable,
                };
                Err(error)
            }
            result => Ok(result),
        }
    }

    fn close(&mut self) {
        self.alive = false;
        self.fence = None;
        if let Some(child) = &mut self.child {
            let _ = child.start_kill();
        }
    }
}

#[cfg(all(test, windows))]
#[path = "ssh_remote_windows_tests.rs"]
mod windows_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handoff_capability_gate_covers_scoped_requests_and_preserves_legacy_requests() {
        use crate::{companion::TaskCommand, handoff::{HandoffPart, HandoffSubmission}, Operation};
        let mut operation: Operation = serde_json::from_value(serde_json::json!({
            "op":"task_progress", "task_id":"fixture", "revision":1,"note":"Checkpoint", "request_id":"fixture"
        })).unwrap();
        assert!(!requires_handoff(&TaskCommand::Operator(operation.clone())));
        assert!(requires_handoff(&TaskCommand::Operator(Operation::TaskHistory {
            task_id:"fixture".into(), handoff_only:true, cursor:None, limit:None,
        })));

        if let Operation::TaskProgress { handoff, .. } = &mut operation { *handoff = Some(HandoffSubmission::default()); }
        assert!(requires_handoff(&TaskCommand::Scoped { scope:"fixture".into(), command:Box::new(TaskCommand::Operator(operation)) }));
        let reference = crate::context::ContextRef { id:"fixture".into(), revision:1, sha256:"fixture".into() };
        assert!(!requires_handoff(&TaskCommand::Operator(Operation::ContextGet { reference:reference.clone(), handoff_part:None })));
        assert!(requires_handoff(&TaskCommand::Operator(Operation::ContextGet { reference, handoff_part:Some(HandoffPart::Status) })));
    }

    #[test]
    fn control_socket_accepts_native_home_without_reinterpreting_guest_paths() {
        let profile = SshProfile { target: "node".into(), config_file: None,
            remote_root: "/home/guest/project".into(),
            companion_path: "/home/guest/.config/warpai/bin/warpai-companion".into(),
            remote_shell: RemoteShell::Posix };
        let path = std::path::Path::new("~/.ssh/179237952528983");
        for wsl in [None, Some("Ubuntu Test")] {
            let command = profile.session_command(path, wsl).unwrap();
            let args: Vec<_> = command.as_std().get_args().filter_map(|v| v.to_str()).collect();
            assert!(args.windows(2).any(|pair| pair == ["-S", "~/.ssh/179237952528983"]));
            assert!(args.contains(&"ControlMaster=no"));
            assert!(args.contains(&"ProxyCommand=false"));
        }
        for path in ["", "relative/socket", "~/.ssh/", "~/.ssh/.", "~/.ssh/..",
            "~/.ssh/../socket", "~/.ssh/socket/child", "~other/.ssh/socket",
            "~/socket", "~/.ssh/%h", "~/.ssh/socket\n"] {
            assert!(!valid_control_socket(std::path::Path::new(path), false), "{path:?}");
        }
    }

    #[test]
    fn wsl_transport_uses_explicit_guest_and_literal_native_arguments() {
        let profile = SshProfile { target: "wsl".into(), config_file: None,
            remote_root: "/home/user/project with spaces".into(),
            companion_path: "/home/user/it\'s $(literal)/companion".into(), remote_shell: RemoteShell::Posix };
        let command = HostClient::wsl_command(&profile, "Ubuntu Test", "guest").unwrap();
        let args: Vec<_> = command.as_std().get_args().map(|value| value.to_str().unwrap()).collect();
        assert_eq!(args, ["--distribution", "Ubuntu Test", "--user", "guest", "--exec", &profile.companion_path]);
        let guarded = HostClient::wsl_guardian_command(&profile, "Ubuntu Test", "guest").unwrap();
        let guarded_args: Vec<_> = guarded.as_std().get_args().map(|value| value.to_str().unwrap()).collect();
        assert_eq!(&guarded_args[..args.len()], args);
        assert_eq!(&guarded_args[args.len()..], ["--guardian-stdio", "wsl"]);
        for (distribution, user) in [("", "guest"), ("--help", "guest"), ("Ubuntu\nother", "guest"), ("Ubuntu", ""), ("Ubuntu", "--root")] {
            assert!(HostClient::wsl_command(&profile, distribution, user).is_err());
        }
    }


    #[test]
    fn ssh_profile_rejects_options_and_encodes_remote_shell_separately() {
        let mut profile = SshProfile {
            target: "research-node".into(),
            config_file: None,
            remote_root: "/srv/project with spaces/多语言".into(),
            companion_path: "/opt/it's $(danger)/companion".into(),
            remote_shell: RemoteShell::Posix,
        };
        assert!(profile.validate().is_ok());
        assert_eq!(
            profile.companion_command().unwrap(),
            "exec '/opt/it'\\''s $(danger)/companion'"
        );
        let command = profile.ssh_command().unwrap();
        let args: Vec<_> = command
            .as_std()
            .get_args()
            .filter_map(|arg| arg.to_str())
            .collect();
        assert!(args.contains(&"StrictHostKeyChecking=yes"));
        assert!(args.contains(&"BatchMode=yes"));
        assert!(!args.iter().any(|arg| arg.starts_with("ControlPath=")));
        for bad in [
            "-oProxyCommand=bad",
            "user@host",
            "host;touch",
            "host\nother",
            "host other",
            "$(bad)",
        ] {
            profile.target = bad.into();
            assert_eq!(profile.validate(), Err(ConnectionError::InvalidProfile));
        }
        profile.target = "research-node".into();
        profile.remote_shell = RemoteShell::PowerShell;
        profile.remote_root = "C:\\project with spaces".into();
        profile.companion_path = "C:\\it's $(danger)\\companion.exe".into();
        assert!(profile.validate().is_ok());
        let command = profile.companion_command().unwrap();
        let encoded = command.split_whitespace().last().unwrap();
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        let words: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        assert_eq!(
            String::from_utf16(&words).unwrap(),
            "& 'C:\\it''s $(danger)\\companion.exe'"
        );
    }
}
