# Warpai 2.0.0 guardian implementation execution prompt

Archived: 2026-11-18. Contract revision: R4, minimal justified CLI.
Inspection baseline: `db280c6b57c3eab7acb8c8e04ab8c8aab084828a`.
Purpose: a self-contained implementation handoff. Archiving this file implements
nothing, starts no agent, and authorizes no deployment or publication. When the
owner explicitly invokes this prompt for implementation, execute the applicable
steps below under the then-current repository rules and authorization.

## Role, objective and current truth

You are the implementation agent for Warpai 2.0.0. Deliver an optional, low-resource
SSH/WSL guardian while preserving the existing workflow. Build the smallest complete
implementation, not a Herdr clone. Communicate with the owner in Chinese. Code,
comments, filenames, commits and technical documents use English; maintain existing
English/Chinese product copy in its own language.

The current 2.0.0 package contains specifications only. No guardian implementation,
CLI, protocol change, native acceptance, benchmark, version bump or release has
been delivered by writing it. Published baseline is Warpai 1.9.0 / Companion 6.1.0;
verify current repository/release state before any version-sensitive action. Do not
report prior planning/document checks as evidence that runtime behavior works.

An earlier plan wrongly required local persistence and later expanded too many
Herdr-like commands. Those proposals were withdrawn. The owner's final scope below
supersedes them; do not infer work from historical D0-D3 ledger entries.

## Non-negotiable scope

1. Guardian is optional/default-off. A connection preference affects future launch
   defaults; each managed session records its actual launch policy. Never silently
   convert, restart, stop or adopt an existing native process.
2. Native local persistence is excluded. Keep the local terminal lifecycle, local
   Broker and local MCP unchanged. Local regression tests are still required.
3. SSH execution hosts are Linux/macOS/Windows. Direct WSL on Windows with the
   dedicated Linux guest helper is required, not a later optional feature.
4. One guardian per verified execution OS account manages all guarded projects,
   Agent vendors and shell/command sessions. No daemon per Agent, type, task or tab.
   Verified SSH/WSL routes to the same Linux account share it with isolated logical
   MCP/Store/admission scopes; different users/kernels/distros remain separate.
5. Enable guardian through the Warpai option, ensure a compatible remote daemon,
   keep processes and essential collaboration alive after desktop/bridge exit,
   and automatically rediscover information on reconnect.
6. The only public functional CLI commands are list, resume, stop; add standard
   help/version. Do not implement status/new/read/wait/rename/remove/completion,
   daemon command groups, redundant aliases, plugins or a second task manager.
   A future command needs a demonstrated unmet journey and a revised contract.
7. Preserve task/handoff/review/readiness/Coordinator semantics from 1.9.0.
   Terminal attachment, output reads and focus never acknowledge or accept work.
8. Use minimal memory/CPU: shared event/deadline scheduling, bounded queues and
   lazy history/snapshots. No hidden GUI, permanent UI polling or full framebuffer.
9. Install the CLI inside Companion/guest binaries. No PATH/profile/rc/global alias
   changes. Use verified absolute paths and a supported session-only shell function.
10. No public daemon port, root service, cloud account, vendor CLI replacement,
    native vendor daemon proxy, disk transcript, cross-host migration, reboot
    resume, multi-writer terminal or live daemon-upgrade handoff.

## Exact CLI semantics to implement

- `warpai list`: bounded current-account managed session rows with name/ID,
  project, process/attention/attachment state and daemon availability summary.
- `warpai resume`: valid current session first, otherwise the sole live session,
  otherwise the last used live session. No picker. A missing latest session falls
  back through live MRU, then newest created/stable-ID order if no MRU history.
- `warpai resume "name"`: one exact unique session label. Duplicate names return
  candidates; `--id <session-id>` gives exact targeting. A label is not a business
  task ID. A writer conflict requires explicit `--takeover`, never silent theft.
- `warpai stop`: immediately stop all guardian-managed runs in this execution
  account, regardless of current Agent/project/client, with no extra confirmation,
  selector or `--yes`. Exclude local/legacy unguarded/unrelated OS processes and
  other execution accounts. An explicit supported project filter can narrow scope.
