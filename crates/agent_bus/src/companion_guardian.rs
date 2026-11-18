//! Account-global guardian election and route-scoped Unix listeners.
#![cfg(unix)]
use super::*;
use std::{
    ffi::OsStr,
    io::ErrorKind,
    os::unix::{
        ffi::OsStrExt,
        fs::{FileTypeExt, MetadataExt, PermissionsExt},
    },
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::Semaphore,
};

pub const IPC_MAJOR: u16 = 1;
const MAGIC: [u8; 4] = *b"WAG1";
const PREFACE_BYTES: usize = 4 + 2 + 1 + 1 + 16 + 16;
const IDLE_EXIT: Duration = Duration::from_secs(60);
const SLOTS: usize = 32;

fn euid() -> u32 {
    unsafe { libc::geteuid() }
}

/// Account home from passwd, never HOME/XDG or route environment.
fn account_home() -> std::io::Result<PathBuf> {
    let mut buffer = vec![0u8; 4096];
    let mut entry = unsafe { std::mem::zeroed::<libc::passwd>() };
    let mut result = std::ptr::null_mut();
    let status = unsafe {
        libc::getpwuid_r(
            euid(),
            &mut entry,
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut result,
        )
    };
    if status != 0 || result.is_null() || entry.pw_dir.is_null() {
        return Err(std::io::Error::other("Account home unavailable"));
    }
    let home = unsafe { std::ffi::CStr::from_ptr(entry.pw_dir) };
    let home = PathBuf::from(OsStr::from_bytes(home.to_bytes()));
    if !home.is_absolute() {
        return Err(std::io::Error::other("Account home is not absolute"));
    }
    Ok(home)
}

pub(crate) fn root_directory() -> std::io::Result<PathBuf> {
    let home = account_home()?;
    #[cfg(target_os = "macos")]
    {
        Ok(home.join("Library/Application Support/warpai/guardian"))
    }
    #[cfg(target_os = "linux")]
    {
        Ok(home.join(".local/share/warpai/guardian"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = home;
        Err(std::io::Error::from(ErrorKind::Unsupported))
    }
}

pub(crate) fn runtime_directory() -> std::io::Result<PathBuf> {
    Ok(PathBuf::from("/tmp").join(format!("warpai-guardian-{}", euid())))
}

/// Existing logical scope for a route; Stores stay route-local.
pub(crate) fn scope_directory(route: Route) -> std::io::Result<PathBuf> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| std::io::Error::other("Scope data directory unavailable"))?
        .join("warpai");
    match route {
        Route::Ssh => Ok(base.join("remote")),
        Route::Wsl => {
            #[cfg(all(target_os = "linux", feature = "wsl_companion"))]
            {
                let distribution = std::env::var("WSL_DISTRO_NAME")
                    .map_err(|_| std::io::Error::other("WSL distribution identity unavailable"))?;
                Ok(base
                    .join("wsl-remote")
                    .join(guests::distribution_namespace(&distribution)?))
            }
            #[cfg(not(all(target_os = "linux", feature = "wsl_companion")))]
            {
                Err(std::io::Error::from(ErrorKind::Unsupported))
            }
        }
    }
}

fn supported_routes() -> Vec<Route> {
    let mut routes = vec![Route::Ssh];
    #[cfg(all(target_os = "linux", feature = "wsl_companion"))]
    routes.push(Route::Wsl);
    routes
}

impl Route {
    fn code(self) -> u8 {
        match self {
            Self::Ssh => 1,
            Self::Wsl => 2,
        }
    }
    fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Ssh),
            2 => Some(Self::Wsl),
            _ => None,
        }
    }
    fn socket_name(self) -> &'static str {
        match self {
            Self::Ssh => "ssh.sock",
            Self::Wsl => "wsl.sock",
        }
    }
}

fn preface(route: Route, guardian: Uuid, boot: Uuid) -> [u8; PREFACE_BYTES] {
    let mut bytes = [0; PREFACE_BYTES];
    bytes[..4].copy_from_slice(&MAGIC);
    bytes[4..6].copy_from_slice(&IPC_MAJOR.to_be_bytes());
    bytes[6] = route.code();
    bytes[8..24].copy_from_slice(guardian.as_bytes());
    bytes[24..40].copy_from_slice(boot.as_bytes());
    bytes
}

fn request_preface(route: Route) -> [u8; PREFACE_BYTES] {
    preface(route, Uuid::nil(), Uuid::nil())
}

