# Warpai 2.0.0 implementation tasks

Status: R4 guardian implementation in progress. Each completed unit requires a
method record in [IMPLEMENTATION.md](IMPLEMENTATION.md), linked to requirements
and native cases in [VERIFY.md](VERIFY.md). No local persistence task.

## Outstanding implementation

| Area | Remaining code | Acceptance dependency |
| --- | --- | --- |
| S1, S5 | Native Windows account guardian for SSH targets | Retain the same ConPTY/Job across Windows SSH bridge loss and verify installation/upgrade |
| S2, S3 | Complete parser/shell-block snapshot, paging, validation and gap recovery | Import active/alternate screens and split sequences without replaying OSC side effects |
| S4, S5 | Desktop per-launch choice, guarded launch/detach, catalog/watch, safe previous-view reattach and native Windows shell entry | Native EN/ZH static and live views, real CLI TTY and Bash/Zsh bootstrap integration on release artifacts |
| S6 | Restore remote workspace context and uncertain-write receipts across desktop restarts; complete failure/stop semantics | Host loss, permission waits, descendants, retention and native editor interruption |
| S7 | Shared pool dispatch, bulk fairness and resource accounting | Multi-project load and actual CPU/RSS/thread measurements with noisy, quiet and blocked sessions |
| S8, S9 | Release gates, artifact integration and bilingual product documentation | Complete V01–V30 on matched artifacts, mixed-version upgrades and release review |

The controlled Unix SSH fixture exercises the actual installed-style `ctl`
entry on an allocated SSH TTY. It checks same-process output and input after a
bridge gap, split detach input, and restoration of the outer terminal settings.
It also starts a CLI stop inside one of two guarded PTYs and verifies both
native process exits after the caller loses its own terminal.

## S0: scope, caller inventory and contract freeze

- [ ] Inventory managed launch/control/read/release callers, native WSL foreground
  discovery, SSH/WSL settings, Agent/session constructors, remote UI close/quit/
  language-restart paths, CLI, feature gates, tests and package manifests.
- [ ] Freeze default-off profile preference, immutable per-session policy, allowed
  session kinds, final execution identity and additive API/schema/errors.
- [ ] Freeze guardian feature-responsibility matrix and no-local-migration boundary.
- [ ] Freeze account-global daemon election with isolated logical SSH/WSL scopes,
  CLI exact-name/ID/default/bulk-stop rules, shell conflicts and catalog watch API.
- [ ] Freeze current/unique/MRU resume, immediate bare stop-all, activity sequencing
  and the minimal list/resume/stop parser/help/version. Confirm every extra tool
  has a missing user journey before adding any task/API for it.
- [ ] Design shared-service catalog, startup locking, limits and data migration.
- [ ] Add old/new client/host and mixed guarded/unguarded protocol fixtures.

Deliver: caller/symbol inventory, field-number/schema contract, policy state
matrix, source impact and decision record. No personal-state migration.

## S1: shared account guardian and retained native ownership

Depends on S0.

- [ ] Add policy to new managed launches without changing legacy disconnect.
- [ ] Retain guarded native PTY, reader/writer, screen, process group/Job and
  optional Agent RunBinding when a client disconnects; shell/command sessions
  receive no Agent authority.
- [ ] Add scoped catalog/list/inspect, minimal metadata and bounded exit retention.
- [ ] Reuse service implementation for one guardian per execution OS account,
  shared across Agent types/clients and proven same-account SSH/WSL routes. Retain
  logical adapter/Store separation and all legacy paths; no per-task/type daemon.
- [ ] Prove SSH launcher exit does not kill retained Unix/Windows native ownership.

Deliver: policy/lifetime diagrams, source methods and same-process receipts.

## S2: compact terminal state and restoration prototype

Depends on S0; representation gate before live UI integration.

