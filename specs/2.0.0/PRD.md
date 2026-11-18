# Warpai 2.0.0 product requirements

Status: R4 proposed scope. Actual work is recorded in [IMPLEMENTATION.md](IMPLEMENTATION.md).

## Product goal and non-disruption

Add an optional low-overhead guardian to existing SSH and WSL workflows. Users can
leave remote Agents, shells, tests and servers running after closing Warpai, then
return through Warpai or an SSH/WSL terminal. Existing workflows and native local
terminals retain their current semantics. WSL persistence is required, not optional.

Guard mode is `off` by default. A connection-profile default and per-launch choice
are presentation/configuration; the runtime's recorded per-session policy is
execution truth. A profile change affects future launches only. Turning the
preference off does not stop existing guarded work. Detach/stop remain explicit.
First delivery launches new guarded sessions; in-place conversion of an existing
unguarded PTY is excluded until exact ownership can be separately proven.

## User journeys

### J1: opt in and leave

On a confirmed SSH account or WSL distribution/user, enable `Keep sessions running`
for a new managed session. Launch an Agent or shell/command. Close the tab, SSH
bridge, WSL client or Warpai desktop. The guardian holds the original PTY/process.
No local daemon is launched, no terminal is globally converted, and no UI prompt
is required on every subsequent intentional detach. Offer a separate explicit
`Stop remote sessions and quit` action when the user wants termination.

### J2: return and take over

Connect through the same SSH host/account or WSL distro/user. List live sessions,
select a session and reattach after checking service boot/run/project identity.
The same process continues; no vendor CLI relaunch, task restart or authority
transfer is inferred. A plain SSH/WSL shell can use the explicit Companion CLI
attach flow to operate a selected PTY; the native Warpai UI can later reattach.
Only one writer exists per session; replacing one is an explicit takeover.

### J3: multiple sessions and tasks

Keep a Coordinator, two worker Agents, a test command and a server in separate
sessions. Close Warpai. Eligible workers continue exchanging task messages and
results; the test/server remain running. One noisy output stream or blocked Agent
must not freeze other sessions. A single Agent still accepts tracked work through
its existing serial recipient queue; parallel work uses distinct Agent sessions.

### J4: restore the remote workspace

On reconnect, refresh the session catalog and project/task facts, attach only
selected terminals, then reopen Explorer, editor and Review against the same
confirmed project. Unfinished client file transfers are not silently replayed;
unsaved editor drafts keep their existing client policy. Reconnection never
silently repeats a file mutation or accepts a task result on the user's behalf.

### J5: host unavailable or stopped

A network break or `wsl.exe` bridge exit is detach. A remote host reboot,
`wsl --terminate`, `wsl --shutdown`, Windows shutdown or runtime crash is execution
loss/uncertainty, not live continuation. Show interruption and existing guarded
recovery without automatically starting duplicate processes or inheriting rights.

## Guardian feature coverage

| Feature | With desktop absent | After reconnect |
| --- | --- | --- |
| Guarded managed PTY, Agent, shell, tests/server descendants | Continue native execution and drain output | Attach same run and bounded current/recent terminal state |
| Agent MCP, task pool, handoff, messages, supervision | Same-account/project work continues where Agents/readiness allow | Refresh authoritative task/history/attention |
| Existing Coordinator/worktree ownership | Retain exact live-run authority, registered checkout identity and guards | Display current facts; do not auto-integrate or broaden rights |
| Permission questions and user decisions | Remain blocked; no automatic approval | Surface pending questions/actions |
| Explorer, editor, code preview, Review UI | No hidden UI, watchers or rendering; retain authoritative remote data | Reopen lazily against confirmed project |
| SFTP / WSL transfers and in-flight file writes | Release/cancel connection-owned resources; preserve uncertainty receipts | Reconcile before retry, never replay uncertain writes |
| Usage/account UI polling, screenshots and notifications | No dashboard polling, framebuffer generation or notification delivery | Refresh through existing client/account workflow |
| Unguarded sessions and native local terminals | Existing lifecycle exactly | Existing workflow exactly |

Guardian preserves the execution/data plane required by enabled functions; it is
not a remote copy of the full GUI. No new cross-host coordination is introduced.