- `warpai stop "name"` or `--id`: stop one exact resolved run.
- Help/version are static metadata and do not launch a missing daemon.

MRU changes only on successful foreground attachment or explicit foreground
selection. Listing, output activity, background refresh and task polling do not
change it. Resume never launches a new vendor process, accepts a result, transfers
Coordinator rights or revives a rebooted process. Stop is a native request with
observed/unknown results, not proof a task completed or was cancelled.

Stop-all is daemon-owned after admission: freeze exact session/run IDs, execute
bounded native stops and keep per-run receipts. If it kills the shell containing
the caller, remaining stops must continue after that CLI/socket disappears. A
same-operation-ID retry cannot include replacement/newly created runs. Empty stop
succeeds with count zero; partial/unconfirmed stop reports failure.

## Mandatory reading and precedence

At project startup read root AGENTS.md, README.md and MEMORY.md, then in order:

1. [Framework](README.md), [PRD](PRD.md), [CLI](CLI.md).
2. [Technical design](TECH.md), [API](API.md), [Tasks](TASKS.md).
3. [Verification](VERIFY.md), [Actual implementation ledger](IMPLEMENTATION.md).
4. [Release plan](RELEASE.md), [Herdr reference](HERDR.md).
5. Relevant existing Agent communication/WSL contracts under
   `specs/agent-communication-v2/` and `specs/wsl-1.6.0/`.

Current explicit owner corrections take precedence. R4 is the active scope;
feasibility text and historical ledger records are context, not a checklist.
If files are genuinely inconsistent, make a focused owning-spec correction using
this scope before code, record it, and continue. Ask only for a real missing goal,
permission, external runtime or destructive-risk decision. Do not ask to continue
routine steps. Never fill gaps by expanding the CLI or migrating native local work.

Use karpathy-guidelines for scope/assumptions/verification and ponytail for reuse
and minimal implementation. Read their available SKILL.md files and announce their
first use. Herdr is source/documentation reference only; do not control a live
Herdr instance or copy its code/dependencies without an explicit need/provenance.
No subagents or outgoing handoff/messages unless separately authorized.

## Execution protocol for every step

For each step: inspect inputs/callers, state the narrow change and acceptance,
update the owning contract if needed, implement, run focused checks, fix failures,
then update IMPLEMENTATION with actual method/source/evidence. A code patch can be
implemented_unverified; only relevant passed native proof makes it verified.
Do not close a TASKS checkbox from a proposal, mock assertion or submitted CI run.

Keep unrelated dirty work. Do not reset/clean/force-push. Check status before edits
and separate this work using a task branch/worktree when needed without dropping
uncommitted specifications. Never read/print .env, credential/token/key files or
private user configuration. Do not install/test on the owner's personal runtime.

Rust builds/tests and native acceptance run in GitHub/cloud with existing caches.
Local work is source/doc inspection and applicable syntax/fixture-script checks.
Use locked toolchains/dependencies and gh for GitHub; do not create a parallel
build/release pipeline. Do not push/dispatch/publish beyond session authorization.

## Step 0: establish a truthful baseline (S0)

How:
1. Record cwd, git status and current HEAD; verify the inspection baseline exists.
2. Read required root files and active specs. Check whether .codegraph exists;
   if so use CodeGraph before code-location searches, otherwise use rg/fd.
3. Inspect changes since the baseline by relevant paths. Revalidate any function
   or behavior changed since inspection; preserve unrelated commits/dirty work.
4. Read current ledger/task status. If prior implementation exists, inspect its
   code and evidence instead of restarting or assuming the historical not_started
   entries remain accurate.
5. Write one current-state entry: actual baseline, remaining requirements, known
   external constraints and which evidence can be reused.

Accept: no claim of implemented guardian without source evidence; no lost work.
Record: baseline SHA, relevant drift and the exact next uncompleted task.

## Step 1: inventory callers and ownership (S0)

How:
1. Trace managed launch/control/read/release from protobuf through host dispatch,
   native PTY, RunBinding and desktop/CLI callers.
