# Warpai 2.0.0 actual implementation ledger

Updated: 2026-11-18. Scope revision: R4 optional SSH/WSL guardian.

## Current truth

Design documents plus one unverified runtime slice exist. The slice adds additive
guarded launch policy, retained native lifetime on connection loss, and
attach/detach/takeover writer fencing in the existing SSH/WSL Companion terminal
registry. Guarded input/resize/read/detach/release now require the exact writer
tuple, takeover fences the observed generation, and detach/transport loss
invalidates queued input through a separate writer epoch. Unit coverage and two
Unix integration tests are written but not compiled or run. No guardian election,
account catalog, snapshot, CLI, desktop opt-in, scheduler, packaging/version change
or release is implemented. V01-V30 runtime acceptance is NOT RUN. No guard client
can reach the slice because `guarded_remote_sessions_v1` is deliberately not
advertised. Runtime inspection baseline is
`db280c6b57c3eab7acb8c8e04ab8c8aab084828a`; all 2.0.0 changes are uncommitted.
Revalidate current HEAD at implementation startup rather than assuming an earlier
observed repository position is still current.

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
| S0 contract/caller inventory | in_progress | Caller/ownership inventory and guarded wire fields frozen ([evidence/S0-baseline.md](evidence/S0-baseline.md)); account catalog/watch and storage schema pending |
| S1 account guardian/native ownership | in_progress | Guarded policy retains SSH/WSL Companion PTY/threads across connection loss; full writer gate and input epoch added after review; account-global election, catalog and native proof pending |
| S2 compact terminal restoration | not_started | None |
| S3 attachments and ordered IO | in_progress | Exact attachment tuple for guarded controls, CAS takeover generation and queued-input invalidation; in-flight write uncertainty, snapshots, dispositions and reconnect transport pending |
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
Implementation commit / PR: none; uncommitted worktree changes

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
Implementation commit / PR: none; uncommitted worktree changes

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
Implementation commit / PR: none; uncommitted worktree changes

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

## Evidence index

| Receipt | Source | Result | Coverage |
| --- | --- | --- | --- |
| S1/S3 source slice | Uncommitted companion diff on `db280c6b` | implemented_unverified | Guarded retention, exact writer gate, CAS takeover, input epoch and tests written; not compiled or run |
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
