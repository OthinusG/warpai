# SSH native parity and Windows shell audit

Date: 2026-10-19. Target: 1.7.6; Companion 5.5.0 / SSH major 1.

## Owner requirements and acceptance

- Local, direct WSL and native SSH share readiness observation and guarded wake
  delivery. SSH's default-off master gates all Companion operations; native SSH
  terminal access remains independent. Retain separate WSL/local preferences.
- Existing SSH tab evidence remains authoritative. Panel and MCP must recognize
  initial readiness without requiring the first vendor conversation.
- Native remote Windows defaults to PowerShell. Do not send POSIX bootstrap or
  add CMD compatibility. Preserve POSIX hosts and configured RemoteCommand.
- Fix the Windows validation supervisor and retain real direct, ProxyJump, SFTP,
  master-revocation, native UI, locale and WSL acceptance gates.
- Dispatch all required cloud validations, then pause. Merge main and release
  1.7.6 only after acceptance and resuming work; no publication during the pause.

## Audit method and findings

Enumerated Windows compile/runtime markers throughout `app/src` and
`crates/agent_bus/src` (143 matching files before this patch), then traced the
terminal bootstrap, confirmed session selection, MCP setup, Agent observation,
input delivery, file/Review and native transport paths. Included shared shell
code in `crates/warp_terminal`. Local OS primitives remain compiled for the OS
which executes them. Remote behavior must derive from authenticated session
metadata and negotiated capabilities.

| Path | Decision and user effect |
| --- | --- |
| `terminal/model/session`, InitShell parser and bootstrap | Keep shell PID on every desktop; report it from actual inline SSH Bash/Zsh payloads as well as initial/subshell assets. |
| `terminal/view/wsl_peer`, AgentCommunication scheduler | Share observation and guarded claim/validate/finish delivery with SSH; keep WSL enumeration Windows-only. Do not project stale remote Starting onto correct SSH tabs. |
| `agent_bus/companion` capabilities | Advertise `native_screen_activity` in ordinary Companion, not only WSL builds. Otherwise the desktop strips evidence before sending the request. |
| `terminal/view/use_agent_footer` | Shared staging helper must compile on every desktop. Existing local raw draft clearing remains local/WSL; SSH preserves drafts. |
| `bootstrap/{bash_body,zsh_body,fish}` | Authenticate and probe before selecting remote grammar. Use native PowerShell seed on Windows and preserve POSIX and explicit RemoteCommand paths. |
| `terminal/bootstrap`, `writeable_pty/pty_controller` | Stage trusted PowerShell content in short native commands on the remote host; a local bootstrap file is unavailable there. Clean up the owned remote file. |
| `terminal/model/session`, `warp_terminal/shell` | Derive PowerShell separators and executable suffix aliases from remote OS, not client compile target. PowerShell is a supported remote shell. |
| `agent_communication` native start observer | Reject confirmed SSH master/legacy sessions as local registrations even without retained SSH arguments. |
| `remote_server/selected_session`, SSH settings, Codex launch | Remote shell, home, PATH, selected account and Companion path already derive from remote metadata. WSL distribution/user discovery stays Windows-only. Native SSH Codex route is shared. |
| `agent_bus/{companion_identity,companion_guests,companion_service,transport,session,native_pty}` | Keep Windows token, process birth, named pipe, service and ConPTY APIs. They execute on the native host/Companion, not according to the other endpoint's OS. |
| `agent_bus/{companion_files,storage,setup,installation,ssh_remote,ssh_files}` | Keep local native ACL, symlink, process flag, executable and file-handle differences. The remote Companion executes remote filesystem operations; SSH/SFTP grammar already follows RemoteShell. Fixture-only host OS choices describe same-host loopback servers. |
| `terminal/{local_tty,available_shells,warpify/settings}`, bootstrap-file implementation | Keep local PTY/spawn, shell discovery, MSYS2, direct WSL and temporary-file primitives. The new remote PowerShell path bypasses local temporary-file sourcing. |
| Editor, Review, file tree, UI/platform and application service markers | Keep client keyboard, clipboard, native dialogs, file-manager names, render/ConPTY reset handling, updater, OS integration and test fixtures. These belong to the running desktop, not the SSH host. No cloud/login/telemetry surfaces restored. |

The audit does not equate shared behavior with removing Windows FFI guards or
running Windows local services on another OS. Vendor-authenticated behavior on
an owner's Mac-to-Windows machine remains runtime acceptance, separate from
controlled native fixtures.

## Validation changes and evidence

Prior remote suite 39232252558 passed on Linux/macOS/Windows, including native
initial readiness and real guarded terminal wake. Full suite 39232770113 passed
macOS/protocol checks but failed Windows in `jump-tests` before launching the
capture app. The diagnostic recorded CalledProcessError, not TimeoutExpired.
The original historical empty capture error does not prove the same cause.

- Precompile SSH integration fixtures before timed runtime acceptance.
- Serialize ignored tests sharing the owned loopback server and ProxyJump route.
  Server admission contention is a hypothesis; do not report it as proven.
- Replace Windows Start-Process -Wait descendant-tree waiting with direct Python
  invocation and preserve its exit code. Kill/reap owned timed-out process trees.
- Publish only source-whitelisted failing Rust test names, fixed stage labels,
  exception classes and source locations; do not publish raw runtime/config logs.
- Keep cache save after acceptance/artifact upload, bounded and cancelable.

Local checks: actual Bash/Zsh wrapper routing with an owned SSH executable;
Windows/POSIX/RemoteCommand paths; Python supervisor timeout/nonzero behavior;
diagnostic redaction; Bash/Zsh syntax; workflow YAML. Fish runtime and Windows
PowerShell 5.1 run in cloud. No local Rust build or personal app/config changes.

Cloud gates: full macOS/Windows communication and three-locale UI + direct WSL;
Linux/macOS/Windows ordinary Companion/native MCP/PTY/file tools; Linux x64/ARM64
artifact-only desktop checks/package smoke. Focused regressions cover remote
PowerShell staging, native shell PID, host path semantics and completions.
