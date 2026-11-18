# Herdr reference and Warpai adaptation

Reviewed 2026-11-18. Reference source:
[herdrdev/herdr at ee5bfb5](https://github.com/herdrdev/herdr/tree/ee5bfb5041bb3e4a49d591c72308aae3c81ecdde).
This is source/documentation research, not control of a running Herdr instance.
No Herdr commands, sessions or user processes were started or modified.

## Verified reference mechanisms

Herdr's server owns running panes; local/SSH clients can detach and later attach.
Its direct terminal attachment uses a single input/resize controller. Compatible
running servers need not restart merely because client versions differ.
Source: [Persistence and remote access](https://herdr.dev/tui/docs/persistence-remote/).

Herdr distinguishes live detach from restart-time layout/session reconstruction;
its runtime restart does not preserve arbitrary running processes. Experimental
live server handoff is separate. Source:
[Session state and restore](https://herdr.dev/tui/docs/session-state/).

Workspaces/tabs/panes organize many live terminals in a server. Agent state and
client presentation are distinct. Source:
[Concepts](https://herdr.dev/tui/docs/concepts/).

The pinned implementation includes client surface-interest handling that retires
pending rendering when a surface deactivates and refreshes projection identity on
activation. Its fairness tests verify bounded event/API batches preserve FIFO and
leave opportunities for scheduled work. These are useful patterns, not evidence
of specific idle RSS/CPU results:

- [surface_interest.rs](https://github.com/herdrdev/herdr/blob/ee5bfb5041bb3e4a49d591c72308aae3c81ecdde/src/server/headless/surface_interest.rs)
- [event_fairness.rs](https://github.com/herdrdev/herdr/blob/ee5bfb5041bb3e4a49d591c72308aae3c81ecdde/src/server/headless/tests/event_fairness.rs)

## Adopt, adapt and exclude

| Reference | Warpai implementation choice | Reason |
| --- | --- | --- |
| Server owns panes | Existing SSH/WSL account service owns opted-in PTYs | Reuse Companion; no per-task daemon or local migration |
| Detached client | Guarded launch policy distinct from connection-owned sessions | Existing workflows/defaults stay intact |
| Multiple terminals | Shared service, independent sessions and fair ready queues | Parallel Agents/tests/servers with bounded overhead |
| Surface interest | Stop client-only rendering/polling when detached; snapshots on demand | Preserve execution with minimum idle work |
| Event fairness tests | Flood/blocked-session tests with bounded IO/control batches | A busy session cannot starve stop or quiet-session work |
| Single terminal controller | Writer-generation fence and explicit CLI/GUI takeover | Prevent duplicate input and resize races |
| Compatible live servers | Reuse compatible helper, stage versions side by side if needed | Updates must not kill retained work |
| Restart recovery/handoff | Excluded from initial guardian release | Same-process detach first; no false reboot-survival claim |
| Herdr task/agent state conventions | Keep existing Warpai task/readiness/ack/review authority | Do not substitute a different collaboration engine |

WSL guest lifetime, default-off guardian policy, native Warpai block restoration,
low-resource budgets and task fairness are Warpai design work requiring native
acceptance. Herdr documentation is not proof those parts already work here.

No Herdr runtime dependency or copied implementation is proposed. Reuse existing
Warpai code first; any later code reuse requires license/provenance review and a
concrete benefit recorded in IMPLEMENTATION. Do not assert performance savings
without measuring the actual Companion implementation on stated workloads.

## R2 account daemon and private command adaptation

Warpai deliberately uses one guardian per execution account rather than one per
Agent kind; verified same-account SSH/WSL routes share the runtime with logical
scope isolation. Client reconnect restores catalog/status automatically. A private
Companion dispatcher backs `warpai list/resume/stop`; session-only shell exposure
avoids Herdr-style global PATH installation. Name resolution, exact-run stop and
no-argument behavior are explicit Warpai contracts in [CLI.md](CLI.md), not claims
that Herdr implements the identical syntax or task lifecycle.

## R3 CLI expansion reference

Rechecked [Herdr CLI reference](https://herdr.dev/tui/docs/cli-reference/) on
2026-11-18. Relevant families include server/status, session attach/stop/delete,
creation/rename/read, state/output waits and shell completion. Warpai adapts these
as small existing-API clients; its bare resume MRU selection and bare account-wide
stop are owner-specified semantics, not asserted Herdr defaults. See CLI.md for
the exact surface, authority, timeout, memory and no-profile-write rules.

Do not mirror Herdr plugins, machine administration, full tab/pane layout controls
or separate worktree management merely to complete the guardian CLI. They add
independent product scope and are not needed for the current remote workflow.

## R4 command necessity gate (supersedes R3 expansion)

The owner clarified that references justify tools only when actually needed.
Keep list/resume/stop and standard help/version. Their discovery/continuation/stop
journeys are explicit; existing UI and automatic runtime lifecycle cover creating,
naming and daemon management. Standalone read/wait/rename/remove/completion/admin
were over-scoped proposals and are withdrawn. The command decision matrix in
CLI.md is the active scope; the earlier R3 family survey is reference history,
not implementation work. No command-parity target with Herdr remains.
