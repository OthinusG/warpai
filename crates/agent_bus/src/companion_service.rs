//! Native single-owner project service with connection-owned Agent runs.
use super::*;
use std::{
    process::Stdio,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;

#[cfg(unix)]
type Stream = tokio::net::UnixStream;
#[cfg(windows)]
type Stream = tokio::net::windows::named_pipe::NamedPipeClient;

fn endpoint(directory: &Path) -> std::io::Result<String> {
    let identity = identity::Identity::open(directory)?;
    #[cfg(unix)]
    {
        let runtime =
            PathBuf::from("/tmp").join(format!("warpai-companion-{}", identity.service_id));
        identity::private_directory(&runtime)?;
        Ok(runtime.join("control.sock").to_string_lossy().into_owned())
    }
    #[cfg(windows)]
    {
        Ok(format!(
            r"\\.\pipe\warpai-companion-{}",
            identity.service_id
        ))
    }
}

async fn connect(endpoint: &str) -> std::io::Result<Stream> {
    #[cfg(unix)]
    {
        Stream::connect(endpoint).await
    }
    #[cfg(windows)]
    {
        tokio::net::windows::named_pipe::ClientOptions::new().open(endpoint)
    }
}

fn start_service() -> std::io::Result<()> {
    let mut command = tokio::process::Command::new(std::env::current_exe()?);
    command
        .arg("--service")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    crate::session::without_terminal_binding(&mut command);
    command.env_remove("VIBE_MCP_SERVERS");
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Only async-signal-safe setsid runs after fork; no allocator or Rust lock.
        unsafe {
            command.as_std_mut().pre_exec(|| {
                if libc::setsid() < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.as_std_mut().creation_flags(0x00000008 | 0x00000200); // DETACHED_PROCESS | NEW_PROCESS_GROUP
    }
    let mut child = command.spawn()?;
    // Reap the child if this proxy remains alive through the idle service exit.
    tokio::spawn(async move {
        let _ = child.wait().await;
    });
    Ok(())
}

pub(super) async fn proxy_stdio() -> Result<(), ProtocolError> {
    let directory = data_directory()?;
    let endpoint = endpoint(&directory)?;
    let (stream, expected_scope) = match connect(&endpoint).await {
        Ok(stream) => (stream, None),
        Err(_) => {
            #[cfg(unix)]
            let scope = identity::Identity::open(&directory)?.service_id;
            #[cfg(unix)]
            if let Some(stream) = ordinary_guardian(&scope).await {
                return forward_stdio(stream, Some(scope), None).await;
            }
            start_service()?;
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    if let Ok(stream) = connect(&endpoint).await {
                        break (stream, None);
                    }
                    #[cfg(unix)]
                    if let Some(stream) = ordinary_guardian(&scope).await {
                        break (stream, Some(scope.clone()));
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            })
            .await
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::TimedOut))?
        }
    };
    forward_stdio(stream, expected_scope, None).await
}

#[cfg(unix)]
async fn ordinary_guardian(scope: &str) -> Option<Stream> {
    let mut candidate = guardian::probe_connection_owned().await.ok()??.compat();
    let request_id = Uuid::new_v4().to_string();
    let request = ClientMessage {
        request_id: request_id.clone(),
        message: Some(client_message::Message::Managed(ManagedRequest {
            operation: Some(managed_request::Operation::Initialize(ManagedInitialize {
                protocol_major: PROTOCOL_MAJOR,
            })),
        })),
    };
    let reply = tokio::time::timeout(Duration::from_secs(2), async {
        write_message_with_limit(&mut candidate, &request, MAX_MANAGED_MESSAGE_SIZE).await?;
        read_message_with_limit::<ServerMessage>(&mut candidate, MAX_MANAGED_MESSAGE_SIZE).await
    }).await.ok()?.ok()?;
    drop(candidate);
    if reply.request_id != request_id || !verified_initialize(&reply, scope, None) {
        // A different logical Store is independent of this legacy launch.
        return None;
    }
    guardian::probe_connection_owned().await.ok().flatten()
}

pub(super) async fn forward_stdio(
    stream: Stream,
    mut expected_scope: Option<String>,
    expected_guardian: Option<(Uuid, Uuid)>,
) -> Result<(), ProtocolError> {
    let (input, output) = tokio::io::split(stream);
    let mut input = input.compat();
    let mut output = output.compat_write();
    let mut stdin = tokio::io::stdin().compat();
    let mut stdout = tokio::io::stdout().compat_write();
    loop {
        let request: ClientMessage =
            match read_message_with_limit(&mut stdin, MAX_MANAGED_MESSAGE_SIZE).await {
                Ok(request) => request,
                Err(ProtocolError::UnexpectedEof) => return Ok(()),
                Err(error) => return Err(error),
            };
        let timeout = match &request.message {
            Some(client_message::Message::Managed(request)) => request.operation.as_ref()
                .map(crate::ssh_setup::control_timeout).unwrap_or(Duration::from_secs(10)),
            _ => Duration::from_secs(10),
        };
        let reply: ServerMessage = tokio::time::timeout(timeout, async {
            write_message_with_limit(&mut output, &request, MAX_MANAGED_MESSAGE_SIZE).await?;
            read_message_with_limit(&mut input, MAX_MANAGED_MESSAGE_SIZE).await
        })
        .await
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::TimedOut))??;
        if let Some(expected) = expected_scope.take() {
            if !verified_initialize(&reply, &expected, expected_guardian) {
                return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied).into());
            }
        }
        write_message_with_limit(&mut stdout, &reply, MAX_MANAGED_MESSAGE_SIZE).await?;
    }
}