fn valid_reply(bytes: &[u8; PREFACE_BYTES], route: Route) -> Option<(Uuid, Uuid)> {
    if bytes[..4] != MAGIC
        || u16::from_be_bytes([bytes[4], bytes[5]]) != IPC_MAJOR
        || Route::from_code(bytes[6]) != Some(route)
        || bytes[7] != 0
    {
        return None;
    }
    let guardian = Uuid::from_slice(&bytes[8..24]).ok()?;
    let boot = Uuid::from_slice(&bytes[24..40]).ok()?;
    (!guardian.is_nil() && !boot.is_nil()).then_some((guardian, boot))
}

/// Server side: verify the client preface and answer with the runtime identity.
async fn accept_preface(
    stream: &mut UnixStream,
    route: Route,
    guardian: Uuid,
    boot: Uuid,
) -> std::io::Result<()> {
    let mut bytes = [0; PREFACE_BYTES];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut bytes))
        .await
        .map_err(|_| std::io::Error::from(ErrorKind::TimedOut))??;
    if bytes[..4] != MAGIC
        || u16::from_be_bytes([bytes[4], bytes[5]]) != IPC_MAJOR
        || Route::from_code(bytes[6]) != Some(route)
        || bytes[7] != 0
        || bytes[8..].iter().any(|byte| *byte != 0)
    {
        return Err(std::io::Error::from(ErrorKind::InvalidData));
    }
    stream.write_all(&preface(route, guardian, boot)).await
}

/// Client side: connect and verify the guardian identity before managed framing.
async fn probe_route(runtime: &Path, route: Route) -> std::io::Result<(UnixStream, Uuid, Uuid)> {
    let mut stream = UnixStream::connect(runtime.join(route.socket_name())).await?;
    stream.write_all(&request_preface(route)).await?;
    let mut bytes = [0; PREFACE_BYTES];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut bytes))
        .await
        .map_err(|_| std::io::Error::from(ErrorKind::TimedOut))??;
    let (guardian, boot) =
        valid_reply(&bytes, route).ok_or_else(|| std::io::Error::from(ErrorKind::InvalidData))?;
    Ok((stream, guardian, boot))
}

fn start_service() -> std::io::Result<()> {
    let mut command = tokio::process::Command::new(std::env::current_exe()?);
    command
        .arg("--guardian-service")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    crate::session::without_terminal_binding(&mut command);
    command.env_remove("VIBE_MCP_SERVERS");
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
    let mut child = command.spawn()?;
    tokio::spawn(async move {
        let _ = child.wait().await;
    });
    Ok(())
}

/// Connect to the live account guardian, starting one only when no owner answers.
pub(crate) async fn connect(route: Route) -> std::io::Result<UnixStream> {
    let runtime = runtime_directory()?;
    if let Ok((stream, _, _)) = probe_route(&runtime, route).await {
        return Ok(stream);
    }
    start_service()?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match probe_route(&runtime, route).await {
            Ok((stream, _, _)) => return Ok(stream),
            Err(_) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(error) => return Err(error),
        }
    }
}

pub(crate) async fn serve() -> Result<(), ProtocolError> {
    let root = root_directory()?;
    let runtime = runtime_directory()?;
    let scopes = supported_routes()
        .into_iter()
        .filter_map(|route| {
            scope_directory(route)
                .ok()
                .map(|directory| (route, directory))
        })
        .collect();
    serve_at(root, runtime, scopes).await
}

struct Scope {
    route: Route,
    directory: PathBuf,
    tasks: Arc<tasks::Projects>,
    _lock: File,
}