2. Inspect at least these existing paths before touching their logic:
   - `crates/remote_protocol/proto/remote_server.proto`
   - `crates/remote_protocol/src/managed.rs`
   - `crates/agent_bus/src/companion.rs`
   - `crates/agent_bus/src/companion_service.rs`
   - `crates/agent_bus/src/companion_terminals.rs`
   - `crates/agent_bus/src/companion_tasks.rs`
   - `crates/agent_bus/src/companion_guests.rs`
   - `crates/agent_bus/src/native_pty_unix.rs`
   - `crates/agent_bus/src/native_pty_windows.rs`
   - `crates/agent_bus/src/screen_detection/native.rs`
   - `crates/agent_bus/src/transport.rs`
   - `crates/agent_bus/src/companion_main.rs`
   - `crates/agent_bus/src/wsl_companion_main.rs`
3. Locate all desktop remote launch, account settings, session EntityId mapping,
   quit/tab-close/language-restart/update and reconnect paths. Confirm exact
   symbols rather than invent module/function names from this prompt.
4. Map native WSL foreground detection separately from service-owned managed PTY
   launch; they have different ownership and cannot be equated by PID/name.
5. Enumerate impacted constructors, tests/fixtures, build feature gates, generated
   wire validators, packaging manifests and installer clients.
6. Identify true single Store/Broker ownership per existing logical scope. An
   account-global guardian may not duplicate a live legacy Broker over the same
   Store, merge namespaces or steal an old run to meet the topology goal.

Accept: one caller/owner table with before/after and regression-only local paths.
Record: concrete file/symbol names, current stop/revoke/Drop boundaries and reuse.

## Step 2: freeze identity, capability and storage contract (S0)

How:
1. Start from API.md and current protobuf. Add explicit guarded operations/policy;
   retain legacy field numbers/reserved slots/disconnect/release behavior.
2. Define guardian account-global identity/boot, logical service namespace,
   connection admission, project/root, session/run, writer attachment/generation,
   request/stop-operation IDs, output offsets and MRU sequence.
3. Prove how verified same-account SSH/WSL helpers discover one guardian without
   treating different legacy service IDs, users or kernels as identical.
4. Freeze supported capability and incompatible-helper behavior. Never spawn a
   second guardian, kill a live runtime or downgrade a Store just for version match.
5. Define minimal catalog tables/versioning/transactions using existing SQLite
   infrastructure. Store only nonsecret identity/policy/timestamps/MRU/exit evidence;
   no argv, environment, capability token, drafts, screen or relaunch plan.
6. Define bounded account metadata discovery separately from project task/content
   authority. IDs/environment hints/labels are selectors, not authentication.
7. Define legacy/new/mixed-policy compatibility and a safe treatment for live legacy
   Store owners. Reuse/forward through a verified compatible owner when possible;
   otherwise fail only affected guarded operations until safe reconciliation. Do
   not silently split ownership or terminate legacy work.
8. Freeze parser flags, CLI exit codes, direct-attach escape and provisional limits.
   Do not add APIs for withdrawn read/wait/admin tools.

Accept: wire/schema/compatibility fixtures specified before caller code changes.
Record: fields, state diagrams, migration/recovery/rollback and rejected options.

## Step 3: implement account guardian election and startup (S1)

How:
1. Reuse Companion's detached service implementation and private IPC mechanisms.
   Add only the account-global guardian discovery/election needed by the contract.
2. On explicit enable or enabled reconnect, connect/probe a compatible owner first.
   Start only if no verified owner exists, under the account's exclusive startup
   lock. Race losers connect to the winner.
3. Validate private endpoint owner/peer and runtime birth/boot. A timeout/stale lock
   alone does not permit unlinking a live socket or replacing a running owner.
4. Keep protocol scopes/adapters and Store handles isolated within the shared
   runtime. Check compile-time features do not accidentally expose WSL-only handlers
   through unadmitted SSH calls or vice versa.
5. Keep launch/health errors bounded and nonsecret. Exclude UI rendering, usage
   polling, recursive watchers, public ports and permanent login installation.
6. Verify Linux/macOS terminal/SIGHUP independence and remote Windows SSH launcher
   Job independence in disposable native fixtures; detached flags alone are not proof.

