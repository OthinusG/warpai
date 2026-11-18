# S0 baseline, caller inventory and contract freeze

Date: 2026-11-18

Baseline: `db280c6b57c3eab7acb8c8e04ab8c8aab084828a` (repository HEAD before and
after this slice; the slice itself is uncommitted). Planned documents under
`specs/2.0.0/` and the owner note in `MEMORY.md` were already uncommitted.

This record covers only what was actually inspected or changed in source. It does
not claim native, protocol or resource verification; see "Verification" below.

## Ownership map at baseline

| Concern | Source | Baseline behavior |
| --- | --- | --- |
| Detached account service | `crates/agent_bus/src/companion_service.rs` | `--service` sets `setsid`/`DETACHED_PROCESS`, `service.lock` exclusive owner, private endpoint, 32 slots, 60 s idle exit |
| Per-connection state | `crates/agent_bus/src/companion.rs` | `Companion` holds fence, project handle, shared `Terminals`/`Projects`; `Drop` calls `terminals.disconnect` |
| Managed terminal registry | `crates/agent_bus/src/companion_terminals.rs` | 32 sessions, 256 KiB ring, 32 KiB pages, connection-owned PTY, reader/screen/wake/writer threads |
| Native process ownership | `crates/agent_bus/src/native_pty_unix.rs`, `native_pty_windows.rs` | Unix process group/SIGKILL on drop; Windows ConPTY + kill-on-job-close |
| Screen/readiness evidence | `crates/agent_bus/src/screen_detection/native.rs` | Memory-only vt100 state used by 300 ms Agent wake loop |
| SSH/WSL protocol client | `crates/agent_bus/src/ssh_remote.rs` | `HostClient` fences project, validates `TerminalState`, negotiates capability strings |
| Desktop callers | `app/src/agent_communication/panel.rs` | Constructs connection-owned `TerminalLaunch`; no guarded fields |
| Remote CLI host | `crates/agent_bus/src/companion_main.rs` | `--bound-agent`/`agent` foreground PTY proxy; no `ctl` dispatcher |
| WSL guest | `crates/agent_bus/src/wsl_companion_main.rs`, `companion.rs` | Dedicated Linux binary sharing the account service with `wsl_companion` feature |
| Protocol types | `crates/remote_protocol/proto/remote_server.proto`, `src/managed.rs` | Protocol major 1; `TerminalLaunch`/`Control`/`State` connection-owned |

Consequences confirmed at baseline:

- One service instance exists per data directory, not per execution OS account across
  distinct helper binaries. SSH Companion and `warpai-wsl-companion` have separate
  data directories/features; no account-global election or shared catalog exists.
- Agent run bindings and wake threads are owned by `Terminals`, which lives in the
  service, so a guarded native run *can* outlive a client connection once the
  connection-owned stop path is bypassed.
- No account fence, catalog/list/inspect, snapshot or CLI dispatcher exists.

## Wire freeze implemented in this slice (additive, major 1)

| Message | Field | Number | Meaning |
| --- | --- | --- | --- |
| `TerminalLaunch` | `guarded` | 9 | Immutable launch policy; true retains the native run across connection loss |
| `TerminalLaunch` | `label` | 10 | Nonsecret display selector, ≤128 bytes, no control characters |
| `TerminalAction` | `TERMINAL_ATTACH` | 8 | Acquire the single writable attachment |
| `TerminalAction` | `TERMINAL_DETACH` | 9 | Release writer lease, keep native run |
| `TerminalControl` | `attachment_id` | 11 | Runtime-issued writer identity (UUID) |
| `TerminalControl` | `attachment_generation` | 12 | Monotonic writer epoch |
| `TerminalControl` | `takeover` | 13 | Explicit fencing of a conflicted writer |
| `TerminalState` | `guarded` | 16 | Recorded launch policy |
| `TerminalState` | `label` | 17 | Recorded display label |
| `TerminalState` | `attachment_generation` | 18 | Current writer epoch |
| `TerminalState` | `attachment_id` | 19 | Current writer identity |

Capability name reserved by API.md: `guarded_remote_sessions_v1`. It is **not**
advertised by the service and the client rejects guarded fields without it, because
no native proof has run.

## Explicitly not implemented by this slice

Account-global election and logical scopes; account fence/catalog/list/inspect and
watch cursors; MRU resume; `ctl list/resume/stop`, help/version, session-only shell
function; paged snapshots and gap resnapshot; bounded ended-session retention;
desktop opt-in UI/reconnect; event-driven wake scheduler; WSL native lifetime proof;
packaging/version changes; resource measurements; V01-V30.

## Verification

- Source edits were statically inspected against the surrounding code only.
- No local `cargo check`, `cargo test`, formatter or linter was run (owner policy:
  Rust builds and native acceptance run in GitHub/cloud).
- No CI run, workflow dispatch, commit, push or release was performed.
- Status of the code slice: `implemented_unverified`.