- [ ] Trace native parser, readiness screen, shell integration and command blocks.
- [ ] Reuse one canonical minimal screen; select exact versioned snapshot encoding.
- [ ] Allocate rings lazily, serialize only on attach/resync, bound account-wide
  serializer concurrency and include all buffers in memory accounting.
- [ ] Fixture prompt/alt screen, split UTF-8/escape, draft, resize and command blocks.
- [ ] Native static import screenshot/semantic QA before attaching real sessions.
- [ ] Verify clipboard/OSC side effects are not replayed; CLI ANSI projection keeps
  host native block metadata intact.

Deliver: encoding/field map, alternatives, semantic comparisons, artifacts and
memory size measurements. Raw text-only replay is not full terminal acceptance.

## S3: attachment fencing, ordered IO and reconnect protocol

Depends on S1; snapshot path depends on S2.

- [ ] Add attach/detach/explicit takeover with exact writer generation.
- [ ] Serialize human input/resize and guarded peer delivery; validate at write.
- [ ] Bound queues and record accepted/written/failed/unknown dispositions.
- [ ] Add immutable paged snapshot cut/tail, digest, expiry and gap resnapshot.
- [ ] Keep legacy controls from operating guarded sessions; late detach/stop must
  not affect a replacement attachment/run.

Deliver: race/idempotency/partial-write tests and no-replay native receipts.

## S4: SSH opt-in UI and plain-terminal attachment

Depends on S1-S3.

- [ ] Add future-launch target preference and per-launch choice, default off.
- [ ] Build fixed-data native states using existing widgets/tokens and review them
  before real data binding; include EN/ZH, multiple sessions and partial failures.
- [ ] Bind guarded Agent and shell/command launches and catalog/status projection.
- [ ] Close/quit detaches guarded remote views; legacy paths remain unchanged.
- [ ] Implement private Companion `ctl` list/resume/stop plus required guarded
  launch/attach/detach transport. Add session-only `warpai` command forwarding by
  absolute path; no PATH/profile change or override of existing user commands.
- [ ] Verify names/IDs/current-context/ambiguity and current/unique/MRU resume
  without a picker; bare stop immediately captures all current-account guarded
  runs without confirmation, returning exact per-run partial-failure receipts.
- [ ] Implement only list/resume/stop and standard help/version. Merge needed
  health/session state into list without new worker/snapshot/lease side effects.
- [ ] Implement/test the direct CLI attachment's local detach escape and literal
  prefix handling; keep normal shell/desktop input and existing UI launch intact.
- [ ] Automatically restore bounded catalog/task/status projections and free-writer
  previous view on reconnect; use shared watch/cursors and lazy snapshots. Never
  steal a CLI writer, acknowledge work or relaunch an interrupted Agent.
- [ ] Discover sessions/tasks first; attach selected PTYs and lazily restore
  Explorer/editor/Review without replaying uncertain transfers or writes.

Deliver: static/live screenshots, CLI handoff and multi-session reconnect evidence.

## S5: mandatory WSL guardian support

Depends on S0-S3; release cannot defer this slice as optional.

- [ ] Use dedicated guest binary/logical namespace and exact confirmed distro/user.
  Prove a shared guardian with SSH only when both reach the same execution account;
  do not equate different distributions/users or merge project/MCP authorities.
- [ ] Explicit guarded launches use guest-service-owned PTYs; preserve native
  foreground MCP discovery and ordinary WSL shells for unguarded sessions.
- [ ] Bind original Linux Agent/process birth identity; no Windows MCP injection,
  arbitrary PID adoption or nested-SSH authority confusion.
- [ ] Add target preference and explicit CLI attachment from a WSL shell.
- [ ] Prove Linux service/process lifetime after all Windows bridges/GUI exit;
  assess minimum guest keepalive only if supported WSL behavior needs it.
- [ ] Treat distro termination/Windows shutdown as host loss, not live detach.
- [ ] Validate source-matched bundled guest installation, update and rollback.