/// Bind one private listener per admitted route under one account lock.
pub(crate) async fn serve_at(
    root: PathBuf,
    runtime: PathBuf,
    scopes: Vec<(Route, PathBuf)>,
) -> Result<(), ProtocolError> {
    identity::private_directory(&root)?;
    let lock = identity::private_file(&root.join("service.lock"))?;
    match lock.try_lock() {
        Ok(()) => (),
        // A live owner holds the account lock; the candidate exits without binding.
        Err(std::fs::TryLockError::WouldBlock) => return Ok(()),
        Err(error) => return Err(std::io::Error::other(error).into()),
    }
    let identity = identity::Identity::open(&root)?;
    let guardian: Uuid = identity
        .service_id
        .parse()
        .map_err(|_| std::io::Error::other("Guardian identity unavailable"))?;
    let boot = Uuid::new_v4();
    let mut catalog = guardian_catalog::Catalog::open(&root.join("catalog.sqlite3"))
        .map_err(std::io::Error::other)?;
    let observed_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0);
    catalog
        .begin_boot(observed_ms)
        .map_err(std::io::Error::other)?;
    let catalog = Arc::new(std::sync::Mutex::new(catalog));
    identity::private_directory(&runtime)?;
    let mut admitted = Vec::new();
    for (route, directory) in scopes {
        identity::private_directory(&directory)?;
        let scope_lock = identity::private_file(&directory.join("service.lock"))?;
        match scope_lock.try_lock() {
            Ok(()) => (),
            // Live legacy ownership refuses this scope; never stop or duplicate it.
            Err(std::fs::TryLockError::WouldBlock) => continue,
            Err(error) => return Err(std::io::Error::other(error).into()),
        }
        let tasks = Arc::new(tasks::Projects::new(&directory));
        admitted.push(Scope {
            route,
            directory,
            tasks,
            _lock: scope_lock,
        });
    }
    let terminals = Arc::new(terminals::Terminals::default());
    let mut listeners = Vec::new();
    for scope in &admitted {
        let path = runtime.join(scope.route.socket_name());
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if !metadata.file_type().is_socket() || metadata.uid() != euid() {
                    return Err(std::io::Error::from(ErrorKind::PermissionDenied).into());
                }
                match probe_route(&runtime, scope.route).await {
                    Ok(_) => return Ok(()),
                    Err(error)
                        if matches!(
                            error.kind(),
                            ErrorKind::ConnectionRefused | ErrorKind::NotFound
                        ) => {}
                    // A responsive but incompatible owner is never unlinked.
                    Err(error) => return Err(error.into()),
                }
                std::fs::remove_file(&path)?;
            }
            Err(error) if error.kind() == ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
        let listener = tokio::net::UnixListener::bind(&path)?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        let metadata = std::fs::symlink_metadata(&path)?;
        if !metadata.file_type().is_socket() || metadata.uid() != euid() {
            return Err(std::io::Error::from(ErrorKind::PermissionDenied).into());
        }
        listeners.push((listener, scope.route));
    }
    let slots = Arc::new(Semaphore::new(SLOTS));
    let (sender, mut receiver) = tokio::sync::mpsc::channel(32);
    for (listener, route) in listeners {
        let sender = sender.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        if sender.send((stream, route)).await.is_err() {
                            return;
                        }
                    }
                    Err(_) => tokio::time::sleep(Duration::from_millis(10)).await,
                }
            }
        });
    }
    drop(sender);
    let mut idle_since = Instant::now();
    loop {
        tokio::select! {
            accepted = receiver.recv() => {
                let Some((mut stream, route)) = accepted else { break };
                let Ok(credentials) = stream.peer_cred() else { continue };
                if credentials.uid() != euid() {
                    continue;
                }
                let Some(scope) = admitted.iter().find(|scope| scope.route == route) else {
                    continue;
                };
                if accept_preface(&mut stream, route, guardian, boot).await.is_err() {
                    continue;
                }
                let Ok(slot) = slots.clone().try_acquire_owned() else { continue };
                idle_since = Instant::now();
                let peer_pid = credentials
                    .pid()
                    .and_then(|pid| u32::try_from(pid).ok());
                let directory = scope.directory.clone();
                let tasks = scope.tasks.clone();
                let terminals = terminals.clone();
                let catalog = catalog.clone();
                let boot = boot.to_string();
                tokio::spawn(async move {
                    let _slot = slot;
                    let _ = serve_channel(
                        stream,
                        &directory,
                        &boot,
                        terminals,
                        tasks,
                        peer_pid,
                        Some(route),
                        Some(catalog),
                    )
                    .await;
                });
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                if slots.available_permits() != SLOTS || terminals.active() {
                    idle_since = Instant::now();
                }
                for scope in &admitted {
                    if scope.tasks.guests.reap_and_active() {
                        idle_since = Instant::now();
                    }
                }
                if idle_since.elapsed() >= IDLE_EXIT {
                    break;
                }
            }
        }
    }
    for scope in &admitted {
        let _ = std::fs::remove_file(runtime.join(scope.route.socket_name()));
    }
    drop(lock);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scopes(root: &Path) -> Vec<(Route, PathBuf)> {
        vec![
            (Route::Ssh, root.join("ssh")),
            (Route::Wsl, root.join("wsl")),
        ]
    }

    #[test]
    fn preface_round_trips_and_rejects_unknown_routes() {
        let guardian = Uuid::new_v4();
        let boot = Uuid::new_v4();
        let bytes = preface(Route::Ssh, guardian, boot);
        assert_eq!(valid_reply(&bytes, Route::Ssh), Some((guardian, boot)));
        assert_eq!(valid_reply(&bytes, Route::Wsl), None);
        let mut unknown = bytes;
        unknown[6] = 9;
        assert_eq!(valid_reply(&unknown, Route::Ssh), None);
        let mut nil = bytes;
        nil[8..24].fill(0);
        assert_eq!(valid_reply(&nil, Route::Ssh), None);
    }

    #[tokio::test]
    async fn concurrent_candidates_yield_one_owner_and_shared_boot() {
        let state = tempfile::tempdir().unwrap();
        let root = state.path().join("guardian");
        let runtime = state.path().join("run");
        let first = {
            let root = root.clone();
            let runtime = runtime.clone();
            let scopes = scopes(state.path());
            tokio::spawn(async move { serve_at(root, runtime, scopes).await })
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        let (stream, guardian, boot) = loop {
            if let Ok(probe) = probe_route(&runtime, Route::Ssh).await {
                break probe;
            }
            assert!(Instant::now() < deadline, "Guardian did not start");
            tokio::time::sleep(Duration::from_millis(20)).await;
        };
        let second = {
            let root = root.clone();
            let runtime = runtime.clone();
            let scopes = scopes(state.path());
            tokio::spawn(async move { serve_at(root, runtime, scopes).await })
        };
        tokio::time::timeout(Duration::from_secs(5), second)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let (_ssh, second_guardian, second_boot) = probe_route(&runtime, Route::Ssh).await.unwrap();
        let (_wsl, third_guardian, third_boot) = probe_route(&runtime, Route::Wsl).await.unwrap();
        assert_eq!(guardian, second_guardian);
        assert_eq!(guardian, third_guardian);
        assert_eq!(boot, second_boot);
        assert_eq!(boot, third_boot);
        drop(stream);
        first.abort();
    }

    #[tokio::test]
    async fn legacy_scope_contention_refuses_only_that_route() {
        let state = tempfile::tempdir().unwrap();
        let root = state.path().join("guardian");
        let runtime = state.path().join("run");
        let wsl = state.path().join("wsl");
        identity::private_directory(&wsl).unwrap();
        let held = identity::private_file(&wsl.join("service.lock")).unwrap();
        held.try_lock().unwrap();
        let server = {
            let root = root.clone();
            let runtime = runtime.clone();
            let scopes = vec![(Route::Ssh, state.path().join("ssh")), (Route::Wsl, wsl)];
            tokio::spawn(async move { serve_at(root, runtime, scopes).await })
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if probe_route(&runtime, Route::Ssh).await.is_ok() {
                break;
            }
            assert!(Instant::now() < deadline, "SSH route did not start");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(probe_route(&runtime, Route::Wsl).await.is_err());
        server.abort();
    }

    #[tokio::test]
    async fn owned_stale_socket_is_reclaimed_under_the_account_lock() {
        let state = tempfile::tempdir().unwrap();
        let root = state.path().join("guardian");
        let runtime = state.path().join("run");
        identity::private_directory(&runtime).unwrap();
        let stale = tokio::net::UnixListener::bind(runtime.join("ssh.sock")).unwrap();
        drop(stale);
        let server = {
            let root = root.clone();
            let runtime = runtime.clone();
            let scopes = vec![(Route::Ssh, state.path().join("ssh"))];
            tokio::spawn(async move { serve_at(root, runtime, scopes).await })
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if probe_route(&runtime, Route::Ssh).await.is_ok() {
                break;
            }
            assert!(Instant::now() < deadline, "Stale socket was not reclaimed");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        server.abort();
    }

    #[tokio::test]
    async fn invalid_peer_preface_is_rejected_before_managed_framing() {
        let state = tempfile::tempdir().unwrap();
        let root = state.path().join("guardian");
        let runtime = state.path().join("run");
        let server = {
            let root = root.clone();
            let runtime = runtime.clone();
            let scopes = vec![(Route::Ssh, state.path().join("ssh"))];
            tokio::spawn(async move { serve_at(root, runtime, scopes).await })
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        let path = runtime.join("ssh.sock");
        while tokio::net::UnixStream::connect(&path).await.is_err() {
            assert!(Instant::now() < deadline, "Guardian did not start");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let mut stream = UnixStream::connect(&path).await.unwrap();
        stream.write_all(b"bad!").await.unwrap();
        stream.shutdown().await.unwrap();
        let mut byte = [0u8; 1];
        let read = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut byte))
            .await
            .unwrap();
        assert!(matches!(read, Ok(0) | Err(_)));
        server.abort();
    }
}
