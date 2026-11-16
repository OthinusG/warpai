# Antigravity usage subprocess networking

Date: 2026-11-16

## Problem and scope

The desktop's native HTTP transport follows system proxies, but `agy /usage`
is a subprocess with vendor-owned networking. A GUI launch without shell proxy
variables can time out even when the same command succeeds in a terminal.

Resolve the system route for `https://cloudcode-pa.googleapis.com/` before
starting the usage command. Use CFNetwork (including PAC) on macOS, WinHTTP
automatic proxy resolution on Windows, and the existing GIO resolver on Linux.
Export the selected route through child-only proxy variables. Keep localhost
communication direct. Do not change global environment, settings, credentials,
interactive CLI sessions, Qoder queries, or the version probe.

The child environment can represent one route, so the usage API's first system
route applies to the entire invocation, including auxiliary HTTPS requests.
It cannot represent a PAC policy that assigns different proxies per auxiliary
host or proxy failover. Do not claim per-request equivalence with native HTTP.

## Acceptance and implementation tasks

- Add a bounded native proxy resolver in `system_http`; return explicit direct,
  a validated HTTP/HTTPS/SOCKS5 proxy URI, or a sanitized error. Never treat PAC
  failure or unsupported proxy types as direct. Do not expose proxy credentials.
- Resolve outside the UI thread, with a deadline; inject uppercase/lowercase
  variables only into the usage child. Clear conflicting inherited variables,
  preserve local loopback bypass, and keep existing process/output deadlines.
- Test environment scoping, direct routing, malformed/unsupported routes and
  URI validation. Extend disposable three-platform native proxy acceptance to
  exercise an actual subprocess, PAC, static proxies and bypass.
- Local checks: native syntax, script syntax and focused source review. Rust
  tests and both app configurations run on GitHub, per project build policy.

## Risks

Native callback lifetime and timeout ownership; stale inherited proxy variables;
PAC results mistaken for proxy addresses; bypass behavior; accidentally logging
credentials; vendor CLI proxy behavior changing between versions. Keep all
network and environment changes local to this query.

Linux PAC requires the distribution's libproxy PAC backend in addition to
glib-networking (Debian/Ubuntu: libproxy1-plugin-webkit). WinHTTP's SOCKS result
represents SOCKS4, which agy cannot use; reject it instead of relabeling it SOCKS5.

References: [CFNetwork PAC](https://developer.apple.com/documentation/cfnetwork/cfnetworkexecuteproxyautoconfigurationurl(_:_:_:_:)),
[WinHTTP resolver](https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpgetproxyforurlex),
[GIO resolver](https://docs.gtk.org/gio/method.ProxyResolver.lookup.html).