Accept: concurrent enabled clients/aliases/Agent kinds yield one guardian per
execution account; guard-off native/local behavior unchanged. Cover V19/V27.
Record: startup election/peer validation and OS-specific ownership methods.

## Step 4: implement guarded native session lifetime (S1)

How:
1. Add explicit connection_owned versus guarded session policy at new launch.
   Do not mutate an existing run when a preference changes.
2. Reuse native validated executable/args/cwd and original Agent adapters/auth.
   A managed shell/command gets a PTY without synthetic Agent rights.
3. For guarded sessions, let the guardian own process group/Job, PTY handles,
   reader/writer, output state and optional exact-run RunBinding.
4. Split client detach from native stop. On guarded transport loss release/fence
   the attachment and its pending human input, but retain the run and collaboration.
   Keep original legacy disconnect/release cleanup byte-for-byte in meaning.
5. Verify descendants, output EOF and shell exit separately. Never infer the entire
   owned group stopped merely because the visible shell or client disappeared.
6. Mark launch attempts uncertain when replies are lost. Reconcile exact request/
   session/run rather than retrying with a new ID or adopting an arbitrary PID.

Accept: V01/V02/V08/V11, mixed policies and same native birth identity after detach.
Record: exact Drop/stop/revoke changes, failure paths and run-binding lifetime.

## Step 5: implement bounded catalog, MRU and cleanup (S1/S6)

How:
1. Reuse authoritative session objects; do not build a competing task registry.
2. Expose paged sanitized account catalog plus admitted project/task projections.
   Include whole-filter totals and bounded daemon availability facts.
3. Atomically update last_active_sequence only on successful foreground attach or
   explicit foreground selection. Preserve deterministic creation-sequence fallback.
4. Implement current/unique/MRU selection, excluding ended/unknown/stale-boot runs.
   Keep duplicate label disambiguation stable and case-sensitive; support exact IDs.
5. Count queues/parser/ring/snapshots/ended history in one account memory budget.
   Do not allocate max buffers at launch or pin all history while listing.
6. On native exit retain bounded recent output for the specified TTL/pressure
   policy, independently retaining durable task results and unresolved attempts.
7. Exit after the genuine idle window only when no owned work/draining IO/active
   request/retained output remains. A catalog row alone cannot keep a daemon alive.

Accept: V13/V20/V23/V29; reads/background refresh leave MRU and task ACK unchanged.
Record: schema/GC algorithm, accounting, limits and TTL evidence.

## Step 6: prove compact terminal fidelity before live UI (S2)

How:
1. Compare current native parser/model, vt100 readiness screen, shell integration,
   command blocks and existing snapshot/transport facilities. Choose the smallest
   canonical state representation; avoid divergent double parsers.
2. Define versioned dimensions/modes/cursor/main+alt buffers/bounded history and
   the shell/block metadata needed for continuing Warpai behavior.
3. The readiness screen's current memory-only contract is not permission to log or
   serialize its private evidence. Provide an authenticated transient terminal
   projection without persistent transcripts or side-effect replay.
4. Build deterministic export/import fixtures for prompt, alternate screen, Unicode,
   split escape/UTF-8, protected draft, block boundaries and resize.
5. Freeze an immutable snapshot at offset N under an ordering barrier. Include
   parser-safe incomplete-sequence state, size/revision/digest and bounded paging.
   Continue tail from N; if ring overflow/expiry occurs, report a gap and resnapshot.
6. Allocate only on attach/resync, cap account serializers and release on every
   success/error/cancel path. Never hold parser/global locks across network IO.
7. Build a static native imported-terminal artifact and review semantic state plus
   screenshots before connecting real session data. Repeat OSC/clipboard actions
   must not occur during replay.

Accept: V06/V07/V23 and measured snapshot size. A raw text tail is not fidelity.
Record: exact encoding/native fields, alternatives, images and model comparison.

## Step 7: implement one writer, ordered input and safe detach (S3)

How:
1. Add runtime-issued attachment ID/generation and atomic acquisition/takeover.
   Revalidate account/project/boot/run and generation at native write/resize time.
2. Serialize human input and peer delivery through the native PTY writer while
   preserving existing readiness, cancellation, redraw and draft guards.
