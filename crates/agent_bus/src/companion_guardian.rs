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
    #[cfg(all(target_os = "linux", feature = "wsl_companion"))]
    {
        vec![Route::Ssh, Route::Wsl]
    }
    #[cfg(not(all(target_os = "linux", feature = "wsl_companion")))]
    {
        vec![Route::Ssh]
    }
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Admission {
    Guarded = 0,
    Operator = 1,
    ConnectionOwned = 2,
}

/// Server side: verify the client preface and answer with the runtime identity.
async fn accept_preface(
    stream: &mut UnixStream,
    route: Route,
    guardian: Uuid,
    boot: Uuid,
) -> std::io::Result<Admission> {
    let mut bytes = [0; PREFACE_BYTES];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut bytes))
        .await
        .map_err(|_| std::io::Error::from(ErrorKind::TimedOut))??;
    if bytes[..4] != MAGIC
        || u16::from_be_bytes([bytes[4], bytes[5]]) != IPC_MAJOR
        || Route::from_code(bytes[6]) != Some(route)
        || bytes[7] > 2
        || bytes[8..].iter().any(|byte| *byte != 0)
    {
        return Err(std::io::Error::from(ErrorKind::InvalidData));
    }
    stream.write_all(&preface(route, guardian, boot)).await?;
    Ok(match bytes[7] {
        0 => Admission::Guarded,
        1 => Admission::Operator,
        _ => Admission::ConnectionOwned,
    })
}

async fn exchange_preface(
    mut stream: UnixStream,
    route: Route,
    admission: Admission,
) -> std::io::Result<(UnixStream, Uuid, Uuid)> {
    let mut request = request_preface(route);
    request[7] = admission as u8;
    stream.write_all(&request).await?;
    let mut bytes = [0; PREFACE_BYTES];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut bytes))
        .await
        .map_err(|_| std::io::Error::from(ErrorKind::TimedOut))??;
    let (guardian, boot) =
        valid_reply(&bytes, route).ok_or_else(|| std::io::Error::from(ErrorKind::InvalidData))?;
    Ok((stream, guardian, boot))
}

fn private_directory(path: &Path) -> std::io::Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_dir() || metadata.uid() != euid() || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(std::io::Error::from(ErrorKind::PermissionDenied));
    }
    Ok(())
}

fn private_socket(path: &Path) -> std::io::Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.file_type().is_socket()
        || metadata.uid() != euid()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(std::io::Error::from(ErrorKind::PermissionDenied));
    }
    Ok(())
}

/// Client side: verify kernel peer identity, private ownership and the guardian tuple.
async fn probe_route(runtime: &Path, route: Route) -> std::io::Result<(UnixStream, Uuid, Uuid)> {
    probe_route_as(runtime, route, Admission::Guarded).await
}

async fn probe_route_as(runtime: &Path, route: Route, admission: Admission) -> std::io::Result<(UnixStream, Uuid, Uuid)> {
    private_directory(runtime)?;
    private_socket(&runtime.join(route.socket_name()))?;
    let stream = UnixStream::connect(runtime.join(route.socket_name())).await?;
    if stream.peer_cred()?.uid() != euid() {
        return Err(std::io::Error::from(ErrorKind::PermissionDenied));
    }
    exchange_preface(stream, route, admission).await
}

/// Owner-side stale detection: any responsive listener, regardless of permissions.
async fn probe_live(runtime: &Path, route: Route) -> std::io::Result<(UnixStream, Uuid, Uuid)> {
    let stream = UnixStream::connect(runtime.join(route.socket_name())).await?;
    exchange_preface(stream, route, Admission::Guarded).await
}

fn route_absent(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        ErrorKind::NotFound | ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset
    )
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

/// Probe-only: never starts a service and never activates a route.
pub(crate) async fn probe(route: Route) -> std::io::Result<(UnixStream, Uuid, Uuid)> {
    let runtime = runtime_directory()?;
    probe_route_as(&runtime, route, Admission::Operator).await
}