fn verified_initialize(
    reply: &ServerMessage,
    service_id: &str,
    expected_guardian: Option<(Uuid, Uuid)>,
) -> bool {
    let Some(server_message::Message::Managed(response)) = &reply.message else {
        return false;
    };
    let Some(managed_response::Result::Initialized(initialized)) = &response.result else {
        return false;
    };
    let Some(fence) = &initialized.fence else {
        return false;
    };
    if fence.service_id != service_id {
        return false;
    }
    match (expected_guardian, &initialized.account_fence) {
        (None, None) => true,
        (Some((guardian, boot)), Some(account)) => {
            account.guardian_id == guardian.to_string()
                && account.boot_id == boot.to_string()
                && account.boot_id == fence.service_boot_id
                && account.connection_id == fence.connection_id
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guardian_stdio_requires_exact_scope_and_socket_owner() {
        let guardian = Uuid::new_v4();
        let boot = Uuid::new_v4();
        let scope = Uuid::new_v4().to_string();
        let connection = Uuid::new_v4().to_string();
        let managed = ManagedInitialized {
            fence: Some(ManagedFence {
                service_id: scope.clone(),
                service_boot_id: boot.to_string(),
                connection_id: connection.clone(),
                ..Default::default()
            }),
            account_fence: Some(AccountFence {
                guardian_id: guardian.to_string(),
                boot_id: boot.to_string(),
                connection_id: connection,
            }),
            ..Default::default()
        };
        let reply = ServerMessage {
            message: Some(server_message::Message::Managed(Box::new(ManagedResponse {
                result: Some(managed_response::Result::Initialized(managed)),
            }))),
            ..Default::default()
        };
        assert!(verified_initialize(&reply, &scope, Some((guardian, boot))));
        assert!(!verified_initialize(&reply, &scope, Some((guardian, Uuid::new_v4()))));
        assert!(!verified_initialize(&reply, &scope, None));
        assert!(!verified_initialize(&reply, &Uuid::new_v4().to_string(), Some((guardian, boot))));
    }
}

pub(super) async fn serve() -> Result<(), ProtocolError> {
    let directory = data_directory()?;
    identity::private_directory(&directory)?;
    let lock = identity::private_file(&directory.join("service.lock"))?;
    lock.try_lock().map_err(std::io::Error::other)?;
    files::clean_abandoned(&directory)?;
    let endpoint = endpoint(&directory)?;
    let boot = Uuid::new_v4().to_string();
    #[cfg(unix)]
    let listener = {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::remove_file(&endpoint) {
            Ok(()) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
        let listener = tokio::net::UnixListener::bind(&endpoint)?;
        std::fs::set_permissions(&endpoint, std::fs::Permissions::from_mode(0o600))?;
        listener
    };
    #[cfg(windows)]
    let mut listener =
        crate::transport::windows_pipe::create(&endpoint, true).map_err(std::io::Error::other)?;
    let slots = Arc::new(Semaphore::new(32));
    let terminals = Arc::new(terminals::Terminals::default());
    let tasks = Arc::new(tasks::Projects::new(&directory));
    let mut idle_since = Instant::now();
    loop {
        let stream = tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                if slots.available_permits() != 32 || terminals.active() { idle_since = Instant::now(); }
                if tasks.guests.reap_and_active() { idle_since = Instant::now(); }
                if idle_since.elapsed() >= Duration::from_secs(60) { break; }
                continue;
            }
            accepted = async {
                #[cfg(unix)]
                { listener.accept().await.map(|(stream, _)| stream) }
                #[cfg(windows)]
                {
                    listener.connect().await?;
                    let next = crate::transport::windows_pipe::create(&endpoint, false)
                        .map_err(std::io::Error::other)?;
                    Ok::<_, std::io::Error>(std::mem::replace(&mut listener, next))
                }
            } => accepted?,
        };
        let Ok(slot) = slots.clone().try_acquire_owned() else {
            continue;
        };
        let directory = directory.clone();
        // Native ownership and admission both require a resolvable same-user peer.
        #[cfg(unix)]
        let peer_pid = {
            let Ok(credentials) = stream.peer_cred() else {
                continue;
            };
            if credentials.uid() != unsafe { libc::geteuid() } {
                continue;
            }
            credentials.pid().and_then(|pid| u32::try_from(pid).ok())
        };
        #[cfg(windows)]
        let peer_pid = {
            use std::os::windows::io::AsRawHandle;
            use windows::Win32::{Foundation::HANDLE, System::Pipes::GetNamedPipeClientProcessId};
            let mut pid = 0;
            let Some(pid) = unsafe {
                GetNamedPipeClientProcessId(HANDLE(stream.as_raw_handle()), &mut pid)
            }
            .ok()
            .and_then(|_| (pid > 1).then_some(pid)) else {
                continue;
            };
            let Ok(current) = crate::transport::windows_pipe::current_user_sid() else {
                continue;
            };
            let Ok(peer) = crate::transport::windows_pipe::client_user_sid(pid) else {
                continue;
            };
            if current != peer {
                continue;
            }
            Some(pid)
        };
        idle_since = Instant::now();
        let boot = boot.clone();
        let terminals = terminals.clone();
        let tasks = tasks.clone();
        tokio::spawn(async move {
            let _slot = slot;
            let _ = serve_channel(stream, &directory, &boot, terminals, tasks, peer_pid, None, None, None, false).await;
        });
    }
    #[cfg(unix)]
    let _ = std::fs::remove_file(&endpoint);
    drop(lock);
    Ok(())
}
