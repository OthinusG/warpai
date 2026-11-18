# Warpai 2.0.0 actual implementation ledger

Updated: 2026-11-18. Scope revision: R4 optional SSH/WSL guardian.

## Current source and evidence

Audit baseline: `61b3fcac` on `feat/2.0.0`, with `main` at `db280c6b`.
S1-l through S1-q describe the successive changes since the original baseline.
Recent receipts cover macOS arm64 app compilation and library tests, plus the
[Linux/macOS/Windows remote wire matrix on `36bdc6d3`](https://github.com/OthinusG/warpai/actions/runs/44282851874).
That matrix exercises the guarded Linux SSH lifetime and ordered resize over a
real SSH bridge. A [separate WSL run on `36bdc6d3`](https://github.com/OthinusG/warpai/actions/runs/44282850183)
passed the guarded lifetime test with every bridge closed for 12 seconds, then
failed two ordinary-workspace cases because the live guardian served another
Store. `92f31965` added the logical Store preflight. The source-matched [Windows
and WSL rerun](https://github.com/OthinusG/warpai/actions/runs/44298760969) and
[remote wire matrix](https://github.com/OthinusG/warpai/actions/runs/44298760965)
use that source. The matrix passed all three desktop runners, and the Windows
protocol passed. Both ordinary WSL workspace cases passed. The new guarded
first-owner fixture hit EOF before its reconnect assertions, so this run gives
no first-owner proof. After the installer permissions fix, the [native WSL
workspace step on `acd9e63e`](https://github.com/OthinusG/warpai/actions/runs/44311567207/job/134826760180)
passed both startup orders, a 12-second bridge-free reconnect, and both ordinary
workspace cases on the same runner. Full V01-V30 acceptance remains open.
Historical records below describe their own revisions.

The branch has a Unix account guardian, scoped Stores, guarded PTY and writer
fencing, a bounded SQLite catalog, a helper bridge, account RPCs and a private
CLI. The SSH and WSL preferences bind a verified remote identity for future
launches. The normal desktop Remote panel keeps its ordinary workspace channel.
Production guarded PTY launch, exact-run desktop restoration and native Windows
guardian ownership remain open.

### Audit findings and closure checks

| Gap | Closure check |
| --- | --- |
| CLI page limit exceeds wire limit; named selection reads only page one | Repaired in the working tree. CLI reads bounded pages. A real-listener fixture returns 40 rows across two pages without project admission. |
| WSL bridge flag and route passed as one argv entry | Repaired. The argv fixture verifies two distinct arguments; native Windows dispatched the real WSL guarded lifetime fixture on `36bdc6d3`. |
| Guarded capability advertised before proof | Installed builds advertise the private capability on the authenticated local operator channel. The remote data bridge keeps the desktop capability gated. |
| Stop operator admission, malformed exact target, replay, and truthful result | Repaired in the working tree. Direct operator mode, strict target, fixed run set, boot-scoped replay and unconfirmed exit status. Native caller-loss and process-tree acceptance remain open. |
| Store cutover and executable/route election | Source-matched Windows/WSL acceptance passed both first-owner orders with isolated logical Store identities and ordinary workspace coexistence. The released mixed-version install/upgrade matrix remains open. |
| Catalog launch/attach write failure | Repaired on the host. Launch reserves exact identity before spawn; an acquired writer returns its native tuple even if the projection write fails. Pending exact generations reconcile on maintenance without changing a newer writer or reordering MRU on replay. |
| Catalog watch and cursor scoping | Repaired in the working tree. Cursor formats now bind filters. The watch ring stores bounded sanitized rows and preserves removal identity. Fixture coverage includes a changed filter and a real retained-window gap. |
| Desktop launch, automatic restore, snapshots and shell entry | Launch through a verified opt-in target, restore only exact runs/free writers, recover from output gaps and exercise the private CLI in a real TTY. |
| Windows and release acceptance | Prove Windows native ownership and real WSL/SSH lifetime, mixed-version install/upgrade, fairness/resource limits, and remaining V01-V30 cases on matched artifacts. |

## Work status

| Work | Status | Actual method/evidence |
| --- | --- | --- |
| D0 original research/framework | superseded_scope | Source inspection and initial documents; native local proposal withdrawn |
| D1 R1 scope/framework revision | verified_documents_only | Optional remote scope correction; superseded topology details refined in D2 |
| D2 unified account daemon/private CLI | verified_documents_only | Initial defaults superseded by owner-specified R3; no runtime work |
| D3 MRU/all-stop defaults and command extension | superseded_command_scope | MRU/all-stop retained; extra commands withdrawn by D4 |
| D4 minimum justified CLI | verified_documents_only | Core commands/decision matrix, narrowed APIs/tasks and document integrity; runtime pending |
| D5 implementation prompt archive | verified_documents_only | Step-by-step handoff, current scope, evidence requirements and archive integrity; runtime pending |
| S0 contract/caller inventory | in_progress | Caller/ownership inventory and guarded wire fields frozen ([evidence/S0-baseline.md](evidence/S0-baseline.md)); v1 SQLite catalog and account RPC tags are on the wire; cutover, operator semantics and compatibility evidence remain open |
| S1 account guardian/native ownership | in_progress | Local Unix guardian and PTY fixtures cover ownership, fencing, the durable catalog and account RPCs. Installed-helper SSH/WSL election, mixed-version Store cutover and native ownership proof remain open. |
| S2 compact terminal restoration | in_progress | Local byte-cut ordering barrier. Decoder and screen/block export/import, lazy snapshots and native fidelity remain open. |
| S3 attachments and ordered IO | in_progress | Exact attachment tuple, CAS takeover, bounded input ledger with idempotent sequence and counted write receipts, non-evictable unknown delivery, between-write cancellation, fail-closed writer faults, exact tuple echo, session-level stale replies that preserve the multiplexed connection, bounded ended-output retention, catalog `attachment_generation` on attach, and the `ctl resume` attach/detach/split-prefix escape client; snapshots and GUI reconnect selection pending |
| S4 SSH optional UI/CLI | in_progress | Verified-target `guardian` preference, controlled Linux/macOS SSH reconnect and native CLI TTY, private `ctl list/resume/stop`, and exact live project admission across working directories. Bash/Zsh session-only command is wired into bootstrap. Desktop launch/restore and native Windows shell entry remain open. |
| S5 mandatory WSL guardian | in_progress | Native Windows/WSL on `acd9e63e` passed both first-owner orders, guarded bridge loss and exact-run reconnect, and ordinary workspace coexistence. Desktop launch/restore, full upgrade matrix, distro termination and host reboot acceptance remain open. |
| S6 failure/feature restoration | in_progress | Boot-scoped stop replay and catalog ownership exercised on the host. Real descendant, caller-loss, permission, task and file-transfer recovery remain open. |
| S7 resource/fairness scheduling | in_progress | Guarded wake checks use runtime notifications, direct terminal lookup and quiet/ready deadlines with pacing and periodic reconciliation. Local regressions cover account control at 32 occupied bridges and a sibling launch while another run holds its session lock. Shared pool dispatch, bulk fairness and measured resource budgets remain open. |
| S8 native acceptance | in_progress | Source-matched controlled Linux SSH and Windows WSL receipts above cover portions of the native matrix. Release-aligned desktop, native Windows guardian and resource measurements remain open. |
| S9 distribution/release | in_progress | Unix and generated WSL installer regressions cover provenance, permissions, repeat install and scope isolation. Native WSL uses the source-matched package. Release artifacts and the complete mixed-version upgrade matrix remain open. |

## Mandatory method records

Update the ledger with each implementation change and later evidence. A completion
record must explain what changed and how; "added guardian" or "tests pass" is not
sufficient. Fill every field or mark not applicable with a reason. State actual
run/source/platform and measured resource use, not proposed budgets as results.

### Record template

```text
Record ID / date:
Work item and PRD requirements:
Status: proposed | in_progress | implemented_unverified | verified | blocked
Baseline commit:
Implementation commit / PR:

Problem and observable before/after:
Scope: callers, fixtures, protocol/config/exports, platforms affected:
Files and symbols changed:

Implementation method:
  - Runtime/process ownership before and after:
  - Identity, state transitions and concurrency ordering:
  - API fields/operations and validation boundaries:
  - Data model/migration and privacy implications:
  - Native terminal integration and bounded-resource behavior:
  - OS-specific process, IPC and packaging behavior:
  - Compatibility, failure handling and rollback behavior:

Decision and rationale:
Simpler/reused alternative considered:
Reason rejected or selected:
Contract changes and links to API/TECH:

Verification:
  - Case IDs and exact source commit:
  - Runner OS/architecture and helper/app versions:
  - CI run/job/artifact links or reproducible command:
  - Expected result and observed result:
  - Relevant numeric measurements:
  - Failures found, fixes applied and rerun receipt:

Unverified behavior and remaining risks:
Follow-up tasks / affected gates:
MEMORY update or reason no durable fact changed:
```

## Guardian-specific record extensions

For relevant units also record: actual launch-time policy and default-off/legacy
regression, SSH vs WSL namespace and ownership, notification/deadline/ready-queue
algorithm, per-session fairness/backpressure, thread/cache/serializer cost, and
which PRD features continue headlessly versus reinitialize on client reconnect.

Performance records include CPU measurement formula/window, base/incremental/peak
RSS or PSS, vendor-process exclusion, VM/platform conditions and repeat runs.
Concurrency records include lock boundaries, lost-wake protection, queue limits,
input/stop priority and same-Agent serial execution proof. WSL records distinguish
bridge exit, desktop exit, distro stop and Windows shutdown.

## D1: revised content/method package

- Request: preserve the current workflow, make guardian opt-in, exclude native
  local persistence, prioritize SSH/WSL low-overhead background functions and
  multi-task handoff; reference Herdr where useful.
- Actual content changes: revise README/PRD/TECH/API/TASKS/VERIFY/RELEASE/this ledger,
  add HERDR reference, supersede old research scope and update MEMORY. Production
  code, protobuf, manifests, workflows, settings and external state untouched.
- Method: inspect current Companion run/connection ownership, 300 ms wake loop,
  memory-only vt100 observation and dedicated WSL native-process contract; compare
  pinned Herdr surface-interest/fairness code and official persistence docs.
- Decisions: reuse account service; immutable per-session policy; SSH/WSL managed
  sessions plus optional Agent bindings; no local Broker move; shared event/deadline
  scheduler and fair per-session queues; memory-only output/on-demand snapshots;
  preserve existing serial per-Agent task slots and exact-run task authority.
- Proposed limits and companion version remain unmeasured/unimplemented choices.
- Runtime tests, native screenshots and benchmarks were not run. Only document
  integrity checks may be marked passed for D1.
- Document verification passed: relative links, quoted existing source paths,
  baseline SHA, fenced blocks/whitespace, actual Cargo package names, 14 FR IDs,
  S0-S9 task/ledger entries and V01-V24 rows. Scope review confirms mandatory WSL,
  default-off guarding and no native local migration in active implementation tasks.
- Memory records the owner's scope correction and the current document entry.

## D2: unified daemon, private commands and automatic restore (R2 defaults superseded)

- Request: enable one low-resource daemon on a remote target, independent of Agent
  type; reconnect automatically fetches information; add list/resume/stop commands
  with names, install inside Companion and do not modify PATH.
- Actual changes: new CLI.md plus R2 updates across framework/API/tasks/verification/
  release/reference/ledger and MEMORY. No production/runtime/package changes.
- Method: confirm existing detached service bootstrap and private helper install
  path; define one account-global guardian with logical scope adapters, private
  `ctl` dispatch, session-only shell function and absolute-path fallback.
- Historical R2 defaults (superseded by D3): resume used a picker for several
  sessions and bare stop selected one. The owner replaced these with MRU resume
  and immediate account-wide bare stop. Named sessions remain distinct from task
  lifecycle identities; automatic previous-view attach still refuses conflicts.
- Source scope correction: R1 separate service namespaces remain logical isolation;
  new guarded ownership is one account daemon, not one guardian per namespace.
  Preserve unchanged legacy services and do not merge existing Stores/Agent rights.
- Added FR-15-FR-17 and V25-V28. Runtime behavior, shell injection, daemon election,
  resource measurements and native Windows/WSL acceptance remain NOT RUN.
- Document integrity passed: ten document links/source paths, pinned source,
  balanced code blocks/whitespace, 17 requirement IDs, S0-S9 task/ledger mapping,
  V01-V28 rows and revised topology/CLI scope review. Memory updated with the
  account/CLI/PATH choice. Native behavior and resource targets remain unverified.

## D3: owner-specified defaults and complementary commands (extension withdrawn by D4)

- Request: bare resume uses current/unique/last-used live session; bare stop stops
  all managed sessions. Complete relevant command families using Herdr as reference.
- Actual changes: CLI/MRU/batch-stop specification, additive API methods, PRD and
  task/verification/release/reference synchronization; MEMORY records exact defaults.
  No runtime code, parser, protocol field, installer or external state changed.
- Method: recheck official Herdr CLI command families; map lifecycle/status/read/
  wait/completion commands to existing managed native launch, catalog, readonly
  terminal projection and shared event/deadline infrastructure. Avoid a new task
  engine, plugin/machine system or remote TUI.
- MRU design: account-serialized sequence on foreground attach/selection only;
  current, unique, most-recent-live order and deterministic no-history fallback.
  Reads/background refresh do not reorder; writer conflicts remain explicit.
- Stop design: no selector/confirmation/yes flag for bare stop; capture current
  account guarded runs and dispatch bounded daemon-owned stops. Caller PTY loss
  cannot cancel sibling stops; exact operation/run receipts reconcile lost replies.
- Command extension: new/status/detach/rename/read/wait/remove/daemon and installed
  help/version/completion; read/wait resources bounded, native task/readiness
  authority preserved, no PATH/profile writes or automatic process resurrection.
- Added FR-18/FR-19 and V29/V30, expanded V25 for immediate stop and caller loss.
- Document integrity passed: links, source paths, Markdown table columns, fenced
  blocks/whitespace, 19 requirement IDs, S0-S9 ledger mapping, V01-V30 cases and
  removal of superseded defaults from active contracts. Runtime/CLI/native/
  benchmark acceptance NOT RUN.

## D4: command necessity correction

- Owner correction: do not copy Herdr tools wholesale; include a tool only after
  demonstrating it is needed. R3 expanded the product beyond the current goal.
- Actual change: limit public CLI to list/resume/stop plus standard help/version.
  Preserve owner-specified MRU resume, immediate account-wide stop, stable-ID
  disambiguation and explicit writer takeover. Update CLI/API/PRD/TECH/TASKS/VERIFY/
  RELEASE/README/HERDR and MEMORY; no runtime source changes.
- Selection method: map each candidate to a concrete continuity journey and test
  whether existing GUI/automatic lifecycle/core command already covers it. CLI.md
  records need/coverage/decision for every excluded family.
- Retained methods: list combines cached availability and session facts; resume
  reuses native attach/snapshot/input with a local detach escape; stop reuses exact
  daemon-owned native stop batches and receipts. No second protocol/task engine.
- Withdrawn methods: standalone projection readers, state waiter registries,
  rename/remove operations solely for CLI, completion generation and public daemon
  administration. Keep existing GUI, snapshot, Broker notifications and retention
  safeguards intact; do not delete existing functionality to simplify the CLI.
- Verification scope: FR-19 and V30 now enforce minimal surface/no side effects;
  V26 covers the actual detach escape instead of an extra detach command. Existing
  FR IDs and other native cases remain meaningful. Runtime remains NOT RUN.
- Document integrity passed: relative links/source paths, table columns/code
  fences/whitespace, 19 requirement mappings, S0-S9 ledger and 30 cases; core CLI
  contains no withdrawn commands and API/tasks contain no required extra-tool
  implementation. MEMORY records the owner's necessity gate. Native/CLI/resource
  acceptance remains NOT RUN.

## D5: self-contained implementation prompt archive

- Request: archive a complete execution prompt that details how to perform every
  step of the currently requested 2.0.0 work.
- Actual change: add AGENT-EXECUTION-PROMPT.md, index it in README and MEMORY, and
  record this archive. Existing runtime task status and acceptance remain pending.
- Method: consolidate final R4 owner scope and superseded-proposal exclusions;
  define Steps 0-17 with concrete source inputs, implementation procedures,
  acceptance and method/evidence records. Map them to S0-S9 and V01-V30.
- Coverage: baseline/caller inventory, wire/storage identity, account election,
  guarded native lifetime, catalog/MRU, compact snapshot fidelity, writer fencing,
  headless tasks, real WSL, private three-command CLI, opt-in UI/reconnect,
  event-driven fairness, failure/retention, packaging, cloud checks, measurements
  and review/release preparation. Preserve actual session publication boundaries.
- No implementation agent was started. No runtime code, native process, workflow
  dispatch, installation, version, commit or public state was changed by archiving.
- Archive integrity passed: 586-line prompt, 18 ordered steps each with how/accept/
  record, relative links and existing source paths, actual Cargo package names,
  owner scope/default statements and README/MEMORY references. Native/CLI/benchmark
  evidence remains NOT RUN.

## S0: caller/ownership freeze for guarded sessions (partial)

```text
Record ID / date: S0-a / 2026-11-18
Work item and PRD requirements: S0 inventory and additive contract (FR-02, FR-03)
Status: implemented_unverified
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: 87e18c942630e58c91849532fb132323e0500ab5 on
  feat/2.0.0; no PR

Problem and observable before/after:
  Before: protocol major 1 had only connection-owned launch/control; no launch
  policy, writer identity or adoption path, and no caller inventory.
  After: additive frozen fields/actions exist and every managed terminal caller
  was inspected and recorded in evidence/S0-baseline.md.
Scope: crates/remote_protocol proto+validation, crates/agent_bus Companion and
  HostClient, app panel launch literal, tests; Unix/Windows shared code.
Files and symbols changed:
  crates/remote_protocol/proto/remote_server.proto (TerminalLaunch, TerminalAction,
    TerminalControl, TerminalState)
  crates/remote_protocol/src/managed.rs (valid_terminal_state attachment/label)
  specs/2.0.0/evidence/S0-baseline.md (new)

Implementation method:
  - Runtime/process ownership before and after: unchanged by S0; ownership map recorded.
  - Identity, state transitions and concurrency ordering: writer tuple is
    (attachment_id UUID, monotonic generation); connection_id stays transport-only.
  - API fields/operations and validation boundaries: numbers as listed in the
    evidence file (launch 9-10, actions 8-9, control 11-13, state 16-19); label
    bounded to 128 bytes without controls; attachment id must be a non-nil UUID;
    legacy defaults false/empty/0.
  - Data model/migration and privacy implications: in-memory only; label is a
    nonsecret selector; no argv/environment/screen persistence added.
  - Native terminal integration and bounded-resource behavior: none in S0.
  - OS-specific process, IPC and packaging behavior: none in S0.
  - Compatibility, failure handling and rollback behavior: legacy decoders ignore
    new fields; new clients must reject guarded fields unless the runtime
    advertises guarded_remote_sessions_v1; capability is intentionally not
    advertised yet, so no client path is enabled.
Decision and rationale: reuse TerminalLaunch/Control/State additively instead of
  new RPCs because the existing dispatch, fencing and validation already own the
  project-admitted terminal path; avoid a second terminal protocol.
Simpler/reused alternative considered: encoding policy/label in session IDs.
Reason rejected: IDs are selectors, not authority or policy, and cannot carry
  mutable writer epochs safely.
Contract changes and links to API/TECH: API.md "Optional policy and launch
  intent"/"Identities and authorization"; TECH.md protocol row.

Verification:
  - Case IDs and exact source commit: none executed; planned V02/V04/V08.
  - Runner OS/architecture and helper/app versions: not run.
  - CI run/job/artifact links or reproducible command: not run; no dispatch.
  - Expected result and observed result: not observed.
  - Relevant numeric measurements: none.
  - Failures found, fixes applied and rerun receipt: none (no build).

Unverified behavior and remaining risks: source was not compiled; account fence,
  catalog/watch, snapshot schema and CLI parser remain unfrozen; advertisement
  must stay off until native proof.
Follow-up tasks / affected gates: S0 remaining bullets; S1/S3; S4 CLI; S5 WSL.
MEMORY update or reason no durable fact changed: MEMORY records the unverified
  slice and the withheld capability.
```

## S1/S3: guarded retention and writer attachment (first slice)

```text
Record ID / date: S1-a / 2026-11-18
Work item and PRD requirements: S1 guarded native ownership; S3 attach/detach/
  takeover and input ordering (FR-02, FR-03, FR-05)
Status: implemented_unverified; writer gate/stop/input semantics superseded by S1-b
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: 87e18c942630e58c91849532fb132323e0500ab5 on
  feat/2.0.0; no PR

Problem and observable before/after:
  Before: losing the control connection always revoked the session, dropped the
  Agent binding and stopped the PTY; every control needed the same connection.
  After: guarded launches keep PTY, reader, screen, Agent wake thread and run
  binding when the connection disappears; attach/detach/takeover fence exactly
  one writer.
Scope: crates/agent_bus/src/companion_terminals.rs, companion_tests.rs,
  ssh_remote.rs, companion_main.rs; app/src/agent_communication/panel.rs and test
  launch literals; shared Unix/Windows terminal registry.
Files and symbols changed:
  companion_terminals.rs: Policy, NativeInput::User{generation}, attach_decision,
    valid_label, Session policy/label/attachment/generation, launch/control/
    disconnect/state
  ssh_remote.rs: capability-gated guarded launch/control
  companion_tests.rs: guarded connectivity test (Unix)
  companion_terminals.rs tests: attach decision and label bounds

Implementation method:
  - Runtime/process ownership before and after: unguarded sessions keep the exact
    previous disconnect path (revoke, binding.take, pty.stop). Guarded sessions
    clear connection_id only; the service holds the native PTY handle and threads,
    so Unix process groups and Windows kill-on-job-close stay alive.
  - Identity, state transitions and concurrency ordering: launch sets
    generation=1 and a new attachment UUID for the creator connection. Transport
    loss keeps both. Attach with exact (generation,id) resumes without takeover;
    a zero/unknown generation returns CONFLICT/STALE unless takeover=true, which
    bumps generation and issues a new id. Detach clears the connection but keeps
    the tuple for the same writer. Input requires the attached connection and is
    queued with the current generation; the writer thread drops queued input whose
    generation was replaced. Stop requires the same project and exact run_id but
    no writer lease. Release removes a session only after native exit and closed
    output.
  - API fields/operations and validation boundaries: see S0 record; attach and
    detach return FEATURE_UNAVAILABLE for connection-owned sessions; control
    rejects nonempty attachment ids that are not UUIDs.
  - Data model/migration and privacy implications: no persisted state; no raw
    screen or launch argv added to any new resource. Reconcile compares policy and
    label so a relaunch cannot silently change them.
  - Native terminal integration and bounded-resource behavior: no new threads,
    parsers or buffers; guarded runs simply avoid stop/revoke on disconnect. The
    existing 256 KiB ring and 300 ms wake loop are unchanged.
  - OS-specific process, IPC and packaging behavior: shared registry; no new IPC
    endpoint. The account service remains one per data directory, so this slice
    does not yet deliver one guardian per execution OS account.
  - Compatibility, failure handling and rollback behavior: legacy clients send
    default fields and keep connection-owned behavior; a superseded writer cannot
    mutate a replacement attachment because the transition is serialized by the
    session mutex and checked against the current tuple.
Decision and rationale: extend the existing terminal registry and connection
  fence rather than add a parallel persistent-terminal subsystem; a takeover
  boolean distinguishes recovery from accidental adoption without treating IDs
  as credentials.
Simpler/reused alternative considered: adopting any same-project connection when
  no connection is attached.
Reason rejected: a cold client that lost its attachment could silently steal a
  deliberately detached writer; explicit takeover keeps one-writer semantics
  auditable.
Contract changes and links to API/TECH: API.md proposed operations and identity
  table; CLI.md writer-lease/takeover rules; TECH.md Companion terminal row.

Verification:
  - Case IDs and exact source commit: none executed; planned V02/V03/V04/V08/V10.
    New test: guarded_run_survives_connection_loss_behind_one_writer_attachment
    (Unix, /bin/sh) plus attach_decision/label unit tests.
  - Runner OS/architecture and helper/app versions: not run.
  - CI run/job/artifact links or reproducible command: not run; no dispatch.
  - Expected result and observed result: not observed.
  - Relevant numeric measurements: none.
  - Failures found, fixes applied and rerun receipt: none (no build); source
    reviewed manually only.

Unverified behavior and remaining risks: not compiled on any target; macOS
  /bin/sh integration, Windows ConPTY continuation, WSL guest service, rapid
  attach races, stop delivery uncertainty and process leak on panic are unproven.
  No catalog/MRU/CLI/snapshot path exists, and guarded runs are unreachable from
  compliant clients until the capability is advertised.
Follow-up tasks / affected gates: S0 catalog/watch; S1 election + proof; S2/S3
  snapshot and reconnect transport; S4 CLI/UI; S5 WSL; S8 V01-V30 and resources.
MEMORY update or reason no durable fact changed: MEMORY records that the guarded
  slice exists unverified and that advertisement stays off pending proof.
```

## S1/S3: review corrections for writer fencing and queued input

```text
Record ID / date: S1-b / 2026-11-18
Work item and PRD requirements: S1 guarded ownership; S3 attachment fence, takeover
  and ordered input (FR-02, FR-03, FR-05)
Status: implemented_unverified
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: 87e18c942630e58c91849532fb132323e0500ab5 on
  feat/2.0.0; no PR

Problem and observable before/after:
  Before (S1-a): guarded Input/Resize/Read/Detach/Release checked only the
  connection, so a stale tuple on the holding transport was accepted after
  takeover; takeover with generation 0 succeeded without observing the writer;
  TerminalStop no longer checked the owning connection for legacy runs; detach and
  transport loss left already queued human input writable.
  After: guarded attachment-bound actions require the connection, exact nonempty
  attachment id and current generation; takeover must fence the observed nonzero
  generation and rejects stale observations; every stop (legacy and guarded) is
  connection-bound, and guarded stop also requires the exact tuple; detach/disconnect
  invalidate queued human input while preserving the resumable attachment tuple.
Scope: crates/agent_bus/src/companion_terminals.rs and companion_tests.rs.
Files and symbols changed:
  companion_terminals.rs: WriteFence{attachment,epoch,gate}, NativeInput::User
    {epoch}, holds_attachment/may_control, attach_decision, control, disconnect,
    Session::state
  companion_tests.rs: guarded test uses the adopted tuple and covers stale input,
    stale takeover and writer-bound stop; legacy connection-gated stop test

Implementation method:
  - Writer identity and input validity are separate: `attachment` is the public
    writer generation; `epoch` invalidates queued input. Takeover, detach and
    disconnect bump `epoch`; only takeover rotates `attachment`.
  - Native human input takes a single writer gate across the epoch check and the
    write. Takeover uses `try_lock`: if a write is in flight it returns CONFLICT
    without rotating, so bytes from the old writer cannot start after a successful
    takeover. Detach and transport loss bump the epoch without waiting, so a blocked
    writer cannot delay disconnect; an already-running write may still complete.
  - Exact tuple reattach is accepted only from the same connection or when the
    writer is detached/lost. A different live connection gets CONFLICT and must
    perform explicit takeover, so a tuple cannot silently displace an active writer.
  - Exact tuple reattach remains idempotent. Takeover with the exact current tuple
    is a no-op reattach; takeover with the observed generation and an empty id
    rotates once; stale or inconsistent tuples return STALE, generation 0 returns
    CONFLICT. A rotated attachment starts its input sequence at 0.
  - Legacy read/resize/input/detach/release keep connection-only gating; legacy
    stop is connection-gated. This slice has no authenticated operator role, so
    guarded stop is also writer-bound; account-wide stop awaits the S4 CLI path.
Decision and rationale: enforce the API.md exact-tuple and expected-generation
  contract at the session lock, and use a separate epoch so canceling queued input
  does not destroy the writer identity needed for reconnect.
Verification:
  - Not compiled or run. Planned unit tests: attach_decision,
    writer_rotation_refuses_an_in_flight_native_write; planned Unix integration:
    guarded_run_survives_connection_loss_behind_one_writer_attachment,
    legacy_stop_requires_the_owning_connection.
  - Failures found, fixes applied and rerun receipt: none; source review only.

Unverified behavior and remaining risks: a native write already in progress when
  transport is lost can still complete without a recorded disposition; snapshots/
  dispositions and real reconnect transport remain future S3 work; Windows/WSL
  paths are unproven; capability stays off.
Follow-up tasks / affected gates: S3 dispositions and write receipts; S4 CLI/UI;
  S8 native regression and V01-V30.
MEMORY update or reason no durable fact changed: no new durable fact beyond the
  existing unverified-slice note.
```

## S1/S3: bounded input receipts and writer fault handling

```text
Record ID / date: S1-c / 2026-11-18
Work item and PRD requirements: S3 ordered input, idempotent retries, dispositions
  and uncertain delivery (FR-03, FR-05)
Status: implemented_unverified
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: 5163030f7535f59f673723c1932563bd186282c3 on
  feat/2.0.0; no PR

Problem and observable before/after:
  Before: Input accepted a fixed next sequence only, never returned a native write
  receipt, conflated queue acceptance with success, used an all-or-nothing
  write_all and treated every writer fault as an unqualified failure. A retry
  after an uncertain reply could not distinguish accepted, written, failed or
  unknown delivery.
  After: guarded input is validated against the exact attachment tuple and a
  bounded per-attachment ledger, accepts only the next sequence, returns the
  accepted or most recent receipt in TerminalState, and distinguishes Accepted,
  Writing, Written, Cancelled, Failed and Unknown. Identical retries of the
  accepted tail are idempotent; different bytes, skips and reversals return
  INPUT_CONFLICT; a missing receipt for a claimed acceptance returns
  DELIVERY_UNKNOWN.
Scope: crates/remote_protocol/proto/remote_server.proto,
  crates/remote_protocol/src/managed.rs, crates/agent_bus/src/ssh_remote.rs,
  crates/agent_bus/src/companion_terminals.rs, and companion_tests.rs.
Files and symbols changed:
  remote_server.proto: InputDisposition, InputReceipt, TerminalState.input_receipt
    (20), MANAGED_INPUT_CONFLICT (10), MANAGED_DELIVERY_UNKNOWN (11)
  managed.rs: guarded tuple coherence, label-only policy, receipt validity and
    receipt-sequence <= accepted-sequence checks
  ssh_remote.rs: strict launch policy echo and guarded input receipt echo; new
    error mappings return conflict
  companion_terminals.rs: InputState/InputRecord/InputLedger, RotateOutcome,
    NativeInput::User{epoch,sequence}, write_counted, writer fault handling,
    Session::receipt, attach/detach/disconnect ledger transitions
  companion_tests.rs: receipt assertions, identical retry, conflicting/skipped
    sequence, adopted-generation receipt, legacy empty tuple, label-only rejection

Implementation method:
  - Each attachment owns a ledger of at most 64 records and at most 64 KiB of
    pending input. A record stores the attachment generation, sequence, random
    receipt ID, SHA-256 digest, byte length, written length and disposition.
    Terminal records are evicted before pending records; capacity errors never
    consume the sequence.
  - The accepted sequence advances only after the bounded channel accepts the
    input. A retry of that sequence is idempotent only when the digest and length
    match the retained record; otherwise it conflicts. A skipped or reversed
    sequence conflicts without enqueueing. Sequence overflow fails the input path
    instead of wrapping.
  - The writer rechecks the attachment epoch under the writer gate, marks the
    record Writing, performs an explicit short-write loop, flushes, then records
    Written or Failed with the actual byte count. A flush failure after a complete
    write is Unknown. A poisoned gate or ledger, a zero write or any other write
    error fails the input path closed and marks queued records Unknown.
  - Detach, transport loss and takeover bump the epoch without waiting for a
    blocked write. Not-yet-started records become Cancelled; an in-flight record
    is Unknown until finish converges it to Written or Failed. Takeover clears the
    old ledger only after rotation succeeds and starts the new attachment at
    sequence 0; exact-tuple reattach keeps the ledger and sequence.
  - Legacy sessions emit guarded=false with an empty attachment tuple and no
    receipt. Labels are rejected unless the launch is guarded. The wire fields are
    additive; no capability is advertised for this slice.
Decision and rationale: make queue acceptance, native write outcome and
  uncertainty separate observable states at the existing session lock, while
  keeping the worker cancellation path non-blocking and the retained state
  bounded by 64 KiB.
Verification:
  - rustfmt --edition 2021 --emit stdout parsed all changed Rust files without a
    syntax error. No cargo build or test was run locally; the workspace has no
    local dependency cache, so compilation and the Unix integration tests remain
    unverified.
  - Planned unit tests: input_ledger_tracks_terminal_and_pending_states,
    input_ledger_bounds_pending_bytes_and_entries,
    counted_writer_reports_short_and_failed_writes,
    writer_rotation_fails_closed_when_the_gate_is_poisoned,
    writer_rotation_refuses_an_in_flight_native_write.
  - Planned Unix integration: guarded_run_survives_connection_loss_behind_one_writer_attachment
    now asserts receipt sequence/generation/ID, identical-tail retry, conflicting
    and skipped sequence rejection and a new receipt generation after takeover.
  - Failures found, fixes applied and rerun receipt: none; source inspection and
    parser checks only.

Unverified behavior and remaining risks: no native or CI compile of the additive
  proto/type changes; in-flight cancellation can only be recorded, not recalled;
  snapshots, reconnect transport, operator-wide stop, Windows/WSL paths and
  V01-V30 remain unproven; capability stays off. A received Unknown receipt is a
  queryable outcome, not proof of non-delivery.
Follow-up tasks / affected gates: S3 reconnect transport and snapshot cut; S4 CLI;
  S8 native regression and V01-V30; cloud CI must compile the proto and both Unix
  tests before this record advances.
MEMORY update or reason no durable fact changed: MEMORY records the receipt ledger
  as part of the unverified guarded slice.
```

## S1: local peer admission and ended-output retention

```text
Record ID / date: S1-d / 2026-11-18
Work item and PRD requirements: S1 account-scoped service boundary and bounded
  ended-output retention (FR-01, FR-09, FR-12)
Status: implemented_unverified
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: 99eaa0eb on feat/2.0.0; no PR

Problem and observable before/after:
  Before: Unix clients were admitted by the private data directory and 0600
  socket mode; the kernel-reported peer UID was collected for process identity but
  not checked for admission. A session whose process ended and whose output closed
  was dropped by the next one-second active sweep, so an exact writer could not
  resume recently ended output. At 32 sessions, even expiring ended sessions could
  block a new launch.
  After: a Unix connection whose kernel-reported UID differs from the service
  effective UID is closed before admission. Windows keeps its existing per-user
  named-pipe DACL. A detached session with no live process and closed output is
  retained for 300 seconds, remains eligible for exact-tuple attach, and is
  removed after expiry. At the 32-session admission ceiling, the oldest ended
  session is evicted first; live, attached or draining sessions are never evicted
  for capacity. Client guarded control now requires the response to echo the exact
  attachment tuple for every action except attach.
Scope: crates/agent_bus/src/companion_service.rs,
  crates/agent_bus/src/companion_terminals.rs,
  crates/agent_bus/src/ssh_remote.rs.
Files and symbols changed:
  companion_service.rs: Unix peer_cred UID comparison against geteuid before slot
    admission and idle reset
  companion_terminals.rs: ENDED_RETENTION, Session.ended_at, retain_session,
    ended_session, prune_sessions, admission-time pruning/eviction, active()
    retention sweep
  ssh_remote.rs: captured expected attachment tuple and rejected non-attach
    guarded replies that do not echo it

Implementation method:
  - Admission still relies first on the 0700 private directory, 0600 Unix socket
    and current-user Windows pipe DACL. On Unix, peer_cred is best-effort because
    not every platform exposes it; when a UID is available and differs from
    geteuid, the connection is refused. The same private endpoint and persisted
    service_id plus service.lock retain single-instance election inside one
    account/transport data directory.
  - retain_session classifies a session as ended only when the process is
    inactive, output is closed and no attachment remains. It records the first
    observation time, keeps the session for ENDED_RETENTION, and clears the
    timestamp when the process, output or attachment becomes live again. A
    poisoned session or output lock is treated conservatively as retained.
  - prune_sessions runs on every active sweep for expiry and before launch
    admission. If the map is at MAX_SESSIONS, it removes ended sessions in
    oldest-ended order until there is room for one new launch. It does not count
    or evict live processes, attached sessions or undrained output.
  - The ceiling remains 32 sessions, each with a lazy 256 KiB output ring, so
    retained raw output is bounded by 8 MiB before other parser/snapshot state.
    The 64 MiB account terminal-data cap is therefore not allocated explicitly in
    this slice; S8 must measure combined RSS and revise if other state grows.
  - The client tuple echo closes a stale or mixed-policy reply instead of
    accepting a state from another attachment. Attach still returns the new tuple
    permitted by rotation or adoption.
Decision and rationale: add defense-in-depth admission without changing the
  existing endpoint/election topology, and retain only the minimum ended state
  needed for an exact resume for a bounded window. Evict ended output before
  refusing a new session so history does not starve live work.
Verification:
  - rustfmt --edition 2021 parsed the changed files; no syntax error. The
    retention helper was copied into an isolated rustc program with mock PTY/output
    types and its expiry path passed; this is not a workspace build.
  - No cargo build or test was run locally; the workspace has no dependency cache.
    The Unix peer-UID path, real PTY retention, eviction at 32 sessions and Windows
    DACL behavior remain unrun.
  - Failures found, fixes applied and rerun receipt: none; parser and isolated
    helper checks only.
Unverified behavior and remaining risks: peer_cred may be unavailable, leaving
  directory/socket permissions as the sole Unix boundary; the Windows peer token
  is not rechecked after the DACL, only the DACL itself; retention timing and PTY
  active/closed transitions are unmeasured; there is no account catalog, list,
  inspect or watch gap yet, so retained output cannot be discovered by a new
  client; SSH and WSL still use separate data directories and are not unified by a
  logical-scope election; daemon-owned stop batches, the 64 MiB accounting cap and
  idle-exit timing with retained output remain unimplemented or unproven.
Follow-up tasks / affected gates: S0 account catalog/watch contract; S1 unified
  logical scope and 60s idle/retention proof; S4 CLI list/resume; S7/S8 resource
  measurement and native peer tests.
MEMORY update or reason no durable fact changed: MEMORY records guarded retention
  and input receipts; the 300s ended-output window and Unix peer UID check are
  implementation details recorded here and do not change the scope decision.
Superseded by: S1-e (fail-closed peer identity, generation-keyed receipts,
  between-write cancellation and multiplexed-connection error handling).
```

## S1/S3: review closure for guarded input and peer admission

```text
Record ID / date: S1-e / 2026-11-18
Work item and PRD requirements: close P1/P2 review findings in the guarded input,
  attachment and local admission path (FR-01, FR-03, FR-05, FR-09, FR-12)
Status: implemented_unverified
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: efe9fd12 on feat/2.0.0; no PR

Problem and observable before/after:
  Before: the receipt ledger was keyed only by sequence, so an old writer message
  dequeued after takeover could cancel the same sequence on the new attachment.
  Unknown delivery was treated as evictable history. write_counted ignored
  cancellation until the full buffer was written, and queued input skipped after
  revocation or an I/O fault could remain Accepted indefinitely. Unix peer
  credential failure admitted the connection, Windows admission relied only on the
  pipe DACL, legacy ended sessions could extend service life, MANAGED_DELIVERY_UNKNOWN
  collapsed into ordinary conflict, and a session-level stale reply closed the
  multiplexed connection for unrelated sessions.
  After: ledger lookup, reservation, cancellation and completion include the
  attachment generation; Unknown is retained as non-evictable evidence and causes
  capacity backpressure instead of silent loss. Counted writes check cancellation
  between writes, partial cancellation records Unknown, zero-byte cancellation
  records Cancelled, and skipped queue entries are settled. Ended retention is
  guarded-only. Unix admission requires credentials to resolve and match the
  service effective UID; Windows rechecks the named-pipe client process token SID
  against the current user in addition to the per-user DACL. DELIVERY_UNKNOWN is a
  distinct client error, and session or attachment stale replies no longer close
  the connection shared by other sessions. Legacy states without `guarded` must
  present either an empty attachment ID with generation zero or a valid non-empty
  tuple.
Scope: crates/agent_bus/src/companion_service.rs,
  crates/agent_bus/src/companion_terminals.rs,
  crates/agent_bus/src/ssh_remote.rs,
  crates/agent_bus/src/windows_pipe.rs,
  crates/remote_protocol/src/managed.rs.
Files and symbols changed:
  companion_terminals.rs: NativeInput::User.attachment, generation-keyed
    InputLedger::{find,remove,mark_writing,cancel,finish}, InputState::terminal,
    WriteEnd, write_counted cancellation predicate, writer queue settlement,
    Session::receipt, guarded-only retain_session/ended_session
  companion_service.rs: fail-closed Unix UID/pid admission, Windows current-user
    versus client-process SID comparison before slot admission
  windows_pipe.rs: client_user_sid(pid), process handle ownership, current-user
    client test
  ssh_remote.rs: ConnectionError::DeliveryUnknown, no close on managed application
    errors or session-level stale replies, explicit receipt-missing error
  managed.rs: legacy `guarded=None` tuple coherence

Implementation method:
  - Ledger records are unique by (attachment_generation, input_sequence). A stale
    queue entry can only cancel or complete its own generation. Unknown records
    count toward the 64-receipt ceiling but not the pending-byte budget, so they
    cannot be evicted while delivery remains unresolved; callers receive Capacity
    instead of losing the evidence.
  - write_counted checks the writer epoch or revocation flag before each write
    syscall. A completion after partial bytes is Unknown because those bytes may
    have reached the PTY; a pre-write cancellation is Cancelled and does not flush.
    Entries skipped when the queue is already revoked or failed transition through
    the same ledger cancellation path rather than staying pending.
  - Retention records ended_at only for Policy::Guarded. Legacy disconnected ended
    sessions are removed on the next sweep and cannot keep the service alive.
  - Unix accept calls peer_cred once, rejects credential retrieval failure and UID
    mismatch, and reuses that credential for the pid passed to native admission.
    Windows accepts only when the client PID and both SIDs resolve and match,
    keeping the DACL as an additional boundary.
  - HostClient still closes on transport desynchronization (timeout, wrong request
    ID, malformed or non-managed reply), but a managed StaleAttachment,
    IncompatibleVersion or DeliveryUnknown reply is returned to the caller without
    killing unrelated sessions. Input without a matching receipt is DeliveryUnknown.
Decision and rationale: preserve evidence of uncertain input and one terminal's
  stale attachment without weakening stream-integrity closure or process identity.
  Capacity backpressure is preferable to silently discarding an unresolved write.
Verification:
  - rustfmt --edition 2021 parsed all changed Rust files with no syntax error;
    git diff --check reported no whitespace errors.
  - No cargo check, build or test was run locally: the workspace target is empty
    and the offline registry lacks required crates (first failure: aho-corasick
    1.1.3). Unit tests were extended but not executed.
  - Failures found, fixes applied and rerun receipt: the first write_counted
    cancellation test expected two bytes; cancellation is checked before each
    write, so the correct expectation is one byte and the test was corrected.
    No rerun receipt exists.
Unverified behavior and remaining risks: no native/CI build or SSH/WSL test
  exercised these paths. Unknown-ledger capacity can eventually backpressure a
  session with unresolved delivery rather than reclaiming old evidence. Windows
  SID comparison and Unix peer_cred behavior are unrun on their native targets.
  The capability remains unadvertised, and no catalog, list, inspect, watch gap,
  reconnect transport, operator stop or resource measurement exists. Transport
  desynchronization still closes the whole connection, which is intentional.
Follow-up tasks / affected gates: fixed-SHA CI and native Unix/Windows admission
  tests; S1 unified election/catalog; S3 reconnect and snapshot ordering; S4 CLI.
MEMORY update or reason no durable fact changed: MEMORY is updated with the
  fail-closed peer checks, generation-keyed receipts and non-evictable unknown
  delivery; scope and capability gating are unchanged.
```

## S0/S1: bounded catalog index prototype

```text
Record ID / date: S1-f / 2026-11-18
Work item and PRD requirements: S0 account catalog/list/watch contract and S4
  list/resume selection rules, implemented first as a pure bounded index
Status: implemented_unverified
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: feat/2.0.0 source commit after this record; no PR

Problem and observable before/after:
  Before: retained sessions had no discoverable metadata; a later client could
  not enumerate retained runs, page them stably, observe changes since a revision
  or choose an MRU resume target without scanning live registry state that is
  connection-owned. There was no representation separating session ID, run ID,
  project scope, session kind and state age.
  After: a self-contained Catalog maintains bounded nonsecret SessionMetadata
  entries (session/run IDs, scope, project, label, kind, state, state age,
  creation/activity sequence) with a stable descending cursor over
  (created_sequence, session_id), boot- and filter-bound cursors, a
  revision-scoped bounded watch ring that reports an explicit gap instead of
  pretending contiguous history, MRU resume selection restricted to live runs,
  and 32-entry/64-event ceilings. It performs no I/O and grants no rights.

Scope: crates/agent_bus/src/companion_catalog.rs (new),
  crates/agent_bus/src/companion.rs (module registration only).

Files and symbols changed:
  companion_catalog.rs: SessionKind, SessionState, SessionMetadata,
    CatalogEntry, CatalogChange, CatalogEvent, CatalogCursor, WatchCursor,
    CatalogPage, WatchPage, Inspection, CatalogError, PageQuery, InspectQuery,
    Catalog::{upsert,touch,remove,page,inspect,watch,resume}, tests
  companion.rs: `pub(crate) mod catalog;`

Implementation method:
  - Entries are keyed by session ID and merge on upsert so metadata updates do
    not reset creation sequence or state_since; state_since changes only when the
    state value changes. Insertion is capped at 32 sessions.
  - Pages sort by creation sequence descending with session ID as a deterministic
    tie-break. A cursor carries boot ID, scope and project filter; a mismatch or
    an unknown boot ID is rejected. The response reports whole-filter totals and
    a real next cursor, independent of page length, so insertions do not shift an
    already-consumed page.
  - Watch is revision- and boot-scoped. Events live in a bounded ring; when the
    requested revision has been evicted the result reports gap=true with no
    events so the client must relist rather than assume continuity.
  - Resume prefers the highest foreground activity sequence, then newest
    creation, then session ID, and excludes ended/interrupted/unknown states.
    touch requires an existing live entry before consuming an activity sequence.
  - Inspection requires the exact scope, project and optional run; a foreign or
    mismatched record returns NotFound. Listing/inspection perform no writer
    acquisition and create no snapshots.

Decision and rationale: freeze pagination, watch gap semantics and MRU tie-breaks
  as pure deterministic logic before account election and wire exposure, so the
  later registry wiring cannot quietly change client-visible ordering rules.

Verification:
  - The module compiled as a standalone test crate with a local uuid stub:
    `rustc --edition 2021 --test crates/agent_bus/src/companion_catalog.rs
    --extern uuid=<stub>`; all 7 tests passed:
    stable pages across insertions and page bound; cursor boot/filter binding;
    revision-scoped watch ring gap; MRU resume excluding ended; state age only on
    transition; 32-entry/64-event bounds; foreign scope/run rejection.
  - rustfmt --edition 2021 parsed the new module; git diff --check is clean.
  - No workspace cargo check, build or test was run: the offline registry lacks
    required crates (first failure: aho-corasick 1.1.3). This is not evidence that
    the module links into the workspace crate.
  - Failures found, fixes applied and rerun receipt: the first isolated harness
    used include!, which cannot host the module's inner doc comment and
    #![allow(dead_code)] at harness scope (E0753); compiling the file directly as
    its own test crate succeeded and produced the passing run above.

Unverified behavior and remaining risks: the catalog is not constructed by
  Terminals, not populated by launch/attach/takeover/exit transitions, not
  persisted across guardian boots, and not exposed through any account-admitted
  operation; therefore no client can use it. Account-wide scope identity and
  unified SSH/WSL election remain absent, so scope strings are caller-supplied in
  tests only. `#![allow(dead_code)]` must be removed when the registry wiring
  lands. Resource accounting (64 MiB ceiling, snapshot serializers) is untouched.
Follow-up tasks / affected gates: S1 unified account guardian election and
  logical scope; then wire launch/attach/exit transitions into Catalog and add the
  account-catalog/change-watch wire messages without advertising
  `guarded_remote_sessions_v1`; S4 list/resume consume pages after CLI design.
MEMORY update or reason no durable fact changed: MEMORY records the bounded
  catalog prototype and that it remains unwired/unexposed; no scope or capability
  decision changed.
Verification superseded by: S1-g (the workspace crate now compiles with the
  catalog module linked; its isolated tests still pass).
```

## S1/S3: real build restoration and host verification

```text
Record ID / date: S1-g / 2026-11-18
Work item and PRD requirements: replace source-only inspection with an actual
  compile and test receipt for the guarded slice (FR-01, FR-03, FR-05, FR-09)
Status: verified_host_only
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: feat/2.0.0 commit after this record; no PR

Problem and observable before/after:
  Before: every earlier slice claimed `implemented_unverified` because
  `cargo check --offline` could not resolve dependencies and `protoc` was absent;
  two test expectations had never executed, and `remote_protocol` did not compile.
  After: dependencies were fetched online, `protoc` 36.2 was installed via
  Homebrew, a real compile error was fixed, stale test expectations were corrected,
  and the full `warp-agent-bus` and `remote_protocol` suites pass on macOS arm64.

Scope: crates/remote_protocol/src/managed.rs,
  crates/agent_bus/src/companion_terminals.rs,
  crates/agent_bus/tests/managed_agent.rs; toolchain only otherwise.

Files and symbols changed:
  managed.rs: corrected generated-enum reference to
    InputDisposition::Unspecified in valid_terminal_state
  companion_terminals.rs: counted write cancellation test expectation changed to
    the two bytes the short writer actually transfers before the next check
  managed_agent.rs: identical-tail input replay now expects success; a separate
    different-bytes replay asserts ManagedInputConflict

Implementation method:
  - Ran `cargo check -p warp-agent-bus` online. The build failed inside
    `remote_protocol` with E0599: a stale prost enum variant name. The generated
    enum uses `Unspecified`, not `InputDispositionUnspecified`; fixed the
    validation predicate.
  - `protoc` was missing, so `brew install protobuf` (36.2) was used. This is a
    local developer tool; it is not added to the repository or release scripts.
  - `cargo test -p warp-agent-bus` exposed one failing unit test and one failing
    integration test, both expectations rather than logic:
      * The short writer transfers a 2-byte chunk before cancellation is observed,
        so the correct byte count is 2.
      * Input retry with the same sequence and identical bytes is defined as
        idempotent by the API contract, so the integration test now asserts a
        successful TerminalState for the retry and uses changed bytes for the
        conflict case.
  - Transient Cargo.lock reordering produced by dependency resolution was
    reverted; no dependency versions were changed.

Decision and rationale: keep the contract (idempotent same-byte tail retry,
  conflicting bytes rejected) and correct the stale tests, rather than weakening
  the implementation to satisfy old expectations.

Verification:
  - `cargo check -p warp-agent-bus`: finished successfully (warnings only).
  - `cargo test -p warp-agent-bus --lib catalog::`: 7 catalog tests passed when
    linked and run through the workspace crate.
  - `cargo check -p warpai --bin warpai`: finished successfully in 5m35s
    (pre-existing library warnings only), so the application binary links the
    guarded slice.
  - `cargo test -p warp-agent-bus -p remote_protocol`: all executed tests passed:
    16 remote_protocol unit tests; 231 agent_bus library tests (1 ignored);
    integration suites 20, 7, 6, 2 and 6 passed; 8 child/fixture tests remain
    ignored; zero failures.
  - The ignored tests require the owned Companion PTY child, real WSL or another
    platform and remain NOT RUN.
  - Failures found, fixes applied and rerun receipts: both test corrections and
    the E0599 fix were rerun; the final full run reported zero failures.
  - Host: macOS 24.6 arm64. SSH remote hosts, Windows, Linux and WSL were not
    exercised; V01-V30 remain NOT RUN.

Unverified behavior and remaining risks: passing host tests do not prove remote
  SSH/WSL lifetime, Windows admission, 300s retention timing, resource ceilings,
  snapshot import or reconnect transport. The application binary compiles, but no
  live desktop/remote scenario was exercised. `guarded_remote_sessions_v1` remains
  unadvertised, so no client path reaches these operations.

Follow-up tasks / affected gates: Windows/Linux CI at a fixed SHA; real SSH and
  WSL acceptance; then account election, snapshot and CLI slices.
MEMORY update or reason no durable fact changed: MEMORY records that the guarded
  slice now compiles and passes host tests, while remote acceptance remains unrun.
```

## S0/S1: session-kind wire freeze and validation

```text
Record ID / date: S1-h / 2026-11-18
Work item and PRD requirements: freeze additive session kind and reject
  policy/kind mismatches before native spawn (FR-03, FR-05, FR-09)
Status: verified_host_only
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: feat/2.0.0 commit after this record; no PR

Problem and observable before/after:
  Before: `TerminalLaunch` had no session-kind field, so a guarded shell/command
  and a guarded Agent were distinguishable only by `agent_program`, and the
  planned catalog could not record the actual kind. A guarded launch without an
  Agent binding was accepted without an explicit declaration.
  After: `TerminalSessionKind` (UNSPECIFIED=0, AGENT=1, SHELL=2, COMMAND=3) is an
  additive `TerminalLaunch.session_kind = 11`. A guarded launch requires a valid
  kind; AGENT requires a valid nonblank `agent_program`; SHELL/COMMAND require its
  absence and create no RunBinding. An unguarded launch must omit the field.
  Absent, zero, unknown and mismatched values are rejected before spawn, and the
  kind participates in launch replay equivalence.

Scope: crates/remote_protocol/proto/remote_server.proto,
  crates/agent_bus/src/companion_catalog.rs,
  crates/agent_bus/src/companion_terminals.rs,
  crates/agent_bus/src/ssh_remote.rs, launcher/fixture call sites,
  specs/2.0.0/API.md, specs/2.0.0/TECH.md.

Files and symbols changed:
  remote_server.proto: TerminalSessionKind, TerminalLaunch.session_kind = 11
  companion_catalog.rs: SessionKind::{from_proto,proto}
  companion_terminals.rs: launch_kind validation, replay comparison includes
    session_kind, guarded kind matrix test
  ssh_remote.rs: client refuses to transmit session_kind without
    guarded_remote_sessions_v1
  panel.rs, companion_main.rs, companion_tests.rs, managed_agent.rs,
    ssh_companion.rs, wsl_workspace.rs: explicit legacy omission; guarded fixture
    declares Shell and asserts changed-kind replay is a conflict

Implementation method:
  - Proto field is optional so legacy callers omit it and decode as None.
  - Server validation is a pure `launch_kind(guarded, kind, has_agent)` helper so
    the matrix is unit-testable without spawning a PTY. `AGENT` requires an agent
    binding; `SHELL`/`COMMAND` forbid one; explicit kind on an unguarded launch is
    invalid.
  - Replay conflict compares the whole original launch including kind, so a
    retried session ID with a different kind cannot silently adopt the run.
  - The client gate reuses the existing capability check; the capability remains
    unadvertised, so no production client can transmit the new kind yet.

Decision and rationale: make shell/command Agent-right-free at the earliest
  validation point and keep legacy semantics byte-identical by requiring omission.

Verification:
  - `cargo test -p warp-agent-bus -p remote_protocol`: all executed tests passed
    (232 library tests, 16 remote_protocol, all integration suites; 8 ignored
    fixtures), including the new kind matrix and changed-kind replay conflict.
  - `cargo check -p warpai --bin warpai`: finished successfully (42s) after the
    app call-site edit.
  - Failures found, fixes applied and rerun receipt: the first test import used
    `crate::companion_catalog`, which is a child of `companion`; corrected to
    `crate::companion::catalog::SessionKind` and reran successfully.

Unverified behavior and remaining risks: the kind is not echoed in TerminalState
  and the catalog is not yet wired, so readback relies on future catalog metadata.
  No remote SSH/WSL launch exercised the field. Capability remains unadvertised.

Follow-up tasks / affected gates: catalog wiring and account election per S1-i;
  native acceptance before advertisement.
MEMORY update or reason no durable fact changed: MEMORY records the frozen kind
  field and pre-spawn validation; scope and capability gating are unchanged.
```

Record ID / date: S1-i / 2026-11-18
Work item and PRD requirements: one account-global Unix guardian with route-scoped
  admission and logical scope isolation (FR-04, FR-05, FR-11, FR-12)
Status: verified_host_only
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: feat/2.0.0 commit after this record; no PR

Problem and observable before/after:
  Before: `companion_service` bound one legacy per-scope socket and had no
  account-level election, no private IPC identity preface and no route-derived
  capabilities. A process could not prove it was talking to the account guardian.
  After: a `--guardian-service` process derives its root from the passwd home
  (never HOME/XDG), holds one `service.lock` for the account, binds private
  per-route sockets (`ssh.sock`, and `wsl.sock` on WSL Linux builds) under
  `/tmp/warpai-guardian-<euid>/`, and exchanges a bounded `WAG1` preface carrying
  IPC major, route, guardian UUID and non-nil boot UUID after a fail-closed peer
  UID check. A losing or already-served candidate exits without binding; a
  responsive incompatible owner is never unlinked; a legacy scope lock makes only
  that route unavailable. Each connection resolves its route from the accepting
  listener, so an SSH connection cannot use WSL-only capabilities or guest
  binding.

Scope: crates/agent_bus/src/companion_guardian.rs,
  crates/agent_bus/src/companion.rs, companion_service.rs,
  companion_terminals.rs, companion_main.rs, companion_tests.rs,
  companion_files_tests.rs.

Files and symbols changed:
  companion_guardian.rs (new): root/runtime/scope paths, passwd-home lookup,
    preface encode/verify, `probe_route`, `connect`, `serve`, `serve_at`,
    per-route scope locks, shared `Terminals`, per-route `Projects`
  companion.rs: `Route`, `Companion::route`, `wsl_route`, `capabilities`,
    route-derived Initialize, chunk/GuestMcpBind admission, `serve_guardian`,
    `serve_channel(..., route)`
  companion_terminals.rs: launch takes `allow_working_directory` instead of a
    compile-time WSL branch
  companion_main.rs: hidden `--guardian-service` entry
  companion_tests.rs: SSH route hides WSL capabilities and rejects GuestMcpBind
  companion_service.rs, companion_files_tests.rs: legacy `route = None`

Implementation method:
  - Election is lock-first: account lock, then scope locks, then catalog, then
    listeners. A would-block account lock is a clean candidate exit; a would-block
    scope lock skips that route and leaves the legacy owner untouched.
  - Existing endpoint handling probes before replacement and removes only an
    owned Unix socket that refuses connections; handshake failure or timeout is
    treated as an incompatible live owner and fails closed.
  - Route is a server-side property of the accepting listener; no protocol field
    can switch it after admission, and legacy stdio/service paths pass `None`.

Decision and rationale: a single account process with separate logical scopes
  keeps one native owner without merging SSH/WSL Stores or capabilities.

Verification:
  - `cargo test -p warp-agent-bus --lib guardian`: 8 passed (guardian and catalog
    modules), including simultaneous candidates, shared boot identity across
    routes, stale owned socket reclaim, legacy scope contention, invalid peer
    preface and SSH route admission.
  - `cargo test -p warp-agent-bus --lib`: 241 passed, 1 ignored.
  - `cargo check -p warp-agent-bus` and `cargo check -p warpai --bin warpai`:
    finished successfully on macOS arm64.
  - Failures found, fixes applied and rerun receipt: the invalid-preface test
    initially left the client write side open, so the server correctly waited for
    the full preface; the test now shuts down its write half and passes. An
    accidental workspace-wide `cargo fmt` was fully reverted and only the six
    intentionally edited files were reapplied and retested.

Unverified behavior and remaining risks: no client connects through the guardian
  sockets yet, so `guardian::connect` has no production caller; guardian sockets
  are not exercised through real SSH/WSL; Windows has no election implementation
  and must fail closed; capability stays unadvertised.

Follow-up tasks / affected gates: wire guarded launches to `guardian::connect`
  and session transitions to the durable catalog; S5 WSL lifetime and S8 native
  acceptance remain open.
MEMORY update or reason no durable fact changed: MEMORY records the account
  election and route isolation as host-verified design only.

Record ID / date: S1-j / 2026-11-18
Work item and PRD requirements: durable bounded catalog under the guardian lock
  with explicit identity validation and no auto-resume (FR-16, FR-17)
Status: verified_host_only
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: feat/2.0.0 commit after this record; no PR

Problem and observable before/after:
  Before: the catalog prototype was in-memory only; there was no durable session
  evidence across guardian restarts and no schema identity check.
  After: `guardian/catalog.sqlite3` is opened only while holding `service.lock`,
  created with `application_id=0x57474341` and `user_version=1`, and stores only
  bounded nonsecret metadata in `guardian_catalog_sessions`. A pre-existing file
  with a different identity or missing table is refused without modification.
  Startup marks prior `live` rows `unknown` in one statement, inserts evict only
  resolved `ended` rows, and unresolved capacity refuses admission.

Scope: crates/agent_bus/src/companion_guardian_catalog.rs (new),
  crates/agent_bus/src/companion.rs, companion_guardian.rs.

Files and symbols changed:
  companion_guardian_catalog.rs: `Catalog::{open,begin_boot,insert,update_state,
    touch_mru,remove,list}`, `Record`, `State`
  companion_guardian.rs: opens the catalog after the account lock and records the
    current boot observation
  companion.rs: module declaration

Implementation method:
  - Diesel raw SQL over the existing bundled SQLite; no ORM schema or migration.
  - Validation runs before any write; creation sets identity and the single v1
    table. Label and sequence values are bounded and control characters refused.
  - MRU updates are a separate explicit call so list/observation never reorder
    resume targets. No argv, environment, screen, output or token is stored.

Decision and rationale: durable metadata is required for external list/inspect
  after reconnect, while terminal bytes and drafts remain memory-only.

Verification:
  - `cargo test -p warp-agent-bus --lib guardian_catalog`: 3 passed (create/boot
    marking/MRU ordering, refuse unknown identity without overwrite, bounded
    eviction and unresolved capacity refusal).
  - Covered by the full 241-test host suite and both `cargo check` receipts above.
  - Failures found, fixes applied and rerun receipt: missing Diesel trait imports
    and two type-inference errors were corrected, then the suite passed.

Unverified behavior and remaining risks: the store is not yet called by terminal
  create/attach/detach/exit transitions, so no session row is produced in
  production; no concurrency test exercises two writers because the account lock
  is the concurrency boundary; no migration exists by design.

Follow-up tasks / affected gates: wire transitions with launch-before-ack and
  MRU only on foreground acquisition; then account-fenced pages and CLI.
MEMORY update or reason no durable fact changed: MEMORY records the catalog file,
  identity and boot-marking rule.

Record ID / date: S1-k / 2026-11-18
Work item and PRD requirements: record guarded session lifecycle before
  acknowledgement and keep MRU explicit (FR-16, FR-17)
Status: verified_host_only
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: feat/2.0.0 commit after this record; no PR

Problem and observable before/after:
  Before: the durable catalog existed but no guarded launch produced a row, so
  external discovery would have had nothing to list.
  After: a guarded launch inserts or updates its metadata before the launch reply
  is written; a guarded read that observes a closed process marks the row ended
  with its exit status; a successful attach updates MRU. Connection-owned and
  unguarded launches write nothing. A replay of an ended session cannot resurrect
  the row to `live`. Guardian connections share one catalog handle; legacy stdio
  and service connections pass `None`.

Scope: crates/agent_bus/src/companion.rs, companion_guardian.rs,
  companion_guardian_catalog.rs, companion_tests.rs,
  companion_files_tests.rs, companion_service.rs.

Files and symbols changed:
  companion.rs: `CatalogHandle`, `Companion::catalog`, `record_catalog_launch`,
    `record_catalog_control`, dispatch wiring for Launch/Control, serve_channel
    catalog parameter
  companion_guardian_catalog.rs: `contains`, `upsert` with ended-state guard and
    peak-preserving MRU
  companion_guardian.rs: catalog wrapped in `Arc<Mutex<_>>` and passed per
    connection
  companion_tests.rs: guarded lifecycle records live, kind, policy, label and
    ended exit status; catalog upsert retry test

Implementation method:
  - Recording happens after the native launch succeeds but before the reply, so a
    failed insert returns `ManagedUnavailable` and a retry reconciles the existing
    run and retries the same upsert.
  - Only `TerminalState.guarded == Some(true)` is persisted, matching the catalog's
    resume purpose; connection-owned behavior and Store semantics are unchanged.
  - Ended state is observed from `output_closed && processes_active == false` on a
    guarded read; no process table scan or synthetic exit is introduced.
  - Attach is the only MRU trigger; read/list/observation never reorder rows.

Decision and rationale: explicit, fenced observations avoid a second lifecycle
  source of truth while still giving reconnect a durable bounded index.

Verification:
  - `cargo test -p warp-agent-bus --lib guarded_session_lifecycle`: passed,
    recording a live guarded shell row and its ended exit status.
  - `cargo test -p warp-agent-bus --lib guardian`: 9 passed including upsert
    deduplication and peak MRU retention.
  - `cargo test -p warp-agent-bus --lib`: 243 passed, 1 ignored.
  - `cargo check -p warpai --bin warpai`: finished successfully.

Unverified behavior and remaining risks: natural exit without any subsequent
  guarded read is not observed until a read/release occurs; retention expiry and
  capacity eviction update the in-memory registry but not the catalog row; no
  page/inspect/watch protocol or CLI exists; no production client connects
  through the guardian sockets.

Follow-up tasks / affected gates: account-fenced list/inspect/watch projection,
  then private CLI list/resume/stop; S5/S8 native proof remains required.
MEMORY update or reason no durable fact changed: MEMORY records that guarded
  launch/attach/exit now populate the durable catalog.

Record ID / date: S1-l / 2026-11-18
Work item and PRD requirements: fail-closed client admission and an installed
  helper bridge for guarded transport (FR-3, FR-5, FR-12)
Status: verified_host_only
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: feat/2.0.0 commit after this record; no PR

Problem and observable before/after:
  Before: the client probe connected to any socket, spawned a service on any
  failure including incompatible or denied owners, and the dispatcher awaited
  each bounded preface inline so one stalled peer delayed accepts and idle
  accounting. Startup with no admitted route stayed alive without listeners.
  After: the client verifies the runtime directory is private and owned, the
  socket is an owned 0600 socket, and the kernel peer UID matches; absent routes
  are distinguished from incompatible/denied owners; probe-only clients never
  spawn, and the ensure path spawns at most once only for an absent route under
  one deadline. Prefaces run in the per-connection task. Zero admitted routes
  fail startup. `warpai-companion --guardian-stdio ssh|wsl` is an internal
  helper that bridges bounded stdio to `guardian::connect`; the GUI never passes
  it, so only the installed helper can start `--guardian-service`.

Scope: crates/agent_bus/src/companion_guardian.rs, companion.rs,
  companion_main.rs.

Files and symbols changed:
  companion_guardian.rs: `private_directory`, `private_socket`,
  `exchange_preface`, `probe_live`, `probe_route` peer-credential check,
  `route_absent`, `probe`, `connect_inner`, zero-route startup refusal, spawned
  preface handling
  companion.rs: `guardian_route`, `serve_guardian_stdio`
  companion_main.rs: hidden `--guardian-stdio <route>` branch

Implementation method:
  - Client checks run before connect; a foreign or loose socket fails closed and
    is never repaired by the client.
  - The owner's stale check deliberately uses `probe_live`, which tests only
    responsiveness so a private socket owned by the account can be reclaimed.
  - Incompatible or denied owners are never unlinked, matching the election
    invariant.
  - The bridge copies with fixed buffers; managed framing remains authoritative.

Verification:
  - `cargo test -p warp-agent-bus --lib guardian`: 14 passed, including loose
    socket/directory rejection, closed route names, zero-route failure and a
    stalled preface not blocking another client.
  - `cargo test -p warp-agent-bus --lib`: 249 passed, 1 ignored.
  - `cargo check -p warpai --bin warpai`: finished successfully.

Unverified behavior and remaining risks: no production caller selects the
  helper path; the GUI binary is unchanged and still cannot handle
  `--guardian-service`; on-demand scope admission, same-scope legacy cutover
  forwarding and guest-MCP retargeting are not implemented; Linux SSH-first
  election of a WSL-incapable image and verified guest namespaces remain open.

Follow-up tasks / affected gates: client transport selection, legacy cutover
  forwarding, scope-on-demand admission; V03/V05 remain NOT RUN.
MEMORY update or reason no durable fact changed: MEMORY records the helper
  bridge and the absence of a production opt-in.

Record ID / date: S1-m / 2026-11-18
Work item and PRD requirements: one lifecycle authority and one serial sequence
  owner (FR-16, FR-17)
Status: verified_host_only
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: feat/2.0.0 commit after this record; no PR

Problem and observable before/after:
  Before: `created_seq`/`mru_seq` were wall-clock milliseconds converted to
  `u64`, so ties and backwards clocks could reorder rows; the catalog accepted
  updates by session ID only; natural exit was persisted only when a client
  later read or released; retention and eviction could drop a row that was never
  reported ended.
  After: the single account owner allocates checked monotonic signed sequences
  seeded from the durable maximum; rows are opened only after identity, exact
  schema columns and every row invariant validate, and creation is
  transactional. Upsert and state updates require the full immutable identity
  (session, run, boot, service, project, policy, kind), ended rows cannot be
  resurrected, and MRU is allocated on foreground attach only. The terminals
  registry is the lifecycle authority: on the guardian's existing one-second
  tick it reports native process exit plus drained output once per exact run,
  and suppresses re-observation only after the durable write succeeds.

Scope: crates/agent_bus/src/companion_guardian_catalog.rs,
  companion_terminals.rs, companion_guardian.rs, companion.rs,
  companion_tests.rs.

Files and symbols changed:
  companion_guardian_catalog.rs: `Catalog::next_seq`, `allocate_seq`,
  `revision`, `find`, `update_state` with exact identity and ended guard,
  `touch_mru` self-allocating, strict `open` validation, stable list ordering
  companion_terminals.rs: `Session::{boot_id, service_id, project_id,
  end_reported}`, `EndedSession`, `Terminals::ended_guarded`,
  `Terminals::mark_end_reported`
  companion_guardian.rs: `wall_millis`, `sync_ended`, maintenance-tick and
  shutdown sync
  companion.rs: launch allocates one sequence, control uses exact identity

Implementation method:
  - DB writes happen on the guardian tick after locks are released per session
    read; a failed write leaves the session eligible for the next bounded tick.
  - No new polling source was added: the existing idle tick is reused, and
    `active()` runs after `sync_ended` so the 300-second retention window cannot
    erase an unreported row first.
  - Ended rows keep metadata but stop accepting state transitions.

Verification:
  - `cargo test -p warp-agent-bus --lib guardian`: 14 passed, including serial
    sequence reseeding, identity conflict, ended non-resurrection and bounded
    capacity.
  - `cargo test -p warp-agent-bus --lib`: 249 passed, 1 ignored.
  - `cargo check -p warpai --bin warpai`: finished successfully.

Unverified behavior and remaining risks: catalog pruning and TTL expiry are not
  independently durable (rows stay until the 64-row cap); DB failure retries are
  bounded only by guardian lifetime; cluster reaping and output TTL do not yet
  write separate catalog evidence; no native remote run.

Follow-up tasks / affected gates: prune/TTL evidence policy and DB-fault tests;
  V11/V12 remain NOT RUN.
MEMORY update or reason no durable fact changed: MEMORY records the single
  serializer and terminals-owned exit observation.

Record ID / date: S1-n / 2026-11-18
Work item and PRD requirements: account-fenced catalog discovery and a real
  host fixture (FR-18, FR-19, FR-21)
Status: verified_host_only
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: feat/2.0.0 commit after this record; no PR

Problem and observable before/after:
  Before: the catalog was durable but unreachable; no message could list or
  inspect rows, and no test exercised the real guardian listener through a
  managed handshake.
  After: additive proto tags 10-13 define `AccountCatalogList`,
  `AccountCatalogInspect`, `AccountCatalogWatch` and `AccountStop`, with
  `ManagedInitialized.account_fence = 7`. List and inspect are implemented for
  guardian-admitted connections only, bind cursors to boot and catalog revision,
  cap pages at 32, filter by project UUID, and project only non-secret metadata.
  Watch and stop parse but return `ManagedFeatureUnavailable`. Legacy services
  advertise no account fence. A host fixture drives the real Unix listener with
  a real PTY: preface, initialize with account fence, project open, guarded
  launch, catalog list, bridge drop, same-boot reconnect, exact attach, stop and
  ended observation without any read.

Scope: crates/remote_protocol/proto/remote_server.proto,
  crates/remote_protocol/src/managed.rs, crates/agent_bus/src/companion.rs,
  companion_guardian.rs, companion_tests.rs, ssh_remote.rs.

Files and symbols changed:
  remote_server.proto: `AccountFence`, `AccountCatalogFilter`,
  `AccountCatalogList/Inspect/Watch`, `AccountCatalogRow/Page/Event/Events`,
  `AccountStop`, `AccountStopTarget/Receipt/Receipts`, request/response tags
  10-13, `ManagedInitialized.account_fence`
  managed.rs: `valid_account_fence`
  companion.rs: `catalog_row`, `valid_uuid`, `dispatch_catalog_list`,
  `dispatch_catalog_inspect`, `check_account_fence`
  companion_guardian.rs: `ManagedClient`, `wait_route`,
  `guarded_sessions_reconnect_and_relist_without_client_reads`

Implementation method:
  - The account fence carries the guardian UUID from the preface identity, the
    shared boot UUID and the connection UUID; it is validated independently of
    the project fence.
  - Cursor format is `boot:revision:session`; any mutation or boot change makes
    the page stale with `ManagedConflict`.
  - `scope_id` filtering is refused until a durable scope identity exists, rather
    than guessed from paths.
  - Rows contain IDs, kind/policy, bounded label, state, sequences, observation
    time and exit status only.

Verification:
  - `cargo test -p warp-agent-bus --lib guarded_sessions_reconnect`: passed,
    exercising the real listener, shared boot identity, reconnect, attach, stop
    and catalog-observed ended state.
  - `cargo test -p remote_protocol`: 17 passed, including account-fence
    validation.
  - `cargo test -p warp-agent-bus --lib`: 249 passed, 1 ignored.
  - `cargo check -p warpai --bin warpai`: finished successfully.

Unverified behavior and remaining risks: no client selects the guarded
  transport, so the RPC is host-reachable only; watch and stop behavior, CLI
  list/resume/stop, takeover receipts, Windows guarded behavior and all real
  SSH/WSL/Windows runs are unimplemented or NOT RUN.

Follow-up tasks / affected gates: client opt-in and transport selection, watch
  ring/gap semantics, stop RPC and CLI; V07/V09/V15/V18/V21 remain NOT RUN.
MEMORY update or reason no durable fact changed: MEMORY records tags 10/11 and
  the host fixture scope.

Record ID / date: S1-o / 2026-11-18
Work item and PRD requirements: optional desktop opt-in and guarded transport
  selection with explicit Windows refusal (FR-01, FR-04, FR-24)
Status: verified_host_only
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: feat/2.0.0 commit after this record; no PR

Problem and observable before/after:
  Before: the guardian existed only behind a private Unix socket; the desktop
  always launched the legacy companion command, no preference existed, and a
  Windows companion answered guardian arguments with the generic stdio error.
  After: an explicit `guardian` preference (default off) switches the remote
  Agent panel to guarded transports; enabling or changing it drops the cached
  remote client so the next connection re-negociates. Guarded constructors
  reject non-POSIX remote shells and require the account fence plus
  `guarded_remote_sessions_v1` and `account_catalog` before returning a client.
  Native Windows prints "unavailable on Windows" for `ctl`,
  `--guardian-service` and `--guardian-stdio` and exits 2.

Scope: app/src/agent_communication/ssh_settings.rs, panel.rs,
  app/src/settings_view/agent_communication.rs,
  crates/agent_bus/src/ssh_remote.rs, ssh_files.rs, companion_main.rs.

Files and symbols changed:
  ssh_settings.rs: `Preferences.guardian`, `set_ssh_guardian`
  agent_communication.rs: `Action::SshGuardian`, `ssh_guardian` row
  panel.rs: `remote_guardian`, guarded connection selection
  ssh_remote.rs: `HostClient::connect_guarded`,
  `connect_session_guarded`, `connect_arguments_guarded`,
  `connect_wsl_guarded`, `require_account_guardian`
  ssh_files.rs: `SshConnection::connect_guarded`
  companion_main.rs: Windows unsupported arms

Implementation method:
  - The preference is a serde-default boolean; legacy JSON without the field
    still deserializes, and the existing generation counter invalidates cached
    clients instead of mutating a live connection.
  - `connect_guarded` appends ` --guardian-stdio <route>` to the installed
    companion command, probes the account fence and capabilities, and closes
    the connection on any mismatch; PowerShell remotes return
    `FeatureUnavailable` before spawning.
  - Windows WSL is routed through the Linux guest companion; a native Windows
    account has no guardian and refuses explicitly.

Verification:
  - `cargo test -p warp-agent-bus --lib`: 261 passed, 1 ignored.
  - `cargo test -p remote_protocol`: 17 passed.
  - `cargo check -p warpai --bin warpai`: finished successfully.
  - The preferences test covers both serialized shapes and the disabled
    default; the settings checkbox and panel switch were not exercised in a
    running desktop.

Unverified behavior and remaining risks: no interactive desktop run, so the
  checkbox, client reset and error surfacing are source-verified only; no
  remote SSH or WSL target produced a guardian connection.

Follow-up tasks / affected gates: V02/V03/V19 remain NOT RUN; native remote
  proof and packaging remain open.
MEMORY update or reason no durable fact changed: MEMORY records the opt-in
  switch and the guarded constructors.

Record ID / date: S1-p / 2026-11-18
Work item and PRD requirements: on-demand scope admission, watch/stop RPC and
  bounded retention (FR-06, FR-07, FR-19, FR-20)
Status: verified_host_only
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: feat/2.0.0 commit after this record; no PR

Problem and observable before/after:
  Before: the guardian admitted a route only when its legacy lock was free at
  startup and exited immediately otherwise; watch (12) and stop (13) returned
  `ManagedFeatureUnavailable`; catalog rows had no writer generation and ended
  rows never expired.
  After: the guardian re-checks unadmitted routes on its one-second tick,
  admits a route as soon as its legacy scope lock is released, retries binding
  without drifting the acknowledgement route, stays alive while a route is
  pending, and prunes ended rows at the 24-hour TTL. Watch pages the bounded
  revision ring with `boot:revision` cursors and an explicit gap; stop resolves
  immediate or exact targets into per-run requested/observed/unknown receipts.
  Rows carry `attachment_generation`, and a successful attach refreshes it so
  takeover presents the observed epoch.

Scope: crates/agent_bus/src/companion_guardian.rs, companion.rs,
  companion_guardian_catalog.rs, companion_terminals.rs, companion_tests.rs.

Files and symbols changed:
  companion_guardian.rs: `admit_scope`, `bind_route`, `spawn_accept`, pending
  route retry, prune tick
  companion.rs: `dispatch_catalog_watch`, `dispatch_account_stop`,
  `catalog_row` generation, attach `touch_attachment`
  companion_guardian_catalog.rs: `Record.attachment_generation`,
  `Catalog::touch_attachment`, `prune_ended`
  companion_tests.rs: `released_scope_is_admitted_under_the_same_guardian`,
  `guardian_watch_and_stop_report_owned_runs`

Implementation method:
  - Admission never steals a live legacy lock: `admit_scope` returns `None`
    while a compatible legacy owner answers, and a stale endpoint is reclaimed
    only after it fails to respond.
  - Watch refuses a cursor from another boot, caps limit at 32, defaults to 16,
    and returns `ManagedConflict` on ring gaps so callers relist.
  - Stop validates project and target IDs before touching the registry and
    maps unproven stops to `UNKNOWN`/`PARTIAL` rather than success.
  - Prune runs on the existing idle tick, so no new timer or thread is added.

Verification:
  - `cargo test -p warp-agent-bus --lib`: 261 passed, 1 ignored, including the
    real-listener admission and watch/stop fixtures and the catalog prune and
    `PRAGMA query_only` fault tests.
  - `cargo test -p remote_protocol`: 17 passed.

Unverified behavior and remaining risks: no two real helpers (SSH and WSL) have
  cut over the same account; no real SSH/WSL/Windows daemon has served watch or
  stop; TTL and capacity were not measured beyond host fixtures.

Follow-up tasks / affected gates: V07/V11/V12/V15/V18 remain NOT RUN.
MEMORY update or reason no durable fact changed: MEMORY records on-demand route
  admission, watch/stop semantics and the 24-hour prune tick.

Record ID / date: S1-q / 2026-11-18
Work item and PRD requirements: minimal private CLI with bounded parser,
  MRU/unique selection and explicit detach (FR-23, CLI.md)
Status: verified_host_only
Baseline commit: db280c6b57c3eab7acb8c8e04ab8c8aab084828a
Implementation commit / PR: feat/2.0.0 commit after this record; no PR

Problem and observable before/after:
  Before: retained runs were reachable only through the protocol; no command
  could list, resume or stop them.
  After: `warpai-companion ctl list|resume|stop` uses the installed executable,
  probes the guardian without ever starting one, opens the project root
  (cwd or `--project`), and speaks the account-fenced catalog, attach and stop
  RPCs over the private Unix socket. `list` reports guardian readiness and
  non-secret rows account-wide or per project. `resume` selects exact ID,
  unique name or the highest-MRU live session, attaches with the observed
  generation only under `--takeover`, replays retained output, forwards
  input/resize, and detaches on `Ctrl+B Q` (doubled prefix forwards one literal
  `0x02`) or stdin EOF without stopping the PTY. `stop` is account-wide when
  bare, resolves one exact run by ID or unique name, and maps receipts to exit
  0/1. Syntax errors and ambiguous names exit 2.

Scope: crates/agent_bus/src/companion_ctl.rs (new),
  crates/agent_bus/src/companion_main.rs, lib.rs, ssh_remote.rs connect_unix.

Files and symbols changed:
  companion_ctl.rs: `parse`, `Command`, `Selection`, `connect`, `select`,
  `resume`, `attach_inner`, `escape_filter`, `stop`, `RawTerminal`
  companion_main.rs: `ctl` dispatch and Windows refusal
  ssh_remote.rs: `HostClient::connect_unix`, `HostWriter`/`HostReader`

Implementation method:
  - Parsing is strict: only the three commands plus help/version, `--` ends
    options, duplicate or conflicting flags fail, IDs are UUID-validated and
    names are bounded without control characters; no shell text is evaluated.
  - The catalog stores no paths, so a bare `resume` resolves within the opened
    project and reports an actionable error when the account-wide MRU row
    belongs to another project; `--project` opens that root explicitly.
  - The attachment keeps the server tuple for every read/input/resize/detach;
    detach and TTY restoration happen on every exit path.
  - The virtual `warpai` shell function is intentionally not installed: the
    absolute `.../warpai-companion ctl ...` invocation is the supported form,
    matching the documented CLI.md limitation.

Verification:
  - Six `companion_ctl` unit tests cover the parser, `--` handling, named/ID
    conflicts, selection precedence, ended exclusion, row sanitization and the
    split-read/doubled-prefix detach escape.
  - `cargo test -p warp-agent-bus --lib`: 261 passed, 1 ignored.
  - `cargo check -p warpai --bin warpai`: finished successfully.

Unverified behavior and remaining risks: the CLI has never attached to a live
  guardian in a real TTY, so raw-mode restoration, replay and escape behavior
  are unit-tested only; no SSH/WSL/Windows run and no `warpai` shell function.

Follow-up tasks / affected gates: V21/V22/V23 remain NOT RUN; interactive CLI
  acceptance is pending.
MEMORY update or reason no durable fact changed: MEMORY records the `ctl`
  surface, the no-path catalog limitation and the absent shell function.

Record ID / date: S1-r / 2026-11-18
Work item: implementation ledger audit and repair of guardian admission, query,
lifecycle and stop defects.
Status: implemented on working tree against `61b3fcac`; see verification below.
Implementation commit / PR: working-tree changes; no commit or PR.

Implementation and method:
- CLI uses the wired 32-row limit and walks cursors to completion. Account-wide
  listing and stop initialize without a project. Bare resume selects within the
  admitted project; exact opens still require a project root.
- WSL preserves validated distro/user arguments and passes the bridge flag and
  route as separate argv entries. The installed build omits the guarded
  capability while the native gate is open.
- The private preface distinguishes direct operator, guarded data bridge, and
  connection-owned forwarding. The latter checks the logical service ID before
  forwarding Initialize and cannot control retained sessions. Scope ownership
  remains fenced by the existing Store lock.
- List/watch cursors bind filter values. Watch filters before paging and retains
  sanitized event rows, including removals. It tracks evicted revisions
  separately from gaps in the sequence allocator.
- A durable unknown reservation precedes guarded native launch. A proven spawn
  failure removes it. Successful launch commits live state; uncertainty after
  a native side effect remains explicit. Ended runs are protected from release
  and eviction until durable observation. Attach updates the exact run and
  monotonic writer generation in one catalog statement. Reads no longer write
  exit state.
- The stop batch captures owned run handles before dispatch, does not hold the
  registry lock during native stop, and replays receipts by request ID and
  intent. Requested-only outcomes are partial. The boot-scoped store refuses
  new IDs after 1,024 admitted batches to preserve exact replay.

Verification on macOS arm64 against this working tree:
- `cargo test --locked --offline -p warp-agent-bus --lib`: 263 passed, one
  ignored. The listener exercises 40-row CLI pagination, ordinary and guarded
  admission, invalid targets, same-ID replay, filtered cursors and natural exit.
  WSL argument construction is tested without executing wsl.exe.
- `cargo test --locked --offline -p remote_protocol`: 17 passed.
- `cargo test --locked --offline -p warp-agent-bus --test managed_agent`: six
  passed, three child fixtures ignored.
- `cargo check --locked --offline -p warpai --bin warpai` passed in default and
  `warp_platform` configurations. `warpai-companion` binary check passed;
  the built binary returned help and version without opening a guardian.
- `git diff --check` passed. The available Homebrew compiler is Rust 1.99.0;
  the repository-pinned Rust 1.92.0 was unavailable on this host.
- Native case IDs: none. Measurements, platform packages, publishing and native
  acceptance are outside the available host evidence.

Remaining gates: installed SSH-first/WSL-first owner election and migration-free
mixed-helper coexistence, desktop guarded launch and reconnect, snapshot/gap
recovery, session shell bootstrap, native Windows ownership, interactive CLI,
resource/fairness benchmarks, cross-platform V01-V30 and packaging. See the
closure checks above. MEMORY.md records the updated capability gate and ownership
semantics.

Record ID / date: S1-s / 2026-11-18
Scope: catalog projection recovery and strict interface localization.
Baseline: `4603a46a` on `feat/2.0.0`.

An established writer returns its native tuple when SQLite is unwritable. The
registry holds one unpublished generation per retained run, and the existing
maintenance pass retries its exact account/boot/service/run projection. Durable
recovery cannot roll back the published generation or revive an ended row. A
launch still requires durable identity reservation before native spawn. The
SSH guardian control now has simplified and traditional Chinese translations.

Verification on macOS arm64: SQLite read-only fault fixture, 264 library tests
passed with one ignored, app binary check and strict interface coverage
(3,005 pairs; zero unclassified sites). Cloud communication run 44277308190
failed at its translation gate for `4603a46a`. Native V-case acceptance remains
open.

Record ID / date: S1-t / 2026-11-18
Scope: native guarded SSH/WSL continuity fixtures and cross-platform CI fixes.
Acceptance planned: run shared fixture logic through disposable OpenSSH and
real Windows-to-WSL bridges with separate scoped Stores. The WSL test runs
serially and leaves every fixture bridge closed for 12 seconds before rejoin. Retain one native command through bridge
loss; compare boot/service/project/session/run identities, recover output from
the same run, fence the old writer, and stop the exact attached run. Verify the
installed helper withholds the guarded capability during this test.

The earlier remote companion run 44277309897 found a stale Windows fixture
call signature and a stale Linux SSH input replay assertion. The Windows test
now passes the native quoted helper command. The Linux test expects identical
input to reconcile and different bytes under the same sequence to conflict.
Linux OpenSSH receipt: run 44279117156, job 134715011124, source
`91f7a2aab8a8ad762b59e71fd696a3aee26934c9`. The guarded fixture and four
other native SSH tests passed. This proves the guarded bridge and native PTY
continuity slice of V02/V04; full desktop and release acceptance stays open.
Windows-to-WSL evidence is pending.

Record ID / date: S1-u / 2026-11-18
Scope: current-session CLI resume without recursively attaching its own PTY.
Baseline: `91f7a2aa` on `feat/2.0.0`.

Direct same-UID operator admission resolves the socket peer's native Unix
session leader and compares it with the retained guardian PTY leader.
`ManagedInitialized.current_guarded_session` (tag 8) reports only exact
session/run IDs. The ordinary and remote data bridges receive no selector.
A bare resume in that process session returns success without project opening
or acquiring a writer. Explicit discovery and attachment continue to use
the existing project and writer fences.

Host proof: a real Companion-owned PTY reports its native session leader.
The fixture verifies a positive exact run match and a negative match from the
test process. The Apple private socket PID lookup compiles on macOS; its
native CLI interactive acceptance and V29 are still pending.

Record ID / date: S1-v / 2026-11-18
Scope: exact guardian bridge identity and account-specific launch preferences.
Baseline: `eaedc067` on `feat/2.0.0`.

The installed stdio bridge verifies the logical service ID and the socket's
guardian/boot IDs against Initialize before releasing the managed handshake.
Direct operator CLI applies the same guardian/boot check before project open.
Guarded WSL resolves the distro from the guest's native root translation and
rejects a conflicting environment hint. The legacy WSL lookup is unchanged.

SSH preference is keyed to the selected account's verified service ID. WSL
prefers its selected installed distribution/user pair. A failed preference
write rolls back the staged toggle. The old global development toggle is
accepted on load without applying it to any account. Ordinary collaboration
uses the connection-owned transport independently of launch preference.

Local proof: scoped-handshake rejection, WSL UNC root parsing, preference
isolation fixtures, strict interface localization and macOS app compilation.
The actual WSL root translation, Windows settings and native appearance still
require their platform runs.

Record ID / date: S3-a / 2026-11-18
Scope: guarded viewport sequencing, CLI TTY handling, and launch contention.
Baseline: `0bba6640` on `feat/2.0.0`.

Guarded resize is individually sequenced per native writer. Identical retries
echo the accepted sequence without reapplying size, a conflicting replay fails,
and a replaced writer cannot resize. The CLI completes attachment before
sending its first viewport resize. It checks local input between retained
output pages, restores native terminal attributes on exit, and detaches once
using the last exact tuple. It forwards literal input preceding a detach
escape in the same read. Native launch reserves a bounded pending ID, then
releases the shared registry before spawning; other sessions can progress
while that launch is blocked.

Host proof on macOS arm64: 268 agent bus tests passed (one ignored) and
17 protocol tests passed. Positive, replay, conflict and stale resize on a real local PTY;
native termios restoration; concurrent sibling launch under an injected
blocked launch with duplicate-ID rejection. The Linux OpenSSH and WSL
fixtures also exercise resize when built from this revision. Full interactive
TTY, snapshot, shared wake scheduling and native release gates remain open.

Record ID / date: S2-a / 2026-11-18
Scope: ordered native parser/retention cut and POSIX attachment cleanup.
Baseline: `36bdc6d3` on `feat/2.0.0`.

The existing native Agent screen and bounded retained bytes now share the
same output lock. A read cannot observe a byte cut ahead of the parser. Resize
takes the lock across native and parser dimensions. An incomplete Unicode
sequence stays in the parser while the exact retained byte count advances;
this makes the need for saved decoder state explicit. The CLI restores the
local raw TTY on handled SIGINT, SIGTERM, SIGHUP and SIGQUIT. If a signal
interrupts a managed frame, it closes the transport and lets the guardian
detach the writer instead of writing another potentially misframed request.

Host proof: the real local cut fixture handles split Unicode and exact
retained bytes. The CLI tests include actual PTY terminal attributes and
escape parsing. Snapshot format/import, block state and interactive native
signal acceptance remain open.

Record ID / date: S5-a / 2026-11-18
Scope: native scope admission under sustained connection traffic.
Baseline: `f0ce2c9f` on `feat/2.0.0`.

The maintenance clock is independent of incoming connections. Released scope
locks are observed under the same guardian even while SSH bridges continue
to connect. A fixture holds the WSL scope, generates repeated SSH handshakes,
releases WSL and verifies the same guardian and boot IDs on both listeners.

Local proof: contention and continuous-accept fixture passed on macOS arm64.
Installed-helper first-owner election and full native WSL remain separate
acceptance cases.

Record ID / date: S5-b / 2026-11-18
Scope: same-account SSH-first/WSL-first election and legacy scope routing.
Baseline: `fd86a43e` on `feat/2.0.0`.

On a verified WSL guest with both helpers installed, the SSH launcher verifies
the owned WSL image against its installed manifest and probes the private
dual-route ABI before invoking it. WSL-first uses that image directly.
The guardian's route-local Projects adapters keep separate setup and Store
ownership. Ordinary SSH keeps its native Agent binding, including the native
working-directory check. Ordinary stdio preflights the guardian's scoped
service ID before redirecting; an unrelated legacy Store starts independently.

Host proof: the manifest fixture rejects modified and symlinked binaries, the
default and feature libraries check on macOS, local Companion tests pass,
and the WSL provisioning script passes `bash -n`. The WSL CI guest now
installs both source-matched helpers. Its native Windows bridge fixture checks
both first-owner orders and distinct logical IDs; the result remains pending.

Record ID / date: S5-c / S7-a / 2026-11-18
Work item: ordinary bridge regression and account-control admission under load.
Baseline: `92f31965`. Status: verified on macOS arm64.

The ordinary stdio preflight now has a real private-socket fixture. It rejects
another Store and returns a fresh connection with the matching Store's service
ID and no account fence. The Unix listener has 32 established bridge permits,
two independent local operator permits and 32 bounded pending prefaces. A peer
must pass the existing kernel UID and preface checks before taking a channel
permit, and receives the boot reply only after admission. Each permit is released
on failure or disconnect. Idle exit counts all three pools. No wire fields or
Store schema changed.

The contention fixture occupies every bridge permit, verifies a further bridge
is refused, then initializes a local operator and sends catalog and stop RPCs.
The full `warp-agent-bus` macOS arm64 suite passed with 272 tests and one ignored.
This does not establish bulk IO fairness or native resource targets. MEMORY.md
records the control-admission limits.

Record ID / date: S7-b / 2026-11-18
Work item: lifecycle scans and run admission under a busy sibling.
Baseline: `5bf0f5a6`. Status: verified on macOS arm64.

Maintenance and admission keep busy owners and retry observation as capacity
becomes available. Eviction under the registry lock requires the run to have
no in-flight owner reference; output locks are probed without waiting. Exit
scans release the registry before inspecting native runs and record their exact
identity for catalog reconciliation. A retained run still counts against the
admission ceiling. No timeout forces a stop or drops task facts.

The native local fixture holds an existing run's session lock, then verifies
maintenance and a sibling PTY launch both complete before the lock is released.
The macOS arm64 suite passed with 273 tests and one ignored. A failed WSL
acceptance run now skips the Windows cache upload so its log can finalize.
No protocol or Store changes. The earlier MEMORY.md resource notes still apply.

Record ID / date: S5-d / S7-c / 2026-11-18
Baseline: `cdf15e0d`. Scope: native election failure diagnostics and output ring.

The `92f31965` WSL job recovered both ordinary-workspace cases and hit EOF in
the guarded fixture. The next fixture prints the startup-order stage. Bridge
stderr distinguishes static startup and identity failure classes without
formatting the underlying native paths or protocol payloads.

The output ring discards bytes before extension so a full 256 KiB ring no
longer doubles its allocation on the next read. The native parser receives
the complete input and absolute output offsets still advance by its full length.
The local fixture checks retained bytes, capacity and offsets for both ordinary
and oversized appends. No wire or Store changes. Native election acceptance and
total RSS targets remain open. The MEMORY.md resource notes still apply.

Record ID / date: S4-b / 2026-11-18
Baseline: `398dbc6f`. Scope: FR-16, FR-18, V25 and V29 CLI resume admission.

Bare or named resume can select a live run in another project of the helper's
logical scope. Selection comes from the scoped catalog, independent of the CLI
working directory. The additive `GuardedProjectOpen` operation uses exact live
run and project identity from memory, verifies local operator admission and
reopens the native project path. A replacement directory, stale run or wrong
scope is rejected. The operation acquires no writer and skips MRU changes. The
existing attach path still enforces writer conflict and explicit takeover.
`--project` retains existing explicit-path admission and works with older
guardians. The catalog still persists no paths.

Host proof: a real Unix listener and PTY fixture verifies matching operator
admission, rejects data bridges, another logical scope, replaced directories
and stale run IDs, and proves opening the project leaves the catalog row intact.
The macOS arm64 library suite passed with 274 tests and one ignored, and all
17 protocol tests passed. Native CLI TTY and desktop acceptance remain open. MEMORY.md records the
boot-local project lookup and scope boundary.

Record ID / date: S7-d / 2026-11-18
Baseline: `9148fdb9`. Scope: isolate long-lived local attach from account control.

CLI resume uses the new local-session preface mode and regular channel capacity.
Catalog and stop retain dedicated local control permits. A full pool returns a
validated busy preface; current clients do not interpret occupied capacity as a
missing daemon. An older peer that closes the unknown local-session preface can
still be reached through the original operator mode. Store, session and task
authorization are unchanged. The real listener fixture fills all regular
channels, including local sessions, verifies explicit busy on both admission
modes, then runs an independent catalog and stop request. All 18 guardian
fixtures pass on macOS arm64. MEMORY.md records the allocation boundary.
The WSL workflow now caps the setup step at ten minutes. The two completed
source-matched provisions took under 90 seconds each; run `44304694837` spent
over eight minutes there before any repository WSL test could execute.
Network-only, Windows-only and WSL acceptance inputs now participate in the
workflow concurrency key. An unrelated validation mode no longer cancels or
queues behind a run with an otherwise identical group.

Record ID / date: S4-c / 2026-11-18
Scope: MRU selection when a candidate ends during discovery and admission.

Bare resume drops a candidate only after its exact run is confirmed absent or
ended, then chooses from the remaining bounded results. A missing or replaced
project root returns a stale attachment rather than a missing run. Exact
selection and writer conflict still report failure for the selected target.
Where no candidate has an MRU sequence, the newest creation sequence wins.
The host fixtures check creation ordering and distinct run/root error codes.
No wire, Store or durable memory changes.

Record ID / date: S5-e / 2026-11-18
Scope: installed WSL helper directory permissions during SSH-first election.

[Two](https://github.com/OthinusG/warpai/actions/runs/44307268038)
[native WSL runs](https://github.com/OthinusG/warpai/actions/runs/44308187073)
failed the first SSH bridge at the permission boundary; both ordinary WSL
workspace cases passed. The packaged installer reproduced the mismatch locally
under umask `002`: it created group-writable WSL directories that the image
verifier refuses. The installer now creates private directories and repairs
group/other write bits on component-owned directories on upgrade. It preserves
the parent configuration directory and rejects an unsafe parent before repair.

The generated installer fixture now exercises permissive-umask first install,
upgrade from writable component directories and prior isolation/corruption cases.
The permission check failed against the old installer and passed against the
fix. The WSL CI guest installs with the same permissive umask. The source-matched
Windows/WSL job at `acd9e63e` passed the guest build, installation and native
workspace checks, including both startup orders, distinct logical identities,
exact-run reconnect and both ordinary workspace cases. MEMORY.md records the
installed boundary and native receipt.
The WSL-focused Windows job skips the native app cache upload when it does not
build or capture the desktop. Its test result can finalize immediately after
the workspace checks. Full desktop builds retain the cache path.

Record ID / date: S7-e / 2026-11-18
Scope: FR-05/FR-11/FR-12, guarded native wake scheduling.

Guarded Agent wake checks use the existing Tokio timer and blocking pools.
Broker watch events coalesce Store changes. Per-run notifications handle input
and committed PTY output without waking siblings for redraws. The shared
readiness predicate provides the next quiet/ready deadline.
Successive evaluations are at least 300 ms apart, including draft cancellation,
paste and Enter. Idle reconciliation uses a five-second deadline.
Legacy wake checks retain their current loop. The worker test observes an idle
window with zero extra evaluations, wakes on notification, and verifies pacing
under a burst. A notification sent while an evaluation is blocked triggers the
next paced evaluation. Native delivery and resource gates remain open.

Record ID / date: S7-f / 2026-11-18
Scope: direct guarded deliveries and project-wide pool arbitration.

A guarded worker resolves the direct wake for its terminal and live run from
the existing Broker registry. Project-wide evaluation is needed only for a
broadcast pool candidate and retains the same winner ranking and claim guards.
The legacy selector and the indexed path share the readiness and message query.
The existing multi-Agent test compares the exact candidate from both paths
before verifying single-claim and draft/cancellation fencing. No wire or Store
changes. MEMORY.md records the direct lookup boundary.

Record ID / date: S8-a / 2026-11-18
Scope: V02/V04 controlled macOS OpenSSH acceptance.

The macOS remote validation job now runs the same explicit guarded bridge
loss/reconnect case against the existing owned, host-key-pinned loopback
fixture. It runs before legacy project tests so a connection-owned service
cannot hold the fixture scope first. The launcher option is restricted to
disposable Unix CI accounts because macOS uses account application support
paths. The Rust fixture compiles and the Python supervisor checks pass on
macOS. Native cloud results are tracked separately.

Record ID / date: S8-b / 2026-11-18
Scope: V25/V26, direct CLI resume and detach on real Unix SSH TTYs.

The controlled reconnect fixture holds every bridge closed for twelve seconds,
then runs the installed-style Companion `ctl resume` command over `ssh -tt`.
It checks the original PID, submits input and reads the run's echo, sends the
detach prefix across two writes, and confirms the enclosing SSH terminal's
`stty -g` value matches its value before resume. A new protocol writer then
reattaches to the same run. The WSL fixture keeps its existing headless
bridge-loss test. The SSH fixture compiles locally; cloud execution is pending.
The fixture also exposed a production admission mismatch: the direct CLI
required a guarded capability that the installed guardian withheld from all
channels. The local authenticated operator now receives it; the remote data
bridge continues to withhold it. The TTY fixture uses the normal installed
binary path and also checks the data bridge's restricted capability set.
Linux OpenSSH passed the native PID, input, split detach and `stty` checks at
`542bf22f` in [run 44322343719](https://github.com/OthinusG/warpai/actions/runs/44322343719).
On macOS, resume and input succeeded but the final restoration marker was
missing. A full local Darwin PTY attribute comparison reproduced the extra
`PENDIN` bit after canonical mode returned. The CLI restores attributes and
flushes pending input without waiting for the output queue. The full attribute
comparison passes after the fix. On `3d33a7d6`, [run 44328848156](https://github.com/OthinusG/warpai/actions/runs/44328848156)
passed all three platforms. The macOS OpenSSH job passed the guarded reconnect
with the outer TTY restoration assertion, then existing direct and jump SFTP
fixtures. Linux passed its corresponding owned OpenSSH matrix on the same source.

Record ID / date: S4-d / 2026-11-18
Scope: Bash/Zsh session-only `warpai` command (FR-16, V26).

The existing Bash/Zsh bootstrap includes a shared fragment after shell startup.
It uses the managed SSH marker or the native WSL UNC root and exact distro hint
to select the dedicated absolute helper path. Installation verification checks
ownership, symlinks, directory/file write permissions and the binary's installed
SHA-256 before dispatch. The shell function forwards literal arguments to `ctl`.
Aliases, functions and executables named `warpai` keep their precedence, with an
absolute-path hint. No profile, persistent environment or PATH changes.

The owned TTY fixture passed in Bash and Zsh on macOS, covering literal Unicode
and metacharacters, three collision types, noninteractive/normal local shells
and corrupt helper rejection. Remote CI includes the fixture, with native WSL
guest path checks when run inside WSL. Native Windows requires its guardian.

Record ID / date: S8-c / 2026-11-18
Scope: V11/V25, caller-loss semantics for account stop.

The controlled Unix case starts two guarded PTYs and invokes bare `ctl stop`
inside the first process in registry order. The external fixture checks the
caller and sibling both reach observed native exit with drained output. This
uses the ordinary installed-style operator path and establishes the caller
receives the first stop request. The fixture never depends on a response from
the caller. Linux and macOS passed this case on `1b3f5e9a` in
[run 44332532490](https://github.com/OthinusG/warpai/actions/runs/44332532490).

Record ID / date: S4-e / 2026-11-18
Scope: non-mutating writer observation and independent WSL name verification.

Account list and inspect return the optional attached/free observation only
for an exact native session, run, boot, service and attachment generation.
The observer holds no catalog lock while sampling and returns unknown when a
registry/session lock is busy. CLI formats this state separately from process
status. No schema migration, event or MRU mutation. The native listener test
observes a writer while attached and free after transport loss.

The WSL shell fixture found that the path translator accepts an environment
hint supplied by its caller. Both native probes strip `WSL_DISTRO_NAME` before
the translation, then compare against the original hint. This isolates the
probe from the text it is supposed to validate. [Run 44333896370](https://github.com/OthinusG/warpai/actions/runs/44333896370)
on `904801ed` passed the positive and mismatched-hint shell cases, the native
guarded reconnect, both WSL/SSH election orders and ordinary WSL workspace tests.

Record ID / date: S8-d / 2026-11-18
Scope: direct Agent delivery after native SSH bridge loss (V05, V14).

The owned loopback fixture uses two registered guarded Agent PTYs in one project.
It kills the receiver SSH process, waits for the writer to be observed free,
then sends from the still-attached peer. The receiver must write a message/run
receipt before any replacement receiver bridge is opened. The replacement
writer verifies the exact run, retained native notification output and the
still-unacknowledged message. The fixture compiles locally. Native execution
and the full task, busy/permission and WSL matrices remain pending.

## Evidence index

| Receipt | Source | Result | Coverage |
| --- | --- | --- | --- |
| S1/S3 source slice | `87e18c94`, `5163030f` and `99eaa0eb` on `feat/2.0.0` | implemented_unverified | Guarded retention, exact writer gate, CAS takeover, input epoch, bounded receipts, fail-closed writer handling, Unix peer UID admission and 300s ended-output retention written; rustfmt parse and isolated helper type-check only, not compiled or run |
| S1-d retention/admission | `99eaa0eb` on `feat/2.0.0` | implemented_unverified | Unix peer UID check, ended-output TTL and capacity eviction, exact tuple echo; no workspace build, native test or resource measurement |
| S1-e review closure | `efe9fd12` on `feat/2.0.0` | implemented_unverified | Generation-keyed ledger, non-evictable Unknown, between-write cancellation, guarded-only retention, fail-closed Unix credentials, Windows client SID recheck, explicit DeliveryUnknown and no connection-wide close for session stale; rustfmt parse and diff check only, no build/test |
| S1-f catalog prototype | Source commit after this record on `feat/2.0.0` | implemented_unverified | Bounded stable pages, boot/filter-bound cursors, revision-scoped watch ring with explicit gap, MRU resume excluding ended runs, state-age tracking, 32/64 ceilings; 7 isolated `rustc --test` cases passed with a uuid stub; not linked into the workspace crate and not wired to any session |
| S1-g build restoration | `e220a5c9` plus the follow-up fix commit on `feat/2.0.0` | verified_host_only | `remote_protocol` E0599 fixed, `protoc` 36.2 installed, stale write-count and idempotent-input test expectations corrected; `cargo check -p warp-agent-bus`, `cargo check -p warpai --bin warpai` and the full `warp-agent-bus` + `remote_protocol` test suites pass on macOS arm64; remote SSH/WSL and ignored fixture tests remain NOT RUN |
| S1-h session kind | Source commit after this record on `feat/2.0.0` | verified_host_only | Additive `session_kind=11`, pre-spawn policy/kind matrix, replay conflict on kind change, client capability gate; full host suites pass; capability unadvertised, no remote launch tested |
| S1-i account election/routes | Source commit after this record on `feat/2.0.0` | verified_host_only | Passwd-home root, account/scope locks, private route sockets, `WAG1` preface with peer-UID check, stale-only reclaim, shared boot identity, route-derived capabilities and guest/chunk denial; 8 focused lib tests and full 241-test host suite pass; no production client and no real SSH/WSL run |
| S1-j durable catalog | Source commit after this record on `feat/2.0.0` | verified_host_only | `catalog.sqlite3` identity `0x57474341`/v1, refuse-without-overwrite on mismatch, boot live→unknown marking, bounded ended eviction, unresolved capacity refusal, explicit MRU update; 3 focused tests plus full host suite; not yet wired to session transitions |
| S1-k catalog lifecycle | Source commit after this record on `feat/2.0.0` | verified_host_only | Guarded launch inserts/updates before ack, attach updates MRU, observed exit marks ended with status, connection-owned launches write nothing, ended rows are not resurrected; lifecycle and upsert tests pass in the 243-test host suite; no natural-exit watch without a read, no retention catalog sync, no list/CLI |
| S1-l client admission/bridge | Source commit after this record on `feat/2.0.0` | verified_host_only | Private directory/socket and kernel peer-UID checks, absent-only single spawn, incompatible-owner fail-closed, preface off the dispatch loop, zero-route refusal, internal `--guardian-stdio` helper; 14 focused and 249 total host tests pass; no production caller and no GUI opt-in |
| S1-m lifecycle authority | Source commit after this record on `feat/2.0.0` | verified_host_only | Account-serial signed sequences, strict open invariants, transactional creation, full-identity updates, ended non-resurrection, terminals-owned exit observation on the existing tick; 14 focused and 249 total host tests pass; no TTL/prune evidence or DB-fault test |
| S1-n account catalog RPC | Source commit after this record on `feat/2.0.0` | verified_host_only | Additive tags 10-13 and `account_fence=7`; account-fenced list/inspect with boot/revision cursor, project filter, 32-row cap; real-listener fixture covers preface, guarded launch, reconnect, attach, stop and read-free ended observation; watch/stop return feature-unavailable; 249 host tests and 17 protocol tests pass; no remote run |
| S1-o opt-in/transport | Source commit after this record on `feat/2.0.0` | verified_host_only | Persisted `guardian` preference with settings checkbox, guarded panel transport, capability/fence-checked guarded constructors, Windows explicit refusal; 261 host and 17 protocol tests plus `warpai` binary check; no desktop or remote run |
| S1-p admission/watch/stop | Source commit after this record on `feat/2.0.0` | verified_host_only | Released-lock route admission, pending retry, 24h prune tick, watch gap cursors, per-run stop receipts, `attachment_generation`; real-listener and catalog fault tests pass; no two-helper or remote run |
| S1-q private CLI | Source commit after this record on `feat/2.0.0` | verified_host_only | `ctl list/resume/stop`, strict parser, MRU/unique/ID selection, `--takeover`, `Ctrl+B Q` detach, exit codes 0/1/2, no daemon start; six unit tests; no live-TTY run |
| S0 caller/contract freeze | Uncommitted docs/source on `db280c6b` + `evidence/S0-baseline.md` | implemented_unverified | Ownership map, frozen fields/actions, capability withheld; no build |
| D5 archive integrity | Uncommitted prompt/archive references on current baseline | passed | Links/paths, Steps 0-17, final scope/defaults, S0-S9/V01-V30 and honest pending runtime state |
| D4 document integrity | Uncommitted R4 framework | passed | Minimal CLI decision matrix, withdrawn API/task removal, 19 requirements/30 cases and integrity |
| D3 document integrity | Uncommitted R3 framework | passed | Defaults, command methods, 19 requirements/30 cases and integrity |
| D2 document integrity | Uncommitted R2 framework | passed | Unified topology, private CLI, 17 requirements/28 cases and document integrity |
| D1 document integrity | Uncommitted R1 changes on baseline above | passed | Links/paths, requirement/task/case mapping, scope consistency, no runtime claims |
| Herdr source reference | ee5bfb5041bb3e4a49d591c72308aae3c81ecdde | inspected | Documented ownership, surface-interest and fairness patterns; no runtime/benchmark proof |

## Current proposed decisions and freeze gates

| Decision | Proposal | Freeze gate |
| --- | --- | --- |
| Scope/default | SSH/WSL guardian, opt-in, native local unchanged | Owner correction, then S0 contract |
| Unit of retention | Explicit managed Agent or shell/command PTY, not arbitrary PID | S0/S1 |
| Service topology | One guardian per execution OS account, separate logical SSH/WSL/Store scopes | S0/S1 |
| WSL lifecycle | Guest service owns guarded PTY, native foreground path preserved | S5 |
| Multi-task execution | Existing serial Agent queues, parallel independent sessions | S7 |
| Low overhead | Shared notifications/deadlines, on-demand snapshots, bounded memory | S2/S7/S8 |
| Restart recovery | Unknown/interruption only, no relaunch or authority inheritance | S6 |
| Bare CLI defaults | Current/unique/MRU resume; immediate account-wide guardian stop | Owner request, S0/S4/S6 |
| Public CLI scope | list/resume/stop + help/version only; extra command requires proven unmet need | Owner correction, S0/S4 |
| Companion version | Proposed 6.2.0 if compatibility remains additive | S0/S9 |

## Change discipline

Update the owning specification before changing its implementation contract, then
callers/fixtures, ledger, tests and release claim. Keep failed approaches only when
they explain a final tradeoff. Every scope correction supersedes conflicting old
text; do not leave withdrawn local-runtime/optional-WSL tasks in an active checklist.
No task checkbox closes without actual code and appropriate source-matched proof.
