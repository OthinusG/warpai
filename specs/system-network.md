# System network transport

Date: 2026-11-03

## Problem and acceptance

Data usage incorrectly compares CFNetwork proxy type strings, discards the system
PAC answer, and connects directly. Update checks bypass that resolver entirely.
All desktop-owned HTTP requests must use one transport. On macOS, NSURLSession
must own proxy/PAC/direct routing and TLS; no proxy ports, PAC evaluation, or
proxy-type translation belong in the application. Windows uses WinHTTP automatic proxy mode. Linux uses libsoup 2.4 with the
GIO system proxy resolver (compatible with the glibc 2.31 baseline).

Preserve methods, sensitive request headers, response status/headers, streaming,
per-request deadlines, cancellation, and redirect policies. Usage and release
checks must still reject redirects and bound response sizes. Credentials remain
request-local; disable shared cookies, credential storage and disk caching.
Do not modify personal provider credentials, system settings or the installed app.

## Implementation

1. Add a shared `system_http` transport below the existing `http_client`, avoiding
   its UI/core dependency cycle for asset loading.
2. Use a shared ephemeral NSURLSession and delegate on macOS, WinHTTP on
   Windows, and libsoup/GIO on Linux. Bridge response
   headers and streaming chunks to Rust; cancel native tasks when futures/bodies
   are dropped. Apply backpressure without blocking the delegate queue.
3. Route usage, release checks, asset loading and the existing HTTP client/OAuth
   through that transport. Remove the bespoke PAC resolver.
4. Preserve the disabled cloud/telemetry surfaces and vendor-owned CLI networking.

## Verification and risks

Run source hygiene locally. Run pinned Rust tests/builds on GitHub as required by
MEMORY.md. Native loopback tests must cover methods/headers/body, streaming before
EOF, cancellation, deadlines, redirects and error propagation without credentials.
Check default and warp_platform desktop builds and non-macOS compatibility.
Verify public update/usage endpoints with no authentication using the native
transport and this machine's existing PAC. Unauthenticated HTTP 401 proves
connectivity only, not provider authentication or actual account quotas.

Native ownership, cancellation races, streaming memory use, redirected secrets,
and TLS trust are the main regression risks. Preserve release/download validation;
the current public updater opens installer links in the browser and does not
download or install packages itself.

## Inventory and verification receipts

- Routed: provider HTTP quotas, public release metadata, URL assets (including
  shared fallback resources), LSP/node downloads, the existing shared HTTP client
  and its streaming/OAuth adapter. HTTP request/response types remain compatible.
- Native CLI usage commands retain vendor-owned networking. SSH/SFTP, local
  sockets, terminal subprocesses and browser-opened installer links keep their
  existing transports. Retained built-in cloud AI/MCP scaffolding remains disabled
  in the local-first product; this change does not restore that product surface.
- macOS local credential-free Foundation probe: Codex endpoint HTTP 401 through
  the existing PAC; GitHub endpoint HTTP 403 (reachable; no API authentication).
- Disposable GitHub runners test request/body/status/header preservation,
  streaming before EOF, cancellation, upload staging, deadlines and redirect
  credential isolation. An additional fixture changes **only disposable hosted
  runner** proxy settings, clears shell proxy variables, and requires an
  otherwise-unresolvable target to arrive at the local proxy.

- Windows native desktop linking requires the same static C++ runtime as locked
  esaxx-rs (MT). system_http sets static_crt(true) on Windows; this does not change
  HTTP behavior, Rust target flags, or macOS/Linux build branches. The mismatch
  was exposed by native review linking in validation run 41680297211.