3. Track accepted/written/failed/unknown input dispositions; sequence/content
   mismatches fail, known duplicates do not enqueue twice, partial writes are not
   automatically replayed after reconnect.
4. Atomically fence old-generation queued bytes on disconnect/takeover. Bytes
   already written cannot be recalled; report uncertainty rather than success.
5. Couple snapshot catch-up, terminal size/revision and writer activation. Listing
   a session or selecting metadata grants no input or resize permission.
6. Implement/test the CLI-only local detach escape, including byte splits and a
   literal-prefix form. Restore TTY modes on every normal/error/signal exit. Do
   not forward a supposed detach key as destructive shell/Agent input.

Accept: V04/V07/V09/V10/V26. No stale input or false task acknowledgement.
Record: lock/order boundaries, writer receipts, escape machine and native results.

## Step 8: preserve headless collaboration and multi-task semantics (S1/S6/S7)

How:
1. Trace existing per-project Broker/Store and native readiness/wake code. Reuse
   their task queues, message IDs, revision checks, run fences and review lifecycle.
2. Keep guarded bindings and essential delivery alive without desktop models.
   The remote service must not rely on a focused view to schedule peer work.
3. Preserve serial tracked execution per Agent; parallel tasks use distinct live
   sessions. Submission releases execution slot while review stays pending.
4. Permission questions, unknown state, protected drafts, overtaken follow-ups and
   unresolved results remain pending according to existing rules. Do not auto-
   approve, reassign on timeout, treat input receipt as execution, or mark read as
   resolved. Coordinator remains the exact proven native run.
5. Exercise multiple Agents, shell/server/test sessions and isolated projects.
   A blocked/failed/noisy session must not freeze other queues or cause auto-spawn.
6. Keep UI/file responsibilities honest: Explorer/editor/Review reconnect lazily;
   release/cancel client SFTP/WSL transfer handles and retain uncertain-write
   receipts. Do not keep watchers or repeat uncertain writes after reconnect.

Accept: V05/V09/V14/V17/V21/V22, plus existing task/handoff/review regressions.
Record: headless feature matrix and actual notification/task ownership methods.

## Step 9: implement required direct WSL behavior (S5)

How:
1. Use the dedicated guest binary, verified payload and literal wsl.exe distro/
   user/exec arguments. Confirm Linux root/account identity and native launch cwd.
2. Keep ordinary guest foreground MCP discovery/aliases/vendor auth unchanged.
   Guarded launch enters a guest-service-owned PTY with verified process ancestry/
   birth identity; no Windows MCP injection or arbitrary background PID adoption.
3. Confirm account-global guardian sharing only for proven same-kernel/account
   SSH/WSL routes, preserving their logical namespaces/admission. Separate distros
   and users cannot share through a label, path or username string alone.
4. Close all wsl.exe bridges and Warpai desktop in real Windows/WSL fixtures, then
   reenter and prove same Linux process/run and working collaboration.
5. Measure whether the supported guest lifecycle needs a minimal keepalive; do
   not use a busy loop or keep unrelated distributions awake by default.
6. Terminate the distro/WSL host and restart. Report interruption/unknown and use
   guarded recovery; never claim process survival or inherit Coordinator authority.
7. Check nested SSH routes: final SSH host owns its run, not the outer WSL shell.

Accept: V03/V12/V14/V18/V27 on real WSL. Linux SSH mocks are insufficient.
Record: distro/user/guest provenance, lifetime proof and stop-boundary behavior.

## Step 10: implement only the private core CLI (S4)

How:
1. Reuse the Companion executable with a ctl dispatcher and existing protocol
   client. Do not add another daemon, launcher dependency or one process per Agent.
2. Implement list from bounded catalog/state only; no snapshots, writer, MRU change
   or implicit daemon start for a diagnostic read.
3. Implement resume through Step 5 selection and Step 7 lease/TTY path. Named
   ambiguity yields candidate IDs; quoted Unicode and leading-dash labels stay
   literal. A conflict never silently changes target.
4. Implement stop-one and daemon-owned stop-all. Freeze run/operation identities,
   continue after caller-PTY death, deduplicate transport retries and return exact
   native requested/observed/unknown receipts with bounded concurrency.
