//! Child-only proxy routing for vendor CLIs that cannot consume OS proxy settings.
use std::{
    ffi::{CStr, CString, c_char},
    process::Command,
    time::Duration,
};

use futures::{
    channel::oneshot,
    future::{Either, select},
};

unsafe extern "C" {
    fn warpai_proxy_for_url(url: *const c_char, result: *mut c_char, capacity: usize) -> i32;
}

const FAILURE: &str = "Could not resolve system proxy for CLI usage";
const PROXY_VARIABLES: [&str; 8] = [
    "HTTP_PROXY",
    "http_proxy",
    "HTTPS_PROXY",
    "https_proxy",
    "ALL_PROXY",
    "all_proxy",
    "NO_PROXY",
    "no_proxy",
];

/// An explicit direct route or the first proxy selected by the operating system.
/// Intentionally has no Debug implementation: proxy URIs may contain credentials.
pub struct SubprocessProxy(Option<String>);

impl SubprocessProxy {
    /// Resolve off the caller's executor without reading provider credentials or response bodies.
    pub async fn resolve(target: &str) -> Result<Self, &'static str> {
        let target = reqwest::Url::parse(target).map_err(|_| FAILURE)?;
        if !matches!(target.scheme(), "http" | "https")
            || target.host_str().is_none()
            || !target.username().is_empty()
            || target.password().is_some()
        {
            return Err(FAILURE);
        }
        let target = CString::new(target.as_str()).map_err(|_| FAILURE)?;
        let (tx, rx) = oneshot::channel();
        std::thread::Builder::new()
            .name("cli-proxy".into())
            .spawn(move || {
                let mut buffer = [0 as c_char; 8192];
                let result = unsafe {
                    warpai_proxy_for_url(target.as_ptr(), buffer.as_mut_ptr(), buffer.len())
                };
                let route = if result == 0 {
                    // Native implementations always terminate a successful result within capacity.
                    unsafe { CStr::from_ptr(buffer.as_ptr()) }
                        .to_str()
                        .map_err(|_| FAILURE)
                        .and_then(Self::parse)
                } else {
                    Err(FAILURE)
                };
                let _ = tx.send(route);
            })
            .map_err(|_| FAILURE)?;
        match select(
            rx,
            Box::pin(futures_timer::Delay::new(Duration::from_secs(6))),
        )
        .await
        {
            Either::Left((result, _)) => result.map_err(|_| FAILURE)?,
            Either::Right(_) => Err("System proxy lookup timed out for CLI usage"),
        }
    }

    fn parse(route: &str) -> Result<Self, &'static str> {
        if route == "direct://" {
            return Ok(Self(None));
        }
        if route.chars().any(char::is_control) {
            return Err(FAILURE);
        }
        let uri = reqwest::Url::parse(route).map_err(|_| FAILURE)?;
        if !matches!(uri.scheme(), "http" | "https" | "socks5" | "socks5h")
            || uri.host_str().is_none()
            || uri.port_or_known_default().is_none_or(|port| port == 0)
            || uri.path() != "/" && !uri.path().is_empty()
            || uri.query().is_some()
            || uri.fragment().is_some()
        {
            return Err(FAILURE);
        }
        Ok(Self(Some(uri.to_string())))
    }

    /// Apply only to a new child command; never mutate the application's environment.
    pub fn apply(&self, command: &mut Command) {
        for name in PROXY_VARIABLES {
            command.env_remove(name);
        }
        if let Some(proxy) = &self.0 {
            for name in &PROXY_VARIABLES[..6] {
                command.env(name, proxy);
            }
        }
        // agy's local language server must remain reachable regardless of inherited bypass rules.
        for name in &PROXY_VARIABLES[6..] {
            command.env(name, "localhost,127.0.0.1,::1");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_routes_without_exposing_them() {
        for route in [
            "",
            "file:///tmp/proxy.pac",
            "socks4://localhost:1080",
            "http://localhost/path",
            "http://localhost/?secret=example",
            "http://localhost\n",
            "https://",
            "http://localhost:0",
            "socks5://localhost",
        ] {
            assert!(matches!(SubprocessProxy::parse(route), Err(FAILURE)));
        }
        for route in [
            "http://localhost:8080",
            "https://[::1]:8443",
            "socks5://localhost:1080",
            "socks5h://localhost:1080",
            "http://user:example@localhost:8080",
        ] {
            assert!(SubprocessProxy::parse(route).is_ok());
        }
    }

    #[test]
    fn proxy_and_direct_override_conflicting_child_environment_only() {
        let mut command = Command::new("synthetic-cli");
        for name in PROXY_VARIABLES {
            command.env(name, "stale");
        }
        let proxy = SubprocessProxy::parse("http://localhost:8080").unwrap();
        proxy.apply(&mut command);
        let vars: std::collections::HashMap<_, _> = command
            .get_envs()
            .map(|(name, value)| (name.to_string_lossy().to_ascii_lowercase(), value))
            .collect();
        for name in &PROXY_VARIABLES[..6] {
            assert_eq!(
                vars[&name.to_ascii_lowercase()],
                Some(std::ffi::OsStr::new("http://localhost:8080/"))
            );
        }
        SubprocessProxy::parse("direct://")
            .unwrap()
            .apply(&mut command);
        let vars: std::collections::HashMap<_, _> = command
            .get_envs()
            .map(|(name, value)| (name.to_string_lossy().to_ascii_lowercase(), value))
            .collect();
        for name in &PROXY_VARIABLES[..6] {
            assert!(vars[&name.to_ascii_lowercase()].is_none());
        }
        assert_eq!(
            vars["no_proxy"],
            Some(std::ffi::OsStr::new("localhost,127.0.0.1,::1"))
        );
        assert!(Command::new("unaffected").get_envs().next().is_none());
    }
}