/// Ordinary helpers join the current owner without enabling guarded features.
pub(super) async fn probe_connection_owned() -> std::io::Result<Option<UnixStream>> {
    let route = if cfg!(all(target_os = "linux", feature = "wsl_companion")) {
        Route::Wsl
    } else {
        Route::Ssh
    };
    let runtime = runtime_directory()?;
    match probe_route_as(&runtime, route, Admission::ConnectionOwned).await {
        Ok((stream, _, _)) => Ok(Some(stream)),
        Err(error) if route_absent(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

/// Connect to the live account guardian, starting one only when no owner answers.
pub(crate) async fn connect(route: Route) -> std::io::Result<UnixStream> {
    connect_inner(route, true).await
}

/// Spawn is allowed only for an absent route; incompatible or denied owners fail closed.
async fn connect_inner(route: Route, may_start: bool) -> std::io::Result<UnixStream> {
    let runtime = runtime_directory()?;
    match probe_route(&runtime, route).await {
        Ok((stream, _, _)) => return Ok(stream),
        Err(error) if !route_absent(&error) => return Err(error),
        Err(_) => (),
    }
    if !may_start {
        return Err(std::io::Error::from(ErrorKind::NotFound));
    }
    start_service()?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match probe_route(&runtime, route).await {
            Ok((stream, _, _)) => return Ok(stream),
            Err(error) if route_absent(&error) && Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(error) if route_absent(&error) => {
                return Err(std::io::Error::new(ErrorKind::TimedOut, error));
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

fn wall_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0)
}

/// Sole lifecycle authority reports observed exit once, independent of client reads.
pub(super) fn sync_ended(terminals: &terminals::Terminals, catalog: &CatalogHandle) {
    let ended = terminals.ended_guarded();
    if ended.is_empty() {
        return;
    }
    let observed = wall_millis();
    for session in ended {
        let recorded = catalog.lock().is_ok_and(|mut catalog| {
            catalog
                .update_state(
                    &session.session_id,
                    &session.run_id,
                    &session.boot_id,
                    &session.service_id,
                    guardian_catalog::State::Ended,
                    observed,
                    session.exit_code,
                )
                .is_ok()
        });
        if recorded {
            terminals.mark_end_reported(&session.session_id, &session.run_id);
        }
    }
}

struct Scope {
    route: Route,
    directory: PathBuf,
    tasks: Arc<tasks::Projects>,
    _lock: File,
}

/// `Ok(None)` means a live legacy owner still holds the scope lock.
fn admit_scope(route: Route, directory: PathBuf) -> std::io::Result<Option<Scope>> {
    identity::private_directory(&directory)?;
    let scope_lock = identity::private_file(&directory.join("service.lock"))?;
    match scope_lock.try_lock() {
        Ok(()) => (),
        // Never stop or duplicate live legacy ownership.
        Err(std::fs::TryLockError::WouldBlock) => return Ok(None),
        Err(error) => return Err(std::io::Error::other(error)),
    }
    let tasks = Arc::new(tasks::Projects::new(&directory));
    Ok(Some(Scope {
        route,
        directory,
        tasks,
        _lock: scope_lock,
    }))
}

async fn bind_route(runtime: &Path, route: Route) -> std::io::Result<tokio::net::UnixListener> {
    let path = runtime.join(route.socket_name());
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if !metadata.file_type().is_socket() || metadata.uid() != euid() {
                return Err(std::io::Error::from(ErrorKind::PermissionDenied));
            }
            match probe_live(runtime, route).await {
                Ok(_) => return Err(std::io::Error::from(ErrorKind::AddrInUse)),
                Err(error)
                    if matches!(
                        error.kind(),
                        ErrorKind::ConnectionRefused | ErrorKind::NotFound
                    ) => {}
                // A responsive but incompatible owner is never unlinked.
                Err(error) => return Err(error),
            }
            std::fs::remove_file(&path)?;
        }
        Err(error) if error.kind() == ErrorKind::NotFound => (),
        Err(error) => return Err(error),
    }
    let listener = tokio::net::UnixListener::bind(&path)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    private_socket(&path)?;
    Ok(listener)
}

fn spawn_accept(
    listener: tokio::net::UnixListener,
    route: Route,
    sender: tokio::sync::mpsc::Sender<(UnixStream, Route)>,
) {
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
    let observed_ms = wall_millis();
    catalog
        .begin_boot(observed_ms)
        .map_err(std::io::Error::other)?;
    let catalog = Arc::new(std::sync::Mutex::new(catalog));
    identity::private_directory(&runtime)?;
    let mut admitted = Vec::new();
    // Held scopes stay pending: the guardian admits them once the legacy owner exits.
    let mut pending = Vec::new();
    for (route, directory) in scopes {
        match admit_scope(route, directory.clone()) {
            Ok(Some(scope)) => admitted.push(scope),
            Ok(None) => pending.push((route, directory)),
            Err(error) => return Err(std::io::Error::other(error).into()),
        }
    }
    if admitted.is_empty() && pending.is_empty() {
        return Err(std::io::Error::other("No guardian route configured").into());
    }
    let terminals = Arc::new(terminals::Terminals::default());
    let mut listeners = Vec::new();
    for scope in &admitted {
        match bind_route(&runtime, scope.route).await {
            Ok(listener) => listeners.push((listener, scope.route)),
            Err(error) if error.kind() == ErrorKind::AddrInUse => return Ok(()),
            Err(error) => return Err(error.into()),
        }
    }
    let slots = Arc::new(Semaphore::new(SLOTS));
    let (sender, mut receiver) = tokio::sync::mpsc::channel(32);
    for (listener, route) in listeners {
        spawn_accept(listener, route, sender.clone());
    }
    // The dispatch loop, not a listener, keeps the channel open.
    let _keepalive = sender.clone();
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
                let Ok(slot) = slots.clone().try_acquire_owned() else { continue };
                idle_since = Instant::now();
                let peer_pid = credentials
                    .pid()
                    .and_then(|pid| u32::try_from(pid).ok());
                let directory = scope.directory.clone();
                let tasks = scope.tasks.clone();
                let terminals = terminals.clone();
                let catalog = catalog.clone();
                let boot_id = boot.to_string();
                // The bounded preface runs off the dispatch loop; a stalled peer
                // cannot delay accepts or idle accounting.
                tokio::spawn(async move {
                    let _slot = slot;
                    let Ok(admission) = accept_preface(&mut stream, route, guardian, boot).await else {
                        return;
                    };
                    let _ = serve_channel(
                        stream,
                        &directory,
                        &boot_id,
                        terminals,
                        tasks,
                        peer_pid,
                        Some(route),
                        Some(catalog),
                        (admission != Admission::ConnectionOwned).then_some(guardian),
                        admission == Admission::Operator,
                    )
                    .await;
                });
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                sync_ended(&terminals, &catalog);
                let mut admitted_new = false;
                let mut index = 0;
                while index < pending.len() {
                    let (route, directory) = pending[index].clone();
                    match admit_scope(route, directory.clone()) {
                        Ok(Some(scope)) => {
                            pending.remove(index);
                            match bind_route(&runtime, route).await {
                                Ok(listener) => {
                                    spawn_accept(listener, route, sender.clone());
                                    admitted.push(scope);
                                    admitted_new = true;
                                }
                                Err(_) => {
                                    pending.push((route, directory));
                                    index += 1;
                                }
                            }
                        }
                        Ok(None) | Err(_) => index += 1,
                    }
                }
                if let Ok(mut catalog) = catalog.lock() {
                    let _ = catalog.prune_ended(wall_millis() - guardian_catalog::ENDED_TTL_MS);
                }
                if slots.available_permits() != SLOTS || terminals.active() || admitted_new {
                    idle_since = Instant::now();
                }
                for scope in &admitted {
                    if scope.tasks.guests.reap_and_active() {
                        idle_since = Instant::now();
                    }
                }
                // Pending legacy scopes are not work; a later connection
                // re-ensures a guardian and admits them once released.
                if idle_since.elapsed() >= IDLE_EXIT {
                    break;
                }
            }
        }
    }
    sync_ended(&terminals, &catalog);
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
    fn guardian_route_names_are_closed() {
        assert_eq!(guardian_route("ssh"), Some(Route::Ssh));
        assert_eq!(
            guardian_route("wsl"),
            cfg!(all(target_os = "linux", feature = "wsl_companion")).then_some(Route::Wsl)
        );
        assert_eq!(guardian_route("tap"), None);
        assert_eq!(guardian_route(""), None);
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
    async fn released_scope_is_admitted_under_the_same_guardian() {
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
        let (stream, guardian, boot) = wait_route(&runtime).await;
        assert!(probe_route(&runtime, Route::Wsl).await.is_err());
        drop(held);
        let deadline = Instant::now() + Duration::from_secs(10);
        let (wsl_stream, late_guardian, late_boot) = loop {
            if let Ok(probe) = probe_route(&runtime, Route::Wsl).await {
                break probe;
            }
            assert!(Instant::now() < deadline, "Released scope was not admitted");
            tokio::time::sleep(Duration::from_millis(50)).await;
        };
        assert_eq!(late_guardian, guardian);
        assert_eq!(late_boot, boot);
        drop((stream, wsl_stream));
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
    async fn zero_route_startup_fails_closed() {
        let state = tempfile::tempdir().unwrap();
        let result = serve_at(
            state.path().join("guardian"),
            state.path().join("run"),
            Vec::new(),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn client_probe_rejects_loose_socket_and_directory_permissions() {
        let state = tempfile::tempdir().unwrap();
        let runtime = state.path().join("run");
        identity::private_directory(&runtime).unwrap();
        let path = runtime.join("ssh.sock");
        let listener = tokio::net::UnixListener::bind(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let loose = probe_route(&runtime, Route::Ssh).await.unwrap_err();
        assert_eq!(loose.kind(), ErrorKind::PermissionDenied);
        drop(listener);
        std::fs::set_permissions(&runtime, std::fs::Permissions::from_mode(0o755)).unwrap();
        let directory = probe_route(&runtime, Route::Ssh).await.unwrap_err();
        assert_eq!(directory.kind(), ErrorKind::PermissionDenied);
    }

    #[tokio::test]
    async fn stalled_preface_does_not_block_other_clients() {
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
        let stalled = UnixStream::connect(&path).await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), probe_route(&runtime, Route::Ssh))
            .await
            .expect("Stalled peer blocked dispatch")
            .unwrap();
        drop(stalled);
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

    #[tokio::test(flavor = "multi_thread")]
    async fn cli_catalog_reads_all_pages_from_the_listener() {
        let state = tempfile::tempdir().unwrap();
        let root = state.path().join("guardian");
        let runtime = state.path().join("run");
        identity::private_directory(&root).unwrap();

        {
            let mut catalog = guardian_catalog::Catalog::open(&root.join("catalog.sqlite3")).unwrap();
            for seq in 1..=40 {
                catalog.insert(&guardian_catalog::Record {
                    session_id: Uuid::new_v4().to_string(),
                    run_id: Uuid::new_v4().to_string(),
                    boot_id: Uuid::new_v4().to_string(),
                    service_id: Uuid::new_v4().to_string(),
                    scope_id: "ssh".into(),
                    project_id: Uuid::new_v4().to_string(),
                    policy: "guarded".into(),
                    kind: "shell".into(),
                    label: None,
                    created_seq: seq,
                    mru_seq: seq,
                    state: guardian_catalog::State::Unknown,
                    observed_ms: 0,
                    exit_status: None,
                    attachment_generation: 0,
                }).unwrap();
            }
        }

        let server = {
            let runtime = runtime.clone();
            let scopes = vec![(Route::Ssh, state.path().join("ssh"))];
            tokio::spawn(async move { serve_at(root, runtime, scopes).await })
        };
        let (stream, _, _) = wait_route(&runtime).await;
        let mut client = crate::ssh_remote::HostClient::connect_unix(stream, None)
            .await.unwrap();
        let rows = crate::companion_ctl::rows(&mut client, "").await.unwrap();
        assert_eq!(rows.len(), 40);
        assert_eq!(rows[0].created_seq, 40);
        assert_eq!(rows[39].created_seq, 1);
        server.abort();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn guardian_watch_and_stop_report_owned_runs() {
        let state = tempfile::tempdir().unwrap();
        let root = state.path().join("guardian");
        let runtime = state.path().join("run");
        let project = state.path().join("project");
        std::fs::create_dir_all(&project).unwrap();
        let server = {
            let root = root.clone();
            let runtime = runtime.clone();
            let scopes = vec![(Route::Ssh, state.path().join("ssh"))];
            tokio::spawn(async move { serve_at(root, runtime, scopes).await })
        };
        drop(wait_route(&runtime).await);
        let (stream, _, _) = probe_route_as(&runtime, Route::Ssh, Admission::Operator).await.unwrap();
        let mut client = ManagedClient::new(stream);
        let initialized = match client
            .call(managed_request::Operation::Initialize(ManagedInitialize {
                protocol_major: 1,
            }))
            .await
        {
            managed_response::Result::Initialized(initialized) => initialized,
            _ => panic!("Guardian initialize required"),
        };
        let account = initialized.account_fence.clone().unwrap();
        let opened = match client
            .call(managed_request::Operation::ProjectOpen(ProjectOpen {
                fence: initialized.fence.clone(),
                root: project.to_str().unwrap().into(),
            }))
            .await
        {
            managed_response::Result::ProjectOpened(opened) => opened,
            _ => panic!("Project open required"),
        };
        let project_fence = opened.fence.unwrap();
        let project_id = project_fence.project_id.clone();
        let missing_id = Uuid::new_v4().to_string();
        let failed = client.call(managed_request::Operation::TerminalLaunch(TerminalLaunch {
            fence: Some(project_fence.clone()),
            session_id: missing_id.clone(),
            executable: project.join("missing-executable").to_string_lossy().into_owned(),
            columns: 80,
            rows: 24,
            guarded: true,
            session_kind: Some(TerminalSessionKind::Command as i32),
            ..Default::default()
        })).await;
        assert!(matches!(failed, managed_response::Result::Error(_)));
        let managed_response::Result::AccountCatalogPage(empty) = client.call(
            managed_request::Operation::AccountCatalogList(AccountCatalogList {
                fence: Some(account.clone()),
                ..Default::default()
            })
        ).await else { panic!("Catalog list required") };
        assert!(empty.rows.iter().all(|row| row.session_id != missing_id));

        let first = launch_guarded(
            &mut client,
            &project_fence,
            Uuid::new_v4().to_string(),
        )
        .await;
        let baseline_next = match client
            .call(managed_request::Operation::AccountCatalogWatch(
                AccountCatalogWatch {
                    fence: Some(account.clone()),
                    filter: Some(AccountCatalogFilter {
                        scope_id: String::new(),
                        project_id: String::new(),
                    }),
                    cursor: String::new(),
                    limit: 16,
                },
            ))
            .await
        {
            managed_response::Result::AccountCatalogEvents(events) => {
                assert!(events.events.is_empty());
                assert!(!events.gap);
                events.next.unwrap()
            }
            _ => panic!("Catalog watch required"),
        };
        let second = launch_guarded(
            &mut client,
            &project_fence,
            Uuid::new_v4().to_string(),
        )
        .await;
        let events = match client
            .call(managed_request::Operation::AccountCatalogWatch(
                AccountCatalogWatch {
                    fence: Some(account.clone()),
                    filter: None,
                    cursor: baseline_next.clone(),
                    limit: 16,
                },
            ))
            .await
        {
            managed_response::Result::AccountCatalogEvents(events) => events,
            _ => panic!("Catalog watch required"),
        };
        assert!(!events.gap);
        assert_eq!(events.events.len(), 2);
        assert_eq!(
            events.events[0].change,
            AccountCatalogChange::Upserted as i32
        );
        let row = events.events.last().unwrap().row.as_ref().unwrap();
        assert_eq!(row.state, "live");
        assert_eq!(row.session_id, second.session_id);

        let (ordinary, _, _) = probe_route_as(&runtime, Route::Ssh, Admission::ConnectionOwned)
            .await.unwrap();
        let mut ordinary = ManagedClient::new(ordinary);
        let managed_response::Result::Initialized(ordinary_init) = ordinary
            .call(managed_request::Operation::Initialize(ManagedInitialize { protocol_major: 1 }))
            .await else { panic!("Ordinary initialize required") };
        assert!(ordinary_init.account_fence.is_none());
        assert!(ordinary_init.capabilities.iter().all(|cap| cap != "guarded_remote_sessions_v1"));
        let ordinary_fence = ordinary_init.fence.unwrap();
        assert_eq!(ordinary_fence.service_boot_id, project_fence.service_boot_id);
        let managed_response::Result::ProjectOpened(ordinary_project) = ordinary
            .call(managed_request::Operation::ProjectOpen(ProjectOpen {
                fence: Some(ordinary_fence),
                root: project.to_str().unwrap().into(),
            }))
            .await else { panic!("Ordinary project open required") };
        let denied = ordinary.call(managed_request::Operation::TerminalControl(TerminalControl {
            fence: ordinary_project.fence,
            session_id: first.session_id.clone(),
            run_id: first.run_id.clone(),
            action: TerminalAction::TerminalAttach as i32,
            ..Default::default()
        })).await;
        assert!(matches!(denied, managed_response::Result::Error(error)
            if error.code == ManagedErrorCode::ManagedFeatureUnavailable as i32));

        let mismatched = client.call(managed_request::Operation::AccountCatalogWatch(
            AccountCatalogWatch {
                fence: Some(account.clone()),
                filter: Some(AccountCatalogFilter {
                    scope_id: "ssh".to_owned(),
                    project_id: project_id.clone(),
                }),
                cursor: baseline_next.clone(),
                limit: 16,
            }
        )).await;
        assert!(matches!(mismatched, managed_response::Result::Error(error)
            if error.code == ManagedErrorCode::ManagedConflict as i32));

        let (bridge, _, _) = wait_route(&runtime).await;
        let mut bridge = ManagedClient::new(bridge);
        let managed_response::Result::Initialized(bridge_init) = bridge
            .call(managed_request::Operation::Initialize(ManagedInitialize { protocol_major: 1 }))
            .await else { panic!("Bridge initialize required") };
        let denied = bridge.call(managed_request::Operation::AccountStop(AccountStop {
            fence: bridge_init.account_fence,
            target: None,
            project_id: String::new(),
        })).await;
        assert!(matches!(denied, managed_response::Result::Error(error)
            if error.code == ManagedErrorCode::ManagedPermissionDenied as i32));

        let rejected = client.call(managed_request::Operation::AccountStop(AccountStop {
            fence: Some(account.clone()),
            target: Some(AccountStopTarget::default()),
            project_id: project_id.clone(),
        })).await;
        assert!(matches!(rejected, managed_response::Result::Error(error)
            if error.code == ManagedErrorCode::ManagedInvalidInput as i32));
        let request_id = Uuid::new_v4();
        let operation = managed_request::Operation::AccountStop(AccountStop {
            fence: Some(account.clone()),
            target: None,
            project_id: project_id.clone(),
        });
        let stopped = match client
            .call_with_id(request_id, operation.clone())
            .await
        {
            managed_response::Result::AccountStopReceipts(receipts) => receipts,
            _ => panic!("Account stop required"),
        };
        assert_eq!(stopped.status, AccountStopStatus::Partial as i32);
        assert_eq!(stopped.receipts.len(), 2);
        assert!(stopped.receipts.iter().all(|receipt| {
            receipt.outcome == AccountStopOutcome::Requested as i32
                && receipt.exit_status.is_none()
        }));

        let third = launch_guarded(&mut client, &project_fence, Uuid::new_v4().to_string()).await;
        let replay = client.call_with_id(request_id, operation).await;
        assert_eq!(replay, managed_response::Result::AccountStopReceipts(stopped));
        assert_eq!(inspect_guarded(&mut client, &account, &third).await.state, "live");

        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let row = inspect_guarded(&mut client, &account, &first).await;
            if row.state == "ended" {
                assert!(row.exit_status.is_some());
                break;
            }
            assert!(Instant::now() < deadline, "Stop was not observed");
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let observed = match client
            .call(managed_request::Operation::AccountStop(AccountStop {
                fence: Some(account.clone()),
                target: Some(AccountStopTarget {
                    session_id: second.session_id.clone(),
                    run_id: second.run_id.clone(),
                }),
                project_id: String::new(),
            }))
            .await
        {
            managed_response::Result::AccountStopReceipts(receipts) => receipts,
            _ => panic!("Account stop required"),
        };
        assert_eq!(observed.status, AccountStopStatus::Completed as i32);
        assert_eq!(observed.receipts.len(), 1);
        assert_eq!(
            observed.receipts[0].outcome,
            AccountStopOutcome::Observed as i32
        );
        assert!(observed.receipts[0].exit_status.is_some());

        let stale = client
            .call(managed_request::Operation::AccountCatalogWatch(
                AccountCatalogWatch {
                    fence: Some(account),
                    filter: None,
                    cursor: format!("{baseline_next}-stale"),
                    limit: 16,
                },
            ))
            .await;
        assert!(matches!(
            stale,
            managed_response::Result::Error(error)
                if error.code == ManagedErrorCode::ManagedConflict as i32
        ));
        server.abort();
    }

    async fn launch_guarded(
        client: &mut ManagedClient,
        fence: &ManagedFence,
        session_id: String,
    ) -> TerminalState {
        match client
            .call(managed_request::Operation::TerminalLaunch(
                TerminalLaunch {
                    fence: Some(fence.clone()),
                    session_id,
                    executable: "/bin/sh".into(),
                    arguments: vec!["-c".into(), "sleep 30".into()],
                    columns: 80,
                    rows: 24,
                    agent_program: None,
                    working_directory: None,
                    guarded: true,
                    label: None,
                    session_kind: Some(TerminalSessionKind::Shell as i32),
                },
            ))
            .await
        {
            managed_response::Result::TerminalState(state) => state,
            other => panic!("Guarded launch required: {other:?}"),
        }
    }

    async fn inspect_guarded(
        client: &mut ManagedClient,
        fence: &AccountFence,
        run: &TerminalState,
    ) -> AccountCatalogRow {
        match client
            .call(managed_request::Operation::AccountCatalogInspect(
                AccountCatalogInspect {
                    fence: Some(fence.clone()),
                    session_id: run.session_id.clone(),
                    run_id: run.run_id.clone(),
                },
            ))
            .await
        {
            managed_response::Result::AccountCatalogRow(row) => row,
            _ => panic!("Catalog row required"),
        }
    }

    struct ManagedClient {
        reader: tokio_util::compat::Compat<tokio::io::ReadHalf<UnixStream>>,
        writer: tokio_util::compat::Compat<tokio::io::WriteHalf<UnixStream>>,
    }

    impl ManagedClient {
        fn new(stream: UnixStream) -> Self {
            use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
            let (reader, writer) = tokio::io::split(stream);
            Self {
                reader: reader.compat(),
                writer: writer.compat_write(),
            }
        }

        async fn call(
            &mut self,
            operation: managed_request::Operation,
        ) -> managed_response::Result {
            self.call_with_id(Uuid::new_v4(), operation).await
        }

        async fn call_with_id(
            &mut self,
            request_id: Uuid,
            operation: managed_request::Operation,
        ) -> managed_response::Result {
            use remote_protocol::protocol::{
                read_message_with_limit, write_message_with_limit, MAX_MANAGED_MESSAGE_SIZE,
            };
            let request = ClientMessage {
                request_id: request_id.to_string(),
                message: Some(client_message::Message::Managed(ManagedRequest {
                    operation: Some(operation),
                })),
            };
            write_message_with_limit(&mut self.writer, &request, MAX_MANAGED_MESSAGE_SIZE)
                .await
                .unwrap();
            let reply = read_message_with_limit::<ServerMessage>(
                &mut self.reader,
                MAX_MANAGED_MESSAGE_SIZE,
            )
            .await
            .unwrap();
            let Some(server_message::Message::Managed(reply)) = reply.message else {
                panic!("Managed reply required");
            };
            reply.result.unwrap()
        }
    }

    async fn wait_route(runtime: &Path) -> (UnixStream, Uuid, Uuid) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Ok(probe) = probe_route(runtime, Route::Ssh).await {
                return probe;
            }
            assert!(Instant::now() < deadline, "Guardian did not start");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn guarded_sessions_reconnect_and_relist_without_client_reads() {
        let state = tempfile::tempdir().unwrap();
        let root = state.path().join("guardian");
        let runtime = state.path().join("run");
        let project = state.path().join("project");
        std::fs::create_dir_all(&project).unwrap();
        let server = {
            let root = root.clone();
            let runtime = runtime.clone();
            let scopes = vec![(Route::Ssh, state.path().join("ssh"))];
            tokio::spawn(async move { serve_at(root, runtime, scopes).await })
        };
        let (stream, guardian_id, boot_id) = wait_route(&runtime).await;
        let mut client = ManagedClient::new(stream);
        let initialized = match client
            .call(managed_request::Operation::Initialize(ManagedInitialize {
                protocol_major: 1,
            }))
            .await
        {
            managed_response::Result::Initialized(initialized) => initialized,
            _ => panic!("Guardian initialize required"),
        };
        let account = initialized
            .account_fence
            .clone()
            .expect("Guardian advertises account custody");
        assert_eq!(account.guardian_id, guardian_id.to_string());
        assert_eq!(account.boot_id, boot_id.to_string());
        let fence = initialized.fence.clone().unwrap();
        let opened = match client
            .call(managed_request::Operation::ProjectOpen(ProjectOpen {
                fence: Some(fence),
                root: project.to_str().unwrap().into(),
            }))
            .await
        {
            managed_response::Result::ProjectOpened(opened) => opened,
            _ => panic!("Project open required"),
        };
        let project_fence = opened.fence.unwrap();
        let launched = match client
            .call(managed_request::Operation::TerminalLaunch(
                TerminalLaunch {
                    fence: Some(project_fence.clone()),
                    session_id: Uuid::new_v4().to_string(),
                    executable: "/bin/sh".into(),
                    arguments: vec!["-c".into(), "printf READY; sleep 30".into()],
                    columns: 80,
                    rows: 24,
                    agent_program: None,
                    working_directory: None,
                    guarded: true,
                    label: Some("fixture".into()),
                    session_kind: Some(TerminalSessionKind::Shell as i32),
                },
            ))
            .await
        {
            managed_response::Result::TerminalState(state) => state,
            _ => panic!("Guarded launch required"),
        };
        assert_eq!(launched.guarded, Some(true));
        let page = match client
            .call(managed_request::Operation::AccountCatalogList(
                AccountCatalogList {
                    fence: Some(account.clone()),
                    filter: Some(AccountCatalogFilter {
                        scope_id: String::new(),
                        project_id: project_fence.project_id.clone(),
                    }),
                    limit: 16,
                    cursor: String::new(),
                },
            ))
            .await
        {
            managed_response::Result::AccountCatalogPage(page) => page,
            _ => panic!("Catalog page required"),
        };
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0].session_id, launched.session_id);
        assert_eq!(page.rows[0].state, "live");
        drop(client);

        let (stream, second_guardian, second_boot) = wait_route(&runtime).await;
        assert_eq!(second_guardian, guardian_id);
        assert_eq!(second_boot, boot_id);
        let mut client = ManagedClient::new(stream);
        let initialized = match client
            .call(managed_request::Operation::Initialize(ManagedInitialize {
                protocol_major: 1,
            }))
            .await
        {
            managed_response::Result::Initialized(initialized) => initialized,
            _ => panic!("Reconnect initialize required"),
        };
        let fence = initialized.fence.clone().unwrap();
        let second_account = initialized
            .account_fence
            .clone()
            .expect("Guardian advertises account custody");
        let reopened = match client
            .call(managed_request::Operation::ProjectOpen(ProjectOpen {
                fence: Some(fence),
                root: project.to_str().unwrap().into(),
            }))
            .await
        {
            managed_response::Result::ProjectOpened(opened) => opened,
            _ => panic!("Reconnect project open required"),
        };
        let second_fence = reopened.fence.unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        let attached = loop {
            let control = TerminalControl {
                fence: Some(second_fence.clone()),
                session_id: launched.session_id.clone(),
                run_id: launched.run_id.clone(),
                action: TerminalAction::TerminalAttach as i32,
                attachment_id: launched.attachment_id.clone(),
                attachment_generation: launched.attachment_generation,
                ..Default::default()
            };
            match client
                .call(managed_request::Operation::TerminalControl(control))
                .await
            {
                managed_response::Result::TerminalState(state) => break state,
                managed_response::Result::Error(error)
                    if error.code
                        == ManagedErrorCode::ManagedConflict as i32
                        || error.code == ManagedErrorCode::ManagedStaleAttachment as i32 =>
                {
                    assert!(Instant::now() < deadline, "Attach did not reconcile");
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                _ => panic!("Attach required"),
            }
        };
        assert_eq!(attached.run_id, launched.run_id);
        assert!(matches!(
            client
                .call(managed_request::Operation::TerminalControl(
                    TerminalControl {
                        fence: Some(second_fence),
                        session_id: launched.session_id.clone(),
                        run_id: launched.run_id.clone(),
                        action: TerminalAction::TerminalStop as i32,
                        attachment_id: attached.attachment_id.clone(),
                        attachment_generation: attached.attachment_generation,
                        ..Default::default()
                    }
                ))
                .await,
            managed_response::Result::TerminalState(_)
        ));
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let row = match client
                .call(managed_request::Operation::AccountCatalogInspect(
                    AccountCatalogInspect {
                        fence: Some(second_account.clone()),
                        session_id: launched.session_id.clone(),
                        run_id: launched.run_id.clone(),
                    },
                ))
                .await
            {
                managed_response::Result::AccountCatalogRow(row) => row,
                _ => panic!("Catalog row required"),
            };
            if row.state == "ended" {
                assert!(row.exit_status.is_some());
                break;
            }
            assert!(
                Instant::now() < deadline,
                "Guardian did not observe natural end without reads"
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        server.abort();
    }
}