5. Use the installed verified absolute helper path. Expose a temporary warpai
   function only in participating supported interactive remote shells and only
   without overriding user commands. Preserve original argv; no eval/PATH/rc edits.
6. In independent SSH/WSL shells support the absolute helper path; do not promise
   bare warpai works globally. Do not inject wrappers into a running vendor CLI,
   native local shell, unknown shell or unrelated SSH session.
7. Add help/version and narrow parser/TTY/exit-code fixtures. Reject withdrawn
   commands rather than keeping unshipped compatibility aliases.

Accept: V25/V26/V29/V30 plus Step 7 detach tests. No extra command family.
Record: actual dispatcher/parser/quoting paths, default selection and batch methods.

## Step 11: add opt-in native UI and automatic reconnect (S4)

How:
1. Reuse current settings/sidebar/panel widgets and tokens. Add target-specific
   future-launch preference and explicit actual launch policy, default-off.
2. First build fixed-data states for enabled/off/unavailable/disconnected/blocked/
   ended/unknown/writer-conflict/multiple sessions/partial stop. Review EN/ZH,
   themes, small windows, scaling, keyboard and focus/draft behavior.
3. After static QA, bind verified SSH/WSL route and capability. Enabling ensures
   the daemon but never converts existing run policy. Unsupported guard remains
   explicit; no silent fallback launch.
4. Reconnect automatically fetches daemon identity/catalog and admitted project/
   task/attention state, using a bounded boot/cursor-scoped shared change watch
   with low-frequency fallback and rescan on a gap.
5. Restore saved view or live MRU view only when the writer is free; display a
   conflict without stealing it. Attach other terminals only when selected.
6. Close/quit/language restart releases guarded client attachments while current
   local/legacy behavior remains intact. No mandatory extra quit prompt for normal
   users. Explicit bulk stop uses the same exact-run daemon batch, not name kills.
7. Restore files/editor/Review lazily, preserving selected confirmed project and
   current uncertainty/draft policy. Reconnect changes no task/read/ACK authority.

Accept: V01/V02/V15/V16/V17/V20/V28, static then live native screenshots.
Record: actual models/events, state projection, images and regression boundaries.

## Step 12: minimize idle overhead and ensure fairness (S7)

How:
1. Inspect current guarded-path 300 ms per-Agent wake sampling before replacing
   it. Preserve necessary quiet/paste/Enter timing and readiness facts.
2. Trigger relevant work from PTY output and Broker changes; index pending work by
   session/run instead of rescanning all Brokers for each Agent.
3. Use shared ready queues and nearest-deadline sleeping for quiet windows,
   process observations, retention and delivery delays. Coalesce dirty redraws
   without losing state transitions; protect against lost wake races.
4. Use bounded per-session work batches, fair ready-session turns and reserved
   stop/control capacity. No global lock across blocking writes/snapshots/Git/IO.
5. Reuse the async runtime; inventory unavoidable native reader/writer thread and
   stack cost before replacing them. No runtime/timer/poll loop per task.
6. Count parser/history/ring/input/snapshot/ended-output and project Broker/cache
   memory. Allocate lazily, close idle safe caches, and reject admission rather
   than killing running Agents or deleting unresolved task evidence.
7. Test one output flood, blocked writer and snapshot storm alongside quiet work
   and native stop. Verify controls and other sessions still progress.

Accept: V19/V21/V22/V23/V24 with native fairness and cost receipts.
Record: algorithms/locks/notifications/deadlines, lost-wake proof and measured cost.

## Step 13: handle native failures, restart and cleanup (S6)

How:
1. Separate transport loss, observation loss, explicit stop, native exit, daemon
   crash, machine shutdown and distro termination in implementation and UI.
2. Native stop must keep ownership evidence until entire group/Job termination is
   confirmed; failed/unobserved stop stays unknown and blocks unsafe duplication.
3. After runtime restart, prior live catalog rows are interruption evidence, not
   runnable plans. Reject stale boot/lease/output cursors and never auto-relaunch.
4. Apply bounded final-output TTL/pressure eviction and genuine idle exit without
   dropping durable task results, unresolved attempts or execution receipts.