Deliver: real Windows/WSL tests with multiple distros/users and headless task
completion. Linux SSH tests cannot substitute for guest lifetime evidence.

## S6: stop, failure, retention and feature restoration

- [ ] Observe native exit/descendants and distinguish stop request from proof.
- [ ] Preserve permission/input waits, task revisions, Coordinator rights and
  unresolved supervision while guarded sessions are detached.
- [ ] Persist minimal interruption metadata without screen/argv/token storage.
- [ ] Apply ended-output TTL/pressure eviction and genuine 60-second idle exit.
- [ ] Reconcile file-transfer/write uncertainty; release client-only resources.
- [ ] Handle mixed-policy quit/stop-one/bare stop-all with per-run receipts and
  explicit daemon shutdown only after all owned native exits are confirmed.
- [ ] Make admitted stop batches daemon-owned so stopping the caller's PTY cannot
  cancel sibling stops; retain bounded operation-ID receipts and retry dedup.
- [ ] Reuse compatible live helper versions; refuse destructive replacement.

Deliver: native fault matrix, feature coverage receipts and privacy/storage checks.

The controlled Unix SSH fixture must prove that a direct peer message reaches
the same registered Agent after its writer disconnects. Verify an exact message
receipt from the native PTY child before opening a replacement receiver bridge,
then verify retained output and the still-unacknowledged inbox entry after attach.
The same fixture then queues a tracked task for that ready Agent, closes both
SSH bridges, and requires a task/attempt receipt from the exact native run
before opening a replacement bridge to inspect the submitted task.

## S7: low-overhead scheduling and multitasking

- [x] Replace guarded per-Agent 300 ms polling with shared event notifications,
  indexed session/run wakes and nearest-deadline scheduling.
- [ ] Prove coalescing preserves idle, paste/Enter, draft and permission guards;
  protect against lost notifications and maintain bounded safety reconciliation.
- [ ] Use bounded per-session queues and fair ready-session turns; reserve stop/
  control capacity and avoid global locks across blocking IO/snapshots/Git.
- [ ] Preserve serial per-Agent task slots and cross-session parallel work without
  introducing a second task engine or automatic Agent spawning.
- [ ] Exercise noisy/quiet/blocked/failed sessions in multiple isolated projects.
- [ ] Measure idle CPU, base/incremental/peak memory, threads and context switches;
  tune rings, parser history, project Broker caches and serializer concurrency.

Deliver: comparison against current Companion polling, fairness latency receipts,
measured limits and documented deviations. Do not publish unmeasured savings.

## S8: cross-platform native acceptance

- [ ] Run V01-V30 on exact source; SSH Linux/macOS/Windows and real WSL required.
- [ ] Native local and all legacy/guard-off paths retain behavior and resource use.
- [ ] Run default/warp_platform checks and existing task/handoff/SSH/WSL regressions.
- [ ] Freeze measured resource budgets, queue/capacity/TTL and viewport limits.
- [ ] Review EN/ZH static/live screenshots, focus/drafts and multi-session controls.
- [ ] Close P0 defects and record all native evidence and known scope boundaries.

Deliver: evidence index with source hashes, platform/run IDs and measured results.

## S9: documentation, distribution and release

- [ ] Follow [RELEASE.md](RELEASE.md) with actual accepted optional scope.
- [ ] Package control commands inside existing Companion/guest binaries, verify
  absolute-path dispatch and shell-session scope, and document external-shell use.
- [ ] Synchronize bilingual README/source guides/website placement; no changelog
  framing in evergreen docs and no local persistence claim.
- [ ] Package source-matched SSH and WSL helpers and test compatible live upgrades.
- [ ] Validate drafts, artifact provenance, hashes and actual installed versions.
- [ ] Publish only through the existing release gate, then verify public surfaces.
- [ ] Update MEMORY and detailed implementation/release receipts.

A check requires an actual implementation commit plus appropriate evidence. An
Agent assertion, proposed design or submitted CI run cannot close a work item.
