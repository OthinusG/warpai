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
    let stream = match connect(&endpoint).await {
        Ok(stream) => stream,
        Err(_) => {
            start_service()?;
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    if let Ok(stream) = connect(&endpoint).await {
                        break stream;
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            })
            .await
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::TimedOut))?
        }
    };
    forward_stdio(stream, None).await
}

pub(super) async fn forward_stdio(
    stream: Stream,
    mut expected: Option<(String, Uuid, Uuid)>,
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
        if let Some((scope, guardian, boot)) = expected.take() {
            if !verified_initialize(&reply, &scope, guardian, boot) {
                return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied).into());
            }
        }
        write_message_with_limit(&mut stdout, &reply, MAX_MANAGED_MESSAGE_SIZE).await?;
    }
}

fn verified_initialize(
    reply: &ServerMessage,
    service_id: &str,
    guardian: Uuid,
    boot: Uuid,
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
    initialized.account_fence.as_ref().is_some_and(|account| {
        account.guardian_id == guardian.to_string()
            && account.boot_id == boot.to_string()
            && account.boot_id == fence.service_boot_id
            && account.connection_id == fence.connection_id
    })
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
        assert!(verified_initialize(&reply, &scope, guardian, boot));
        assert!(!verified_initialize(&reply, &scope, guardian, Uuid::new_v4()));
        assert!(!verified_initialize(&reply, &Uuid::new_v4().to_string(), guardian, boot));
    }
}

pub(super) async fn serve() -> Result<(), ProtocolError> {
    serve_native(#[cfg(windows)] false).await
}

#[cfg(windows)]
pub(super) async fn serve_windows_guardian() -> Result<(), ProtocolError> {
    serve_native(true).await
}

async fn serve_native(#[cfg(windows)] guarded: bool) -> Result<(), ProtocolError> {
    #[cfg(unix)]
    let directory = data_directory()?;
    #[cfg(windows)]
    let directory = if guarded {
        guardian::scope_directory(Route::Ssh).await?
    } else {
        data_directory()?
    };
    identity::private_directory(&directory)?;
    let lock = identity::private_file(&directory.join("service.lock"))?;
    lock.try_lock().map_err(std::io::Error::other)?;
    files::clean_abandoned(&directory)?;
    #[cfg(unix)]
    let endpoint = endpoint(&directory)?;
    #[cfg(windows)]
    let endpoint = if guarded { guardian::endpoint()? } else { endpoint(&directory)? };
    let boot = Uuid::new_v4();
    #[cfg(windows)]
    let guardian = if guarded {
        let account = identity::Identity::open(&directory)?.service_id.parse::<Uuid>()
            .map_err(std::io::Error::other)?;
        let mut catalog = guardian_catalog::Catalog::open(&directory.join("guardian-catalog.sqlite3"))
            .map_err(std::io::Error::other)?;
        catalog.begin_boot(unix_millis()).map_err(std::io::Error::other)?;
        Some((account, Arc::new(std::sync::Mutex::new(catalog))))
    } else {
        None
    };
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
    #[cfg(windows)]
    let handshakes = Arc::new(Semaphore::new(32));
    #[cfg(windows)]
    let operators = Arc::new(Semaphore::new(2));
    #[cfg(windows)]
    let mut project_reaper = None::<tokio::task::JoinHandle<()>>;
    #[cfg(windows)]
    let mut next_project_reap = Instant::now();
    let mut maintenance = tokio::time::interval(Duration::from_secs(1));
    maintenance.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut idle_since = Instant::now();
    loop {
        let stream = tokio::select! {
            _ = maintenance.tick() => {
                #[cfg(windows)]
                if let Some((_, catalog)) = &guardian {
                    for attachment in terminals.pending_attachments() {
                        guardian_catalog::sync_attachment(&terminals, catalog, &attachment);
                    }
                    guardian_catalog::sync_ended(&terminals, catalog);
                    if let Ok(mut catalog) = catalog.lock() {
                        let _ = catalog.prune_ended(unix_millis() - guardian_catalog::ENDED_TTL_MS);
                    }
                    if handshakes.available_permits() != 32 || operators.available_permits() != 2 {
                        idle_since = Instant::now();
                    }
                    if Instant::now() >= next_project_reap
                        && project_reaper.as_ref().is_none_or(tokio::task::JoinHandle::is_finished)
                    {
                        next_project_reap = Instant::now() + Duration::from_secs(10);
                        let projects = tasks.clone();
                        project_reaper = Some(tokio::task::spawn_blocking(move || projects.reap_idle_brokers()));
                    }
                }
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
        let terminals = terminals.clone();
        let tasks = tasks.clone();
        let boot_id = boot.to_string();
        #[cfg(windows)]
        if let Some((account, catalog)) = &guardian {
            let Ok(handshake) = handshakes.clone().try_acquire_owned() else { continue };
            let slots = slots.clone();
            let operators = operators.clone();
            let account = *account;
            let catalog = catalog.clone();
            tokio::spawn(async move {
                use super::guardian_ipc::{preface, read_preface, Admission, REPLY_BUSY};
                use tokio::io::AsyncWriteExt;
                let mut stream = stream;
                let route = Route::Ssh;
                let Ok(admission) = read_preface(&mut stream, route).await else { return };
                let capacity = if admission == Admission::Operator { operators } else { slots };
                let mut reply = preface(route, account, boot);
                let Ok(_slot) = capacity.try_acquire_owned() else {
                    reply[7] = REPLY_BUSY;
                    let _ = stream.write_all(&reply).await;
                    return;
                };
                drop(handshake);
                if stream.write_all(&reply).await.is_err() {
                    return;
                }
                let _ = serve_channel(
                    stream, &directory, &boot_id, terminals, tasks, peer_pid, Some(route), Some(catalog),
                    Some(account),
                    matches!(admission, Admission::Operator | Admission::OperatorSession),
                ).await;
            });
            continue;
        }
        let Ok(slot) = slots.clone().try_acquire_owned() else { continue };
        tokio::spawn(async move {
            let _slot = slot;
            let _ = serve_channel(stream, &directory, &boot_id, terminals, tasks, peer_pid, None, None, None, false).await;
        });
    }
    #[cfg(windows)]
    if let Some((_, catalog)) = guardian {
        if let Some(reaper) = project_reaper {
            let _ = reaper.await;
        }
        guardian_catalog::sync_ended(&terminals, &catalog);
    }
    #[cfg(unix)]
    let _ = std::fs::remove_file(&endpoint);
    drop(lock);
    Ok(())
}