## Functional requirements

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-01 | Optional guardian, default off; future-launch profile preference and explicit session policy | P0 |
| FR-02 | Same-run discovery/attach from Warpai and explicit SSH/WSL CLI bridge | P0 |
| FR-03 | SSH guarded runs survive transport loss and desktop exit | P0 |
| FR-04 | WSL guarded runs survive bridge/desktop exit under the exact distro/user | P0 |
| FR-05 | Existing remote Broker/MCP/readiness/task delivery continue headlessly | P0 |
| FR-06 | Bounded terminal snapshot/tail preserves interactive behavior and shell blocks | P0 |
| FR-07 | Single writer per session, explicit takeover, stale input/resize rejected | P0 |
| FR-08 | Native stop evidence, interruption/unknown handling and bounded retention | P0 |
| FR-09 | Independent transport/process/attention state and truthful feature coverage | P0 |
| FR-10 | Existing local/unguarded/legacy-client behavior unchanged | P0 |
| FR-11 | Multiple sessions/projects; fair IO and serial per-Agent task semantics | P0 |
| FR-12 | Shared low-overhead guardian, measured CPU/memory and bounded caches/queues | P0 |
| FR-13 | Source-matched SSH/WSL payloads and compatible live-runtime upgrades | P0 |
| FR-14 | Lazy UI/file workflow restoration without uncertain mutation replay | P0 |
| FR-15 | One account guardian across Agent types/clients; logical scope isolation | P0 |
| FR-16 | Companion-private list/resume/stop commands with exact target resolution and no PATH changes | P0 |
| FR-17 | Automatic bounded catalog/status restoration and safe previous-view reattachment | P0 |
| FR-18 | Owner-specified MRU resume and immediate account-wide guarded stop defaults | P0 |
| FR-19 | Minimal CLI list/resume/stop plus standard help/version; extra tools require demonstrated unmet need | P0 |

## UX and operation rules

Use existing native session/sidebar and collaboration-panel surfaces. Label scope
(host/account or distro/user), actual session guard policy, attached/detached,
process state, pending work and evidence age. A preference is not proof an already
running session is guarded. Do not offer a successful enable toggle if the required
host capability is absent; keep the original path available for explicit use.

Closing a guarded view detaches; quitting the desktop preserves guarded sessions.
Show a concise retained-work summary without adding a mandatory quit dialog to
all users. Bare CLI stop immediately targets all current-account guarded runs;
record exact IDs/run receipts and partial failures, without a selector/extra
confirmation. Ordinary terminals and other accounts are excluded.
Disabling the profile default applies to future launches and explains that current
runs keep their policy. Group multi-session lists by confirmed project and host,
with real pagination and task/attention totals independent of visible rows.

Reconnect only desired terminals; list/status discovery does not acquire writer
or resize authority, acknowledge work, or instantiate every hidden screen. Plain
CLI direct attach may render a bounded ANSI projection; Warpai shell/block state
must still remain correct for subsequent native UI attachment.

## Acceptance and resource priorities

All FR requirements are mandatory for the stated SSH/WSL scope. Native local
regression is required, but local persistence is excluded. CPU and memory targets
in TECH are provisional acceptance targets, not measured claims. Report guardian
runtime overhead separately from Agent/vendor/test/server resources and include
threads, parser state, project Brokers and queues. S8 freezes tested limits.
No permanent login service, root privilege or new sleep-prevention policy is needed.

## Unified daemon and command experience

Enabling guardian ensures one compatible daemon for the execution account, not
one per Agent. All opted-in managed sessions use it. Different users/kernel/distro
boundaries remain isolated; same-account SSH/WSL access shares the daemon with
separate logical MCP/project scopes. Keep all legacy/local behavior intact.

Offer `warpai list`, `warpai resume [name]` and `warpai stop [name]` in participating
Warpai remote shells, implemented through absolute Companion paths without PATH or
persistent shell changes. Plain external SSH/WSL shells use the absolute helper.
The label denotes a managed session/job, not a task-engine lifecycle identity.
Names must resolve uniquely; IDs and project filters disambiguate. Bare resume
uses current/unique/last-used live-session selection without a picker; bare stop
means stop-all for the current execution account with exact per-run receipts and
no additional confirmation. See [CLI.md](CLI.md).

Warpai reconnect automatically populates sessions and task/status information.
A previous active view can reattach when its original run remains and the writer
is free; occupied views are never stolen. Other PTYs remain lazy. Information
restore cannot auto-relaunch processes, change task authority or replay mutations.

R4 narrows the CLI to list/resume/stop plus standard help/version. List includes
needed availability/state, creation/naming remain in existing Warpai UI, detach
uses client closure/direct-attach escape, and guardian start/idle exit are automatic.
No standalone read/wait/completion/admin or deletion workflow is justified now.
The CLI decision matrix records concrete coverage for every excluded candidate;
Herdr is a design reference, not a command-parity requirement.
