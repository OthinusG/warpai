# Warpai 2.0.0 optional SSH/WSL guardian framework

Date: 2026-11-18. Baseline: `db280c6b57c3eab7acb8c8e04ab8c8aab084828a`.
Status: proposed specification; no runtime implementation or acceptance yet.
Revision R4 incorporates the owner's correction: preserve the existing workflow,
make guarding optional, exclude native local persistence, require SSH and WSL,
and prioritize low overhead and multi-session/task continuity.

## Product direction

An optional guardian keeps explicitly selected SSH/WSL sessions and their native
processes running when the Warpai desktop closes or its transport disconnects.
Reconnect to the same execution account/project and take over the same live
session. Preserve 1.9.0 tasks, handoffs, review and exact-run authority.

The guardian is default-off; the current unguarded workflow remains the default.
Enabling a connection-profile
preference only changes the default for future launches; each session stores its
actual launch-time policy. Existing sessions are never silently converted,
restarted or terminated. Native local terminals/Broker remain unchanged; no local
background service or local session migration is part of 2.0.0.

SSH execution hosts remain Linux, macOS and Windows. Direct WSL is a required
Windows-to-guest-Linux route, scoped to the selected distribution/user and its
existing isolated logical scope. WSL is an execution target here, even though
it resides on the same physical PC. Its availability still depends on that PC
and distribution running.

## Complete release scope

- Optional guarded remote Agent sessions and managed shell/command sessions;
  arbitrary pre-existing processes are not adopted by PID.
- One guardian daemon per execution OS account, independent of Agent type, hosting
  all managed projects/sessions. SSH/WSL routes share it when they prove the same
  account; their logical MCP/Store scopes stay isolated.
- Headless native PTY IO and existing Agent collaboration; client-only features
  reconnect on demand rather than staying active in a hidden desktop.
- Same-process reconnect via Warpai and explicit Companion CLI attachment through
  plain SSH or a WSL shell. Full project UI remains in Warpai.
- Low-memory terminal state, on-demand snapshots, event-driven scheduling, bounded
  queues, fair multi-session IO and existing per-Agent task serialization.

## Documents and source of truth

| Document | Owns |
| --- | --- |
| [AGENT-EXECUTION-PROMPT.md](AGENT-EXECUTION-PROMPT.md) | Archived self-contained implementation prompt with step-by-step methods, gates and recording requirements |
| [PRD.md](PRD.md) | Outcomes, opt-in policy, feature coverage and acceptance |
| [TECH.md](TECH.md) | Source changes, guardian runtime, low-overhead design and multitasking |
| [CLI.md](CLI.md) | Unified daemon, private commands and automatic client reconnect |
| [API.md](API.md) | Policy, identities, operations, ordering, compatibility and limits |
| [TASKS.md](TASKS.md) | S0-S9 ordered work and concrete deliverables |
| [VERIFY.md](VERIFY.md) | V01-V30 failure/compatibility/performance cases |
| [IMPLEMENTATION.md](IMPLEMENTATION.md) | Actual changes, methods, decisions and evidence |
| [RELEASE.md](RELEASE.md) | Packaging, bilingual content and release gates |
| [HERDR.md](HERDR.md) | Pinned primary-source reference and adaptation decisions |

The [feasibility study](../persistent-sessions/RESEARCH.md) is historical analysis;
this revised scope supersedes its local-persistence recommendation. The ledger
must distinguish proposed work, implementation and passed acceptance.

## Definition of complete

With guarding off, existing local/SSH/WSL behavior remains unchanged. With guarding
on, SSH and WSL processes and eligible collaboration continue without the desktop;
multiple sessions/tasks remain isolated and responsive. Reconnect verifies the
same runtime/run, restores a bounded terminal view and rejects stale input. Native
failure handling, client/host compatibility, resource measurements and packaging
pass on source-matched builds. Every delivered item has a detailed method record.

The CLI ships inside Companion and is invoked through verified absolute paths;
Warpai remote shells may expose a temporary `warpai` function. No PATH/profile
changes. Automatic reconnect restores catalog/state and safely restores a previous
writer only if free; it never steals another client's input ownership.

Bare CLI resume uses current/unique/most-recently-used live-session selection;
bare stop immediately stops all guardian-managed runs in the current execution
account. Only list/resume/stop plus standard help/version are CLI scope; CLI.md
records why other Herdr-like commands are unnecessary for current user journeys.

## Exclusions

No native local daemon/Broker migration, forced workflow replacement, automatic
adoption of existing native terminals, cross-host federation or process migration,
shared multi-writer terminal, disk transcript history, reboot resume, live runtime
upgrade handoff, bundled cloud AI, public daemon port or replacement vendor daemon.
