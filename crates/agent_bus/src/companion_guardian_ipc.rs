//! Shared framing for the private native guardian transports.
use super::Route;
use std::{io::ErrorKind, time::Duration};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

pub const IPC_MAJOR: u16 = 1;
const MAGIC: [u8; 4] = *b"WAG1";
const PREFACE_BYTES: usize = 4 + 2 + 1 + 1 + 16 + 16;
pub(super) const REPLY_BUSY: u8 = 1;

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

}

pub(super) fn preface(route: Route, guardian: Uuid, boot: Uuid) -> [u8; PREFACE_BYTES] {
    let mut bytes = [0; PREFACE_BYTES];
    bytes[..4].copy_from_slice(&MAGIC);
    bytes[4..6].copy_from_slice(&IPC_MAJOR.to_be_bytes());
    bytes[6] = route.code();
    bytes[8..24].copy_from_slice(guardian.as_bytes());
    bytes[24..40].copy_from_slice(boot.as_bytes());
    bytes
}

pub(super) fn request_preface(route: Route) -> [u8; PREFACE_BYTES] {
    preface(route, Uuid::nil(), Uuid::nil())
}

pub(super) fn valid_reply(bytes: &[u8; PREFACE_BYTES], route: Route) -> Option<(Uuid, Uuid)> {
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
pub(super) enum Admission {
    Guarded = 0,
    Operator = 1,
    OperatorSession = 3,
}

pub(super) async fn read_preface(stream: &mut (impl AsyncRead + Unpin), route: Route) -> std::io::Result<Admission> {
    let mut bytes = [0; PREFACE_BYTES];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut bytes))
        .await
        .map_err(|_| std::io::Error::from(ErrorKind::TimedOut))??;
    if bytes[..4] != MAGIC
        || u16::from_be_bytes([bytes[4], bytes[5]]) != IPC_MAJOR
        || Route::from_code(bytes[6]) != Some(route)
        || !matches!(bytes[7], 0 | 1 | 3)
        || bytes[8..].iter().any(|byte| *byte != 0)
    {
        return Err(std::io::Error::from(ErrorKind::InvalidData));
    }
    Ok(match bytes[7] {
        0 => Admission::Guarded,
        1 => Admission::Operator,
        _ => Admission::OperatorSession,
    })
}

pub(super) async fn exchange_preface<T: AsyncRead + AsyncWrite + Unpin>(
    mut stream: T,
    route: Route,
    admission: Admission,
) -> std::io::Result<(T, Uuid, Uuid)> {
    let mut request = request_preface(route);
    request[7] = admission as u8;
    stream.write_all(&request).await?;
    let mut bytes = [0; PREFACE_BYTES];
    tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut bytes))
        .await
        .map_err(|_| std::io::Error::from(ErrorKind::TimedOut))??;
    if bytes[7] == REPLY_BUSY {
        bytes[7] = 0;
        if valid_reply(&bytes, route).is_some() {
            return Err(std::io::Error::from(ErrorKind::WouldBlock));
        }
        return Err(std::io::Error::from(ErrorKind::InvalidData));
    }
    let (guardian, boot) =
        valid_reply(&bytes, route).ok_or_else(|| std::io::Error::from(ErrorKind::InvalidData))?;
    Ok((stream, guardian, boot))
}