5. Keep same-operation-ID stop receipts bounded and recoverable after lost client
   replies. Never translate lost transport into success or replay a file mutation.
6. Verify compatible runtime reuse and refusal of destructive incompatible upgrade.
   Do not remove a live helper/dependency or force an older catalog schema.

Accept: V10-V14/V16/V18/V23/V25. No false exit, task resolution or reboot resume.
Record: fault matrix, GC/cleanup boundaries, errors and safe reconciliation.

## Step 14: integrate private packaging without global changes (S9)

How:
1. Reuse existing Companion installers, dedicated WSL packaging and release jobs.
   Inspect `script/companion/package.py`, `script/companion/package-wsl.py` and
   installer manifests before editing feature/version/provenance information.
2. Ship the dispatcher with the existing binaries, correct native dependencies and
   scope capabilities. No separate local guardian package or PATH/profile installer.
3. Stage verified helper/runtime versions side by side if live work depends on
   old files. GC only unused versions; install/update cannot silently stop runs.
4. In disposable profiles validate fresh/repeat install, spaces/Unicode, corrupt
   payloads, wrong architecture/user/distro, uninstall with live runs and downgrade.
5. Keep proposed Companion 6.2.0 as a proposal until actual compatibility/version
   freeze; protocol major changes follow contract breaks, not desktop version.

Accept: V08/V16/V18/V26 and existing installer/guest provenance tests.
Record: file layout, signing/checksums/source/version and native install receipts.

## Step 15: run source-matched cloud verification (S8)

How:
1. Use the closest existing tests and meaningful failure fixtures; do not invent
   tests merely mirroring implementation. Freeze exact source before evidence.
2. Extend `.github/workflows/validate-agent-communication.yml` and
   `.github/workflows/validate-remote-companion.yml` where needed. Inspect current
   dispatch inputs, OS/features and trusted disposable profile setup before use.
3. On authorized GitHub/cloud runners, run relevant locked checks including:

```sh
cargo test -p remote_protocol --locked
cargo test -p warp-agent-bus --locked
cargo check -p warpai --bin warpai --locked
cargo check -p warpai --bin warpai --features warp_platform --locked
```

4. Include native SSH execution hosts Linux/macOS/Windows and actual Windows/WSL,
   applicable shipping architectures and cross-OS routes in VERIFY.md.
5. Preserve existing local/legacy Agent, task/storage/handoff/review, file, scope,
   WSL, installation and locale regression coverage. Keep default and warp_platform
   builds compiling; mocks do not close native process/UI/guest lifetime gates.
6. Inspect every relevant job's actual outcome and artifacts. Fix failures and rerun
   affected checks. A new source change invalidates affected earlier receipts.
7. Follow owner monitoring preference: stay in the active turn, check CI at the
   recorded cadence and do not detach a monitor. Keep waits interruptible and
   progress communication timely; no long unresponsive blocking sleep.

Accept: required V01-V30 native evidence passes on source-matched artifacts.
Record: run/job URLs, SHA, OS/arch, versions, command, expected/actual and artifacts.

## Step 16: measure and freeze low-resource budgets (S7/S8)

How:
1. Read TECH/VERIFY limits. They are provisional targets, not proven performance.
2. On documented quiet reference hosts compare existing polling versus guarded
   event-driven mode for 0/1/8/32 sessions, attached/detached, same/multiple projects,
   noisy/quiet/blocked sessions and concurrent attach/snapshot workloads.
3. Measure base/incremental/peak RSS and PSS where available, guardian CPU-time
   deltas per wall second/core, threads/context switches, parser/cache/queue size,
   stop/wake latency and attach/replay traffic. Exclude vendor process cost from
   guardian overhead while accounting for every guardian worker/Broker.
4. Repeat at least three warmed 60-second idle samples and record host noise;
   test flood/reconnect peaks separately. Verify an empty daemon exits as designed.
5. Initial design targets are base RSS <=32 MiB, incremental 120x40 idle session
   <=2 MiB, 8-session idle CPU <=0.2% of one core on the stated quiet 2-vCPU VM.
   Initial memory controls include 32-session admission ceiling, lazy 256 KiB ring,
   32 KiB pages, 2 MiB transient snapshot ceiling and 64 MiB account terminal-data
   cap. These are not permission to omit uncounted native/parser/Broker costs.
