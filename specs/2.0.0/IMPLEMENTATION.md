# Warpai 2.0.0 actual implementation ledger

Updated: 2026-11-18. Scope revision: R4 optional SSH/WSL guardian.

## Current truth

Design documents plus one unverified runtime slice exist. The slice adds additive
guarded launch policy, retained native lifetime on connection loss, and
attach/detach/takeover writer fencing in the existing SSH/WSL Companion terminal
registry. Guarded input/resize/read/detach/release now require the exact writer
tuple, takeover fences the observed generation, and detach/transport loss
invalidates queued input through a separate writer epoch. A bounded input ledger
now returns per-attachment receipts for accepted, writing, written, cancelled,
failed and unknown delivery, keys records by attachment generation and sequence,
rejects conflicting or skipped sequences, retains unknown delivery as
non-evictable evidence, and fails the writer path closed on write, flush, gate or
ledger faults. Cancellation is checked between writes, and queued input skipped
after revocation or a writer fault is settled instead of remaining pending. The
service now requires Unix peer credentials to resolve and match its effective UID,
re-checks the Windows pipe client SID after DACL admission, and retains detached
ended output for 300 seconds within the 32-session ceiling, evicting the oldest
ended session only under admission pressure. Client control also requires the
guarded attachment tuple to be echoed exactly for every non-attach action; a
session-level stale reply no longer closes other sessions on the multiplexed
connection, and unconfirmed input delivery is reported explicitly. A pure
bounded catalog index prototype (stable descending pages, revision-scoped watch
ring with explicit gap, MRU resume selection, bounded entries/events) now exists
with passing isolated tests, but it is not wired to the terminal registry and
grants no client access. The `warp-agent-bus` and `remote_protocol` crates now
compile and their full test suites pass on macOS arm64, including the real
Companion PTY integration tests; this is host evidence, not remote SSH/WSL
evidence. No unified SSH/WSL election, runtime catalog/list/inspect/watch surface,
snapshot, CLI, desktop opt-in, scheduler, packaging/version change or release is
implemented. V01-V30 runtime
acceptance is NOT RUN. No guard client can reach the slice because
`guarded_remote_sessions_v1` is deliberately not advertised. Runtime inspection
baseline is `db280c6b57c3eab7acb8c8e04ab8c8aab084828a`; the source slice is
commits `87e18c942630e58c91849532fb132323e0500ab5`,
`5163030f7535f59f673723c1932563bd186282c3`, `99eaa0eb`, `33f59ec1` and
`efe9fd12` on `feat/2.0.0`, while `main` remains at the baseline. A read-only peer
review at the first commit identified remaining P1/P2 work; the review findings
were addressed in `efe9fd12`, but this remains source evidence, not runtime
evidence. Revalidate current HEAD at implementation startup rather than assuming
an earlier observed repository position is still current.

The owner corrected the initial plan: guarding is optional/default-off, native
local persistence is excluded, SSH and WSL are both core, and resource overhead
and multi-task/session continuity are primary requirements. This correction
supersedes the original mandatory-local/optional-WSL design; do not implement it.

## Work status

