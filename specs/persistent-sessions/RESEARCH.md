# Persistent local and remote sessions: feasibility

Status: historical feasibility research, not implementation or shipped capability.
Scope corrected by the owner on 2026-11-18: guardian is optional/default-off for
SSH and WSL, native local persistence is excluded, and low overhead plus multiple
sessions/tasks are core. The revised 2.0.0 documents supersede earlier local-runtime
recommendations below; they must not be used as an implementation checklist.
Reviewed 2026-11-18 against Warpai `db280c6b57c3eab7acb8c8e04ab8c8aab084828a`.

The detailed proposed 2.0.0 delivery package is now in
[specs/2.0.0](../2.0.0/README.md), including implementation methods and an actual-work
ledger. Its creation does not implement persistent sessions.

## Problem and acceptance boundary

An Agent should be able to continue working when its desktop view closes or its
SSH transport disconnects, then return to the same live process and collaboration
run when a client reconnects. A retained session belongs to its execution host and
OS account; a desktop is an attached view, not its lifetime owner.

Distinguish three capabilities:

| Capability | Process identity | Proposed scope |
| --- | --- | --- |
| Detach and reattach while the host runtime lives | Same process and run | First milestone |
| Runtime crash, host restart, or power loss recovery | New process; optional vendor-native conversation resume | Separate follow-up |
| Moving a running session between hosts | Different execution environment | Excluded |

Sleeping a laptop preserves neither execution progress nor network availability.
Keeping work on an awake remote host is the useful continuity path in that case.
Current durable tasks/handoffs support supervision recovery, not process survival.

## Existing implementation and reusable foundations

- `app/src/terminal/local_tty/terminal_manager.rs`: `TerminalManager::drop` shuts
  down the PTY event loop. The desktop currently owns the local terminal lifetime.
- `app/src/agent_communication.rs`: `AgentCommunication` starts and holds the local
  `RunningBroker`; readiness observations and wake scheduling also involve GUI
  models. Keeping only the child process alive would not preserve collaboration.
- `crates/agent_bus/src/companion_terminals.rs`: the account service already owns
  native PTYs, a bounded output buffer, native screen observations, and guarded
  peer delivery. However, `disconnect` revokes the attachment, drops the run
  binding and requests native process stop. Existing reconciliation explicitly
  refuses adoption by another connection.
- `crates/agent_bus/src/companion_service.rs`: the service remains alive while
  terminals/guests are active; its 60-second timeout is for genuine idle periods.
- `crates/agent_bus/src/native_pty_{unix,windows}.rs`: Unix process-group ownership
  and Windows Job Object ownership already provide explicit cleanup boundaries.
- `specs/agent-communication-v2/TECH.md` explicitly specifies connection-owned
  Agent PTYs and excludes retained-session takeover. Persistence changes this
  lifecycle contract; it is not a transport reconnect fix.

Historical recommendation (superseded for native local scope): remote Agent
sessions are the smallest first implementation because their host
service already owns most of the required machinery. Local sessions need a
headless ownership boundary for PTY, Broker, readiness and guarded delivery.
Evaluate reuse of Companion's account runtime before introducing another daemon.
Existing terminal transport and renderer paths require compatibility assessment;
do not replace unrelated native terminal behavior to accelerate the first slice.

## Recommended design constraints

1. Make persistence explicit for sessions created through the supported host
   runtime. Preserve existing ordinary terminal behavior initially. Distinguish
   detach, reconnect and stop in the UI; closing a view must not silently mean
   either background execution or process termination.
2. Separate durable session identity, runtime boot identity, native run identity
   and client attachment generation. Reconnect to a proven same run without
   reassigning task ownership or Coordinator authority. A new runtime/process
   cannot silently inherit the old run's capabilities or execution claims.
3. Release the client attachment on disconnect, retaining the host-owned PTY and
   valid run binding. Revalidate host/account/root/runtime/run before attachment.
   Do not revive the archived takeover implementation without a new contract.
