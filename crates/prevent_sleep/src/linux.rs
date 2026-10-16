use std::os::fd::OwnedFd;

use zbus::blocking::{Connection, Proxy};

/// Holds logind's inhibitor descriptor for the lifetime of the guard.
pub struct Guard {
    inhibitor: Option<OwnedFd>,
    reason: &'static str,
}

impl Drop for Guard {
    fn drop(&mut self) {
        drop(self.inhibitor.take());
        log::info!("No longer preventing sleep with reason: {}", self.reason);
    }
}

/// Requests an idle and suspend inhibitor from systemd-logind.
pub fn prevent_sleep(reason: &'static str) -> Guard {
    let inhibitor = (|| {
        let connection = Connection::system()?;
        let proxy = Proxy::new(
            &connection,
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )?;
        proxy.call("Inhibit", &("idle:sleep", "Warpai", reason, "block"))
    })();

    match inhibitor {
        Ok(inhibitor) => {
            log::info!("Preventing sleep with reason: {reason}");
            Guard {
                inhibitor: Some(inhibitor),
                reason,
            }
        }
        Err(error) => {
            log::warn!("Could not inhibit system sleep: {error}");
            Guard {
                inhibitor: None,
                reason,
            }
        }
    }
}