6. If results miss targets, optimize the observed cause and remeasure. If a bound
   must change for correctness/platform reality, document measurements, tradeoff
   and matching spec/test changes. Do not silently waive or advertise a target.

Accept: reproducible measurements support frozen budgets and bounded backpressure.
Record: raw nonsecret counters/conditions, summary comparisons and chosen limits.

## Step 17: document accepted product behavior and prepare review (S9)

How:
1. Update all actual-work records and task statuses. Explain exact files/symbols,
   methods, identities, concurrency, compatibility, failure handling and evidence.
2. Prepare bilingual README/source SSH/WSL/Agent guide and website text in existing
   relevant positions. Explain optional guardian, scope, reconnect, core commands,
   host loss and measured limits as product behavior, not a changelog.
3. Keep unaccepted claims/drafts unpublished. Never imply local persistence, every
   UI feature runs headlessly, arbitrary process adoption or reboot survival.
4. Preserve gh-pages assets, accepted mobile header and production base URL. Product
   docs must match verified helper capability and eventual downloadable artifacts.
5. Read RELEASE.md and existing release contract. Prepare reviewable candidate,
   packaging/acceptance receipts and proposed release notes. This prompt alone
   does not grant merge/tag/publication authority; honor actual session approvals.
6. Do not publish/download-surface changes against nonexistent artifacts. Follow
   the established release gate if separately authorized; otherwise leave a concrete
   reviewable result and precise remaining publication steps, not a false release.
7. Update MEMORY with lasting choices and receipt links, without secrets or
   duplicating obvious code. Preserve source provenance and existing releases.

Accept: complete implementation/verification package and honest release state.
Record: exact candidate source, verified scope, documents, risks and remaining gates.

## Required record after each implementation unit

Use IMPLEMENTATION.md's full template. At minimum include:

- Requirement/task/case IDs, baseline and actual commit/PR.
- Before/after behavior and changed files/symbols/callers/features/fixtures.
- Native ownership, state/identity, API fields and trust-boundary validation.
- Actual ordering/locking/notifications, queue/capacity and error/cleanup methods.
- Store/migration/privacy, native platform/dependency and compatibility details.
- Why this approach was chosen and which simpler existing path was reused.
- Source-matched tests/CI/artifacts, expected versus actual, failures and reruns.
- Numeric resource measurements where relevant, unverified behavior and next gate.
- MEMORY update or explicit reason no lasting fact changed.

Every document/evidence status must distinguish proposed, implemented_unverified,
verified, failed and blocked. Record real blocked dependencies without inventing
claims or repeatedly asking permission for normal engineering choices.

## Final acceptance and handoff checklist

- [ ] Default-off local/legacy behavior preserved; no native local guardian/Broker move.
- [ ] One guardian per execution account, no Agent-type split or Store/scope merge.
- [ ] SSH and real WSL survive client exit with same native process/run evidence.
- [ ] Headless collaboration, task serialization and parallel sessions remain correct.
- [ ] Bounded faithful snapshots, one writer, no stale/replayed uncertain input.
- [ ] Automatic metadata/MRU restoration without writer theft or authority inheritance.
- [ ] Only list/resume/stop + help/version; defaults exactly match owner requirements.
- [ ] Stop-all finishes after caller loss, reports exact per-run native results.
- [ ] CLI stays private to Companion, no PATH/profile changes; safe local detach.
- [ ] Native failure/reboot/retention/upgrade behavior truthful and tested.
- [ ] Measured CPU/memory/fairness meet frozen documented resource budgets.
- [ ] Required V01-V30 native/platform/regression evidence passes on exact source.
- [ ] Detailed methods/decisions/receipts recorded and MEMORY updated appropriately.
- [ ] Product documents and release state match actual verified/authorized delivery.

When implementation is invoked, persist through the authorized work instead of
returning only a plan. The final Chinese report should state what changed, verified
source/evidence, remaining real risks/gates, unperformed work and MEMORY status.
Do not claim release, performance compliance or feature completion without proof.