4. Start with one writable attachment per session. Fence input and resize by its
   generation; serialize human input and guarded peer delivery through the host.
   Never replay uncertain terminal input after reconnect. Multiple read-only
   viewers can wait until there is a demonstrated need.
5. Restore terminal state from a bounded, sequenced snapshot plus live output.
   Raw output tails alone cannot reconstruct alternate screens, modes and cursor
   state. Prove compatibility with Warpai command blocks, shell integration,
   scrollback and resize behavior before promising full terminal fidelity.
   Report output gaps honestly. Keep screen/output history in memory initially;
   disk transcript persistence would change the current privacy/storage boundary.
6. Keep native vendor executables, args, cwd and authentication. A Warpai-owned
   PTY service must not become a replacement Codex daemon/frontend or modify
   vendor configuration. Use private per-user IPC and existing SSH transport;
   no public listener, hosted account service or cross-host task federation.
7. Preserve permission waits, blocked-state detection, draft protection and
   exact-run wake guards while no desktop is attached. GUI focus/reattachment
   must not acknowledge work or fabricate readiness. Unproven process exit
   remains uncertain execution rather than permission to launch a duplicate.
8. Retained live processes keep the host service active. Apply the existing idle
   timeout only after there is no retained live work. Specify bounded retention
   for exited sessions so finished output remains inspectable without an
   unbounded idle service or unlimited memory growth.
9. Negotiate persistent-session support and snapshot protocol compatibility.
   A client update must not automatically replace an incompatible live host
   runtime and kill its sessions. Companion needs a new feature-bearing release
   if implemented; protocol-major changes depend on the final compatibility
   contract. Do not retroactively require this capability for 1.9.0 / 6.1.0.

## Delivery sequence

1. Specify API identities, attach/input/resize/stop authority, lifetime policy,
   compatibility behavior and terminal snapshot format before coding.
2. Implement and validate retained remote Agent sessions using the account
   runtime, with explicit reconnect UI and no automatic vendor resume.
3. Apply the same guarded contract to the dedicated WSL guest service, preserving
   native foreground discovery for unguarded sessions. Native local persistence
   and local Broker migration are excluded by the owner's revised scope.
4. Include explicitly guarded remote shell/command sessions and fair multi-session
   scheduling. Defer vendor-native reboot resume, multi-writer collaboration and
   live daemon upgrade handoff.

## Required verification before release

- GUI exit/reopen and SSH loss/reconnect preserve the same native process/run.
- Agents exchange tasks, report results and observe permission waits while the
  desktop is absent; local and remote coordination regressions remain covered.
- Stale attachments cannot inject input or resize; uncertain input is not sent
  twice. Host/account/project and runtime replacement invalidate stale authority.
- Output flood stays bounded; screen snapshots and live tails form one ordered
  stream. Exercise alternate-screen apps, Unicode, command blocks and resize.
- Stop failure, process descendants, Windows Job handles and disconnected clients
  produce truthful status. Empty runtimes exit; exited-session retention is bounded.
- Runtime crash and host reboot never masquerade as continuous execution or
  automatically repeat a potentially completed command.
- Run native Rust acceptance in GitHub/cloud for macOS, Linux and Windows, plus
  direct WSL coverage if included. Perform desktop/mobile-adjacent view QA only
  for actual UI surfaces touched; no runtime validation was performed for this study.

## Herdr reference and deliberate exclusions

Herdr separates live detach/reattach from layout restoration and vendor-native
conversation resume after a server restart. Its host server owns running panes;
SSH carries a remote attachment. Its direct-attach API uses a single input/resize
controller. Experimental server replacement handoff is a separate capability.
These ownership boundaries are relevant; its TUI implementation and integration
installation model are not prerequisites for Warpai.

- [Persistence and remote access](https://herdr.dev/tui/docs/persistence-remote/)
- [Session state and restore](https://herdr.dev/tui/docs/session-state/)

References were reviewed on 2026-11-18; website documentation may change.