| Work | Status | Actual method/evidence |
| --- | --- | --- |
| D0 original research/framework | superseded_scope | Source inspection and initial documents; native local proposal withdrawn |
| D1 R1 scope/framework revision | verified_documents_only | Optional remote scope correction; superseded topology details refined in D2 |
| D2 unified account daemon/private CLI | verified_documents_only | Initial defaults superseded by owner-specified R3; no runtime work |
| D3 MRU/all-stop defaults and command extension | superseded_command_scope | MRU/all-stop retained; extra commands withdrawn by D4 |
| D4 minimum justified CLI | verified_documents_only | Core commands/decision matrix, narrowed APIs/tasks and document integrity; runtime pending |
| D5 implementation prompt archive | verified_documents_only | Step-by-step handoff, current scope, evidence requirements and archive integrity; runtime pending |
| S0 contract/caller inventory | in_progress | Caller/ownership inventory and guarded wire fields frozen ([evidence/S0-baseline.md](evidence/S0-baseline.md)); account catalog/watch ring prototype is tested in isolation but not on the wire; storage schema pending |
| S1 account guardian/native ownership | in_progress | Guarded policy retains SSH/WSL Companion PTY/threads across connection loss; writer gate, generation-keyed receipt ledger, fail-closed Unix peer identity and Windows client SID recheck are written; 300s ended-output retention is unverified; unified SSH/WSL election, registry-backed catalog and native proof pending. `cargo check -p warp-agent-bus` and its full test suite pass on macOS arm64 |
| S2 compact terminal restoration | not_started | None |
| S3 attachments and ordered IO | in_progress | Exact attachment tuple, CAS takeover, bounded input ledger with idempotent sequence and counted write receipts, non-evictable unknown delivery, between-write cancellation, fail-closed writer faults, exact tuple echo, session-level stale replies that preserve the multiplexed connection, and bounded ended-output retention; host tests cover idempotent tail replay versus conflicting bytes; snapshots, reconnect transport and operator stop pending |
| S4 SSH optional UI/CLI | not_started | None |
| S5 mandatory WSL guardian | not_started | None |
| S6 failure/feature restoration | not_started | None |
| S7 resource/fairness scheduling | not_started | None |
| S8 native acceptance | not_started | None |
| S9 distribution/release | not_started | None |

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
  - `cargo test -p warp-agent-bus -p remote_protocol`: all executed tests passed:
    16 remote_protocol unit tests; 231 agent_bus library tests (1 ignored);
    integration suites 20, 7, 6, 2 and 6 passed with 11 ignored child fixtures;
    zero failures.
  - The ignored tests require the owned Companion PTY child, real WSL or another
    platform and remain NOT RUN.
  - Failures found, fixes applied and rerun receipts: both test corrections and
    the E0599 fix were rerun; the final full run reported zero failures.
  - Host: macOS 24.6 arm64. SSH remote hosts, Windows, Linux and WSL were not
    exercised; V01-V30 remain NOT RUN.

Unverified behavior and remaining risks: passing host tests do not prove remote
  SSH/WSL lifetime, Windows admission, 300s retention timing, resource ceilings,
  snapshot import or reconnect transport. The application binary compile result
  is recorded separately when it completes. `guarded_remote_sessions_v1` remains
  unadvertised, so no client path reaches these operations.

Follow-up tasks / affected gates: Windows/Linux CI at a fixed SHA; real SSH and
  WSL acceptance; then account election, snapshot and CLI slices.
MEMORY update or reason no durable fact changed: MEMORY records that the guarded
  slice now compiles and passes host tests, while remote acceptance remains unrun.
```

## Evidence index

| Receipt | Source | Result | Coverage |
| --- | --- | --- | --- |
| S1/S3 source slice | `87e18c94`, `5163030f` and `99eaa0eb` on `feat/2.0.0` | implemented_unverified | Guarded retention, exact writer gate, CAS takeover, input epoch, bounded receipts, fail-closed writer handling, Unix peer UID admission and 300s ended-output retention written; rustfmt parse and isolated helper type-check only, not compiled or run |
| S1-d retention/admission | `99eaa0eb` on `feat/2.0.0` | implemented_unverified | Unix peer UID check, ended-output TTL and capacity eviction, exact tuple echo; no workspace build, native test or resource measurement |
| S1-e review closure | `efe9fd12` on `feat/2.0.0` | implemented_unverified | Generation-keyed ledger, non-evictable Unknown, between-write cancellation, guarded-only retention, fail-closed Unix credentials, Windows client SID recheck, explicit DeliveryUnknown and no connection-wide close for session stale; rustfmt parse and diff check only, no build/test |
| S1-f catalog prototype | Source commit after this record on `feat/2.0.0` | implemented_unverified | Bounded stable pages, boot/filter-bound cursors, revision-scoped watch ring with explicit gap, MRU resume excluding ended runs, state-age tracking, 32/64 ceilings; 7 isolated `rustc --test` cases passed with a uuid stub; not linked into the workspace crate and not wired to any session |
| S1-g build restoration | `e220a5c9` plus the follow-up fix commit on `feat/2.0.0` | verified_host_only | `remote_protocol` E0599 fixed, `protoc` 36.2 installed, stale write-count and idempotent-input test expectations corrected; `cargo check -p warp-agent-bus` and the full `warp-agent-bus` + `remote_protocol` test suites pass on macOS arm64; remote SSH/WSL and ignored fixture tests remain NOT RUN |
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
