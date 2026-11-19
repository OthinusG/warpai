//! Windows guardian election and same-account named-pipe admission.
use super::{
    guardian_ipc::{exchange_preface, Admission},
    service, PathBuf, ProtocolError, Route, Uuid,
};
use crate::transport::windows_pipe;
use std::{io, os::windows::io::AsRawHandle, time::Duration};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use windows::Win32::{
    Foundation::{ERROR_PIPE_BUSY, HANDLE},
    System::Pipes::GetNamedPipeServerProcessId,
};

pub use super::guardian_ipc::IPC_MAJOR;

pub(super) async fn scope_directory(route: Route) -> io::Result<PathBuf> {
    if route != Route::Ssh {
        return Err(io::ErrorKind::Unsupported.into());
    }
    Ok(dirs::data_local_dir().ok_or(io::ErrorKind::NotFound)?
        .join("warpai").join("guardian-scopes").join("ssh"))
}

pub(super) fn endpoint() -> io::Result<String> {
    let sid = windows_pipe::current_user_sid().map_err(io::Error::other)?;
    Ok(format!(r"\\.\pipe\warpai-guardian-{sid}"))
}

async fn probe_as(route: Route, admission: Admission) -> io::Result<(NamedPipeClient, Uuid, Uuid)> {
    if route != Route::Ssh {
        return Err(io::ErrorKind::Unsupported.into());
    }
    let endpoint = endpoint()?;
    let stream = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match ClientOptions::new().open(&endpoint) {
                Err(error) if error.raw_os_error() == Some(ERROR_PIPE_BUSY.0 as i32) => {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
                result => return result,
            }
        }
    })
    .await
    .map_err(|_| io::Error::from(io::ErrorKind::WouldBlock))??;
    let mut pid = 0;
    unsafe { GetNamedPipeServerProcessId(HANDLE(stream.as_raw_handle()), &mut pid) }
        .map_err(io::Error::other)?;
    let current = windows_pipe::current_user_sid().map_err(io::Error::other)?;
    if windows_pipe::client_user_sid(pid).map_err(io::Error::other)? != current {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    exchange_preface(stream, route, admission).await
}

pub(crate) async fn probe(route: Route, attach: bool) -> io::Result<(NamedPipeClient, Uuid, Uuid)> {
    probe_as(route, if attach { Admission::OperatorSession } else { Admission::Operator }).await
}

pub(crate) async fn connect(route: Route) -> io::Result<(NamedPipeClient, Uuid, Uuid)> {
    match probe_as(route, Admission::Guarded).await {
        Err(error) if error.kind() == io::ErrorKind::NotFound => (),
        result => return result,
    }
    crate::native_pty::start_windows_guardian()?;
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match probe_as(route, Admission::Guarded).await {
                Err(error) if error.kind() == io::ErrorKind::NotFound => (),
                result => return result,
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))?
}

pub(super) async fn serve() -> Result<(), ProtocolError> {
    crate::native_pty::verify_windows_guardian_job()?;
    service::serve_windows_guardian().await
}
