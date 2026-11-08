# Agent execution prompt: Warpai 1.9.0 R2 independent-process lifecycle

Date: 2026-11-08. Status: executable work instructions when explicitly invoked.
Writing this file does not start implementation.

## 1. Starting point and assignment

Assume the previous 1.9.0 implementation round is provisionally complete. Continue
from its current source and existing decisions. Do not restart the original
milestone, rewrite the attention panel, reopen every old checkbox, or recreate
the previous execution prompt. Preliminary completion is the starting assumption,
not permission to invent missing CI receipts or claim unperformed acceptance.

Your assignment is the second-round delta: adapt useful Superagent execution
mechanics to independent Warpai CLI Agent processes. Audit the eight R2 groups,
verify scenarios SP01–SP10 and fix only demonstrated gaps in current behavior.
An already-correct path closes with evidence and no new production code.

The required result is reliable separation of:

`durable task → revision → execution attempt → native process run → CLI conversation when observable → owned background resources`

These are related identities, not a demand to add six new entities or tables.
Reusing a process is allowed. An independent process is not automatically an
isolated checkout, a fresh conversation, or proof that all children stopped.

Read these current sources of truth:

- [R2 process review](specs/1.9.0/PROCESS-REVIEW.md): eight responsibilities and evidence.
- [TODO ledger](specs/1.9.0/TODO.md): update only R2 and directly affected receipts.
- [Product invariants](specs/1.9.0/PRODUCT.md) and [technical plan](specs/1.9.0/TECH.md): preserve first-round behavior.
- [Previous execution prompt](specs/1.9.0/AGENT-EXECUTION-1.9.0.md): context only, not another checklist to restart.
- Active [API](specs/agent-communication-v2/API.md),
  [Worktree contract](specs/agent-communication-v2/WORKTREE-COLLABORATION.md) and
  [handoff contract](specs/agent-communication-v2/HANDOFF.md), including superseding corrections.

Follow current owner instructions and applicable AGENTS.md over this file.
Within R2, this prompt supplies procedure and PROCESS-REVIEW supplies rationale;
record progress in TODO, not in a second competing ledger. If first-round planning
status text is stale, record the current baseline rather than interpreting the
stale text as an instruction to reimplement the first round.

## 2. Scope and explicit exclusions

Required: process/task/context boundaries; background lifetime; partial startup;
existing-queue backpressure and fairness; independent failure/retry ownership;
checkout write coordination; duplicate handling and supervisor reconstruction;
available launch/configuration provenance.

Excluded:

- External event intake, GitHub/Feishu listeners, webhooks and connectors.
- danger-full-access remediation, permission-mode changes, new permission prompts
  or sandbox implementation. Preserve existing credential and identity safeguards.
- Automatic Agent launching/replacement/pooling, a new global worker scheduler,
  new project pause/drain controls, model routing and prompt/model editors.
- A new task-job registry, terminal transcript ingestion, vector storage, another
  daemon, permanent Router, cloud AI/accounts/billing, telemetry or cross-host federation.
- Forced fresh process per task, automatic conversation resets, undocumented
  vendor resume commands, global CLI configuration edits or account/model switching.
- Unrelated redesign, dependency upgrades, terminal-core rewrites and release work.

No automatic launcher is needed to test capacity/fairness of existing queues.
Existing process-group/Job Object ownership must not become authority to kill
arbitrary jobs in a user-attached terminal.

## 3. Work rules

1. Use Chinese for progress and final communication; English for code, comments,
   tests and technical documents. Follow the existing localization system for UI.
2. Apply karpathy-guidelines and ponytail: trace the real owner/callers, define the
   failure, reuse existing mechanisms and produce the smallest complete fix.
3. Do not spawn subagents unless separately authorized by the owner or applicable
   instructions. A/B/C, worker processes and providers in scenarios are disposable
   test fixtures, not authorization for paid/live vendor sessions.
4. Read CodeGraph first only when `.codegraph/` exists. Otherwise use `rg` for
   symbols/callers. Do not scan unrelated history, credentials or personal sessions.
5. Preserve dirty work and first-round implementation. Do not reset, clean, move
   user files, force-push or change working directories of running user processes.
6. Rust builds and native acceptance follow MEMORY.md: currently GitHub with
   caches, local source/script checks only. Do not install/update local toolchains
   or the owner's app to make a test convenient.
7. Fix a demonstrated bug in the shared owner before adding transport-specific
   behavior. Include actual affected callers, tests, fixtures and contracts.
8. New nontrivial logic requires a focused behavioral regression; read the actual
   assertion and prove a test executed. String-presence checks alone cannot prove
   process lifetime, transactional behavior or recovery.
9. Never store secrets, personal raw transcripts, draft contents or unbounded
   command output in logs, fixtures, receipts or MEMORY. Use synthetic task text.
10. Give concise progress updates at least once per minute while working. Continue
    through routine decisions and failures; ask only for a necessary missing input,
    permission, material scope change or genuinely destructive action.

## 4. Stage A — Capture the inherited baseline without reopening the first round

Perform in order:

1. Read root/nested instructions, README and MEMORY, then R2 review and the relevant
   active contracts. Inspect current source, not just the old research commit.
2. Record HEAD, branch, worktrees, dirty paths and available first-round receipts
   in the R2 section of TODO. Preserve source-only versus runtime-verified status.
3. Identify inherited behavior to preserve: explicit TaskStart/answer/result facts,
   task-revision-linked follow-ups, stale-message reconciliation, the attention
   projection, review/integration separation, handoff pins and transport authority.
4. Read only relevant inherited tests/receipts. Reuse them if applicable; execute
   additional regression checks only where R2 changes can affect them. Do not run
   the whole first-round workflow simply to satisfy an old checklist.
5. If an R2 path reveals an inherited defect, record its trigger, scope and dependency
   and fix it where needed for R2. Do not roll back otherwise unrelated first-round
   work. Unrelated old pending gates remain separately pending.

Output: one short R2 baseline note with source SHA, inherited diff boundaries and
specific evidence/limitations. Exit: current work can proceed without overwriting
prior work or relying on invented completion evidence.

## 5. Stage B — Classify R2 gaps and map execution ownership

Start with these source areas; re-resolve symbols against current HEAD:

| Concern | Inspect |
| --- | --- |
| Task/attempt/run | `crates/agent_bus/src/lib.rs`, `storage.rs`, `project_memory_storage.rs` |
| Native ownership | `crates/agent_bus/src/native_pty.rs`, `native_pty_unix.rs`, `native_pty_windows.rs` |
| Per-launch binding | `crates/agent_bus/src/session.rs`, `launch.rs`, registration and readiness callers |
| Shared inbox/wake | `crates/agent_bus/src/transport.rs`, `transport_orchestration.rs`, `readiness.rs` |
| Remote owners | `companion_terminals.rs`, `companion_tasks.rs`, `ssh_remote.rs` and current app SSH/WSL peer paths |
| Questions, reservations, recovery | Current Store operation handlers and related tests |
| Supervision/context | `mcp.rs`, bootstrap/execution-thread/handoff reads, role-change notifications |
| Existing UI | Current `app/src/agent_communication/` projection and attention/handoff controls |
| Regression harness | `crates/agent_bus/tests/{native_pty,managed_agent,readiness,coordination,ssh_companion,ssh_native_mcp,wsl_workspace}.rs` and inline tests |

For each R2-01–R2-08:

1. Trace caller → handler → transaction → process/transport owner → projection →
   existing action. Read failure and cleanup branches, including all platform variants.
2. Separate what the host owns from what it merely observes. Identify exact run,
   process birth evidence, root/checkout and remote account/generation when applicable.
3. Record available facts and unavailable vendor details. Do not add a field for
   something the system cannot observe or a capability solely to resemble Harness.
4. Classify as `existing_verified`, `existing_unverified` or `demonstrated_gap`.
   Verified requires relevant passing evidence, not source presence alone.
5. For a gap, write the smallest expected-versus-observed reproduction and identify
   every affected caller, transport, fixture, contract and user consequence.

Output: append an R2 classification/evidence table in TODO. Exit: every R2 group
has a concrete verification/fix action; no speculative runtime redesign is selected.

## 6. Stage C — Specify only necessary deltas before code

For each demonstrated gap:

1. Describe the desired behavior and transition/fact in TECH, linking the exact
   R2 and SP identifiers. Prefer existing fields, operations and evidence descriptors.
2. If the API must change, update its active contract first: arguments/results,
   defaults/omission, validation, caller authority, stale/replay behavior, bounds,
   errors, compatibility and capability fallback. Never silently alter semantics
   of legacy fields or make every client appear to support a vendor feature.
3. If storage must change, prove current records cannot express the requirement;
   specify backup, migration, restart and protected-retention cases before editing.
   New schema/protocol versions are not automatic scope.
4. Define the test oracle: which persisted facts, owned process observations,
   notification IDs and forbidden effects prove correctness. Distinguish protocol
   correctness from unobservable model reasoning or conversation contents.
5. Do not introduce UI unless the gap cannot be represented truthfully through
   existing detail/status/actions. If UI is necessary, read warp-ui-guidelines,
   establish existing-panel visual references and static fixtures before live wiring;
   use ui-design-handoff-fidelity and perform native screenshot QA.

Exit: a small, reviewable implementation delta with a meaningful failure test.
Resolve routine choices yourself. Do not request approval merely because more
than one implementation exists; ask only if the proposed fix exceeds this scope.

## 7. R2-01 — Reused process context and replacement runs (SP01)

Steps:

1. Inspect TaskStart, bootstrap, execution-thread bindings, assignment delivery
   and the previous handoff of an Agent that completed another task.
2. Verify the new task carries its own objective, acceptance, current revision and
   exact input refs. Old conversation or remembered instructions cannot grant new
   task authority. Do not duplicate all historical context into the new input.
3. Verify a replacement process receives a fresh run identity and normal admission;
   unresolved prior execution is reconciled before another owner starts work.
4. Use vendor conversation IDs only if an existing supported mechanism exposes
   them. Do not read personal session stores, reset the CLI, or manufacture continuity.

SP01 setup: use one disposable participating process for task X and then unrelated
Y with intentionally conflicting synthetic objectives. Also replace the process
while an attempt is unresolved. Assert current task payload/bindings and old-run
rejection, not that a model has psychologically forgotten X. Check recovered
bootstrap inputs without claiming unobserved vendor conversation restoration.

Completion: explicit task boundaries and replacement-run behavior are proven;
no forced process-per-task policy or new conversation service is introduced.

## 8. R2-02 — Turn/task/process/child lifetimes (SP02, SP03)

Steps:

1. Trace NativePty exit_code/is_active/stop and every caller that interprets them.
   Read Unix process-group and Windows Job Object paths separately.
2. Reuse the existing `owned_group_can_stop_background_children_after_observing_leader_exit`
   fixture. Verify that an exited leader can still have owned active children.
3. Inspect task submit/review/cancel and native terminal close/restart paths.
   A task transition must not accidentally invoke broad process cleanup.
4. Separate owned managed children, attached user terminals and detached daemons.
   Never infer per-task job ownership from ancestry alone or apply one backend's
   kill authority to another. Preserve unknown ownership explicitly.
5. Preserve a preview server intentionally left for review. A later exit/result
   from that old work must not wake new implementation or mutate a new attempt.
6. Verify cancellation reports only the observed owning-run outcome. Task stop
   confirmation does not prove every unrelated process in a terminal has exited.

SP02: spawn the existing owned leader/child fixture, observe leader exit, verify
active ownership and safe owned-child shutdown; assert unrelated fixture processes
remain alive. Clean up all disposable resources with bounded fixture teardown.

SP03: submit a ready task with a synthetic preview child alive, enter review and
start later authorized work through current supported semantics. Verify no
automatic kill, no stale wake into a replacement attempt and truthful unknown
task-to-job ownership. Do not build a job registry just to make this observable.

Completion: lifecycle and kill boundaries are proven on applicable native paths;
no background-resource manager or new auto-kill policy is introduced.

## 9. R2-03 — Partial startup and readiness (SP04)

Steps:

1. Trace existing worktree creation, native launch, per-run binding, MCP handshake,
   native readiness and TaskStart. Worktree creation must still launch no Agent.
2. Find failure points before/after workspace existence, spawn, registration and
   safe readiness. Treat these as distinct observations, not a new universal enum.
3. Check re-entry against retained root/run/request evidence. An existing directory
   is not a preparation receipt and a live PID is not an initialized MCP client.
4. Reject late registration/binding from a replaced run. Never delete or overwrite
   a partially prepared user checkout to simplify recovery.

SP04: disposable checkout exists; inject launch/handshake failure, an exit before
registration and a delayed old registration. Re-enter the supported setup path.
Assert no duplicate live process, false readiness, checkout overwrite, wrong-root
adoption or automatic respawn. Inspect request and run identities at each boundary.

Completion: existing explicit launch paths recover truthfully; no new launcher,
provisioner, restart watchdog or directory-existence shortcut is added.

## 10. R2-04 — Existing queue backpressure and fairness (SP05)

Steps:

1. Inspect current Agent/task/inbox/database bounds, CONTROL_MESSAGE_RESERVE,
   per-assignee serialization, wake selection and result acknowledgement ordering.
2. Distinguish admission, retained notification, eligibility for safe native input
   and actual explicit handling. A reserved row does not prove handling latency.
3. Exercise independent recipients, offline receivers, noisy observations and
   duplicate/replayed inputs. A failing recipient must not monopolize shared work.
4. Preserve causal/per-request ordering and existing native input guards. Do not
   paste into busy/approval prompts or arbitrarily reorder a mailbox for speed.
5. If an existing queue path has a starvation bug, fix its actual owner with
   bounded work/coalescing only where semantics permit. Do not add a global slot
   scheduler or automatic launching. Lack of a launcher is not a skip reason.

SP05: saturate a disposable recipient, produce a finite replay/noise burst, keep
another healthy worker active and issue questions/answers/cancellation. Assert
correct admission/refusal, retained identities, eventual handling once native
input is eligible, continued healthy-peer progress and no duplicate execution.
Use barriers/fake clocks and finite deterministic scheduling; do not assert an
unmeasured fixed latency or promise control execution while the CLI cannot accept input.

Completion: existing queues remain bounded, truthful and live under the tested
conditions. A new project concurrency setting remains outside scope.

## 11. R2-05 — Failure classes and independent retry/health (SP06)

Steps:

1. Inventory errors the current protocol actually exposes: startup, handshake,
   native delivery, disconnected/unknown execution, capacity refusal and explicit
   task failure. Separate structured vendor metadata from generic terminal text.
2. Identify retry owner at each layer. Replaying an idempotent protocol request
   is different from rerunning a CLI turn or repeating a side-effecting task.
3. Preserve vendor-owned model retries. Do not interpret a terminal string such
   as "429" as authority to restart, change account/model or dispatch again.
4. Inspect health/error ownership and generation fences. One healthy process must
   not clear another run's issue; an old-run recovery cannot clear a new-run blocker.
5. Verify one broken worker does not stop healthy peers. Coalesce transient status
   noise without hiding a genuinely new human-action or unknown-execution issue.

SP06: keep worker A healthy while B disconnects or presents a fixture-defined
retry condition. Replace B's run, then deliver stale recovery/error events.
Assert A progresses, B's current issue stays accurate, no second task retry is
layered over CLI retry and no provider diagnosis is invented when unsupported.

Completion: fault domains and retry ownership are explicit; no new model event
integration, provider credentials, paid calls or whole-task replay is introduced.

## 12. R2-06 — Independent processes and shared writes (SP07)

Steps:

1. Inspect task/run checkout binding, native worktree identity, path normalization
   and advisory reservation acquire/conflict/expiry/release paths.
2. Verify two runs in the same checkout are not described as physically isolated;
   per-Agent serialization does not serialize another Agent's writes or Git index.
3. Reuse distinct registered worktrees for independent code tasks. For intentional
   shared-root/non-Git work, preserve explicit allowed-path instructions and the
   existing advisory reservation rules; do not present them as OS-enforced locks.
4. Audit failed/disconnected/unknown attempts before freeing reservations. Apply
   current lifetime semantics; do not infer stop solely from timeout or process name.

SP07: two disposable workers target overlapping paths in one checkout, then
distinct worktrees. Assert the supported reservation conflict behavior, truthful
advisory wording, current lifetime rules and successful independent-root behavior.
Include a non-Git project and canonical-path alias case where supported. Do not
prove safety by actually racing writes to a user's repository.

Completion: coordination and isolation claims match the actual guarantees;
no global workspace lock or silent process relocation is introduced.

## 13. R2-07 — Duplicate events and supervisor reconstruction (SP08, SP09)

Steps:

1. Trace event IDs, generation/revision fences, notification coalescing, pending
   questions/results and pagination. Identify replaceable observations separately
   from distinct requests, questions, results and control messages.
2. Verify an old delivery attempt cannot clear a replacement pending message.
   Repeated observations must not multiply task starts or erase actionable work.
3. Preserve explicit result processing: a result is not another instruction, and
   blocked stays outstanding. Do not create reply/acknowledgement loops.
4. Reconstruct the outstanding set from current durable tasks, inbox/history,
   questions and review state after an empty supervisor-local memory. Read all
   relevant pages, including offline assignees, within existing bounds.
5. Preserve current Coordinator selection/authority and immutable task provenance
   across handover. Reconstructing supervision grants no new execution ownership.
6. Use existing MCP bounded wait/ready. Do not copy Harness's internal tool guards
   into a terminal command parser or impose blanket implementation-round limits.

SP08: interleave duplicate old notifications, a replacement result and a new
question. Assert each real request is retained/handled once through its existing
semantics, no new task is created from a result and replaceable noise does not hide it.

SP09: create more outstanding tasks/requests than one page, including offline,
submitted, blocked and uncertain work; discard the supervisor's local set and
perform normal reconstruction or Coordinator handover. Assert all authorized
outstanding IDs are rediscovered without restarting work or expanding permissions.
Do not claim a fixture proves a real model survives context compaction.

Completion: supervision can recover from durable facts and duplicate events
converge without a new scheduler, agent memory service or arbitrary review counter.

## 14. R2-08 — Launch/configuration provenance and capability limits (SP10)

Steps:

1. Inspect existing bounded launch and attempt-binding metadata for executable,
   args, cwd, run, checkout and exact context refs. Do not log secret-bearing argv
   or environment values; preserve the project's current redaction rules.
2. Separate immutable attempt facts from current focus/defaults and capability
   observations. A changed default cannot retarget an existing attempt.
3. Preserve vendor-owned model/auth/global configuration. If exact model/prompt
   or vendor conversation is unavailable, record the limitation rather than invent it.
4. Derive named-Agent coverage from existing CLIAgent/adapter definitions. Do not
   create a duplicate allowlist or terminal-text fallback pretending to provide a
   structured acknowledgement, model status or conversation restoration.
5. Revalidate fresh run admission and existing remote host/account/root boundaries.
   Unsupported/absent/stale metadata must not become a successful restore claim.

SP10: bind an attempt, change focus/default Agent/configuration, replace the native
run and deliver a late old response. Assert immutable original provenance, no
retargeting, normal new admission and truthful unsupported-capability behavior.
Use disposable argv and metadata that cannot contain credentials.

Completion: execution provenance is inspectable within current capabilities;
no central model router, prompt editor or global CLI configuration write is introduced.

## 15. Verification environment and evidence rules

Read current workflow inputs, package names, test names and feature gates first.
Relevant existing CI entry points include:

```sh
cargo test -p warp-agent-bus --test native_pty --locked
cargo test -p warp-agent-bus --test coordination --locked
cargo test -p warp-agent-bus --test managed_agent --locked
cargo test -p warp-agent-bus --test readiness --locked
cargo test -p warp-agent-bus --locked
cargo test -p remote_protocol --locked
```

Run Rust commands in the environment prescribed by MEMORY, currently GitHub.
Use `.github/workflows/validate-agent-communication.yml` and
`.github/workflows/validate-remote-companion.yml`; inspect their actual conditions.
Preserve existing Linux artifact coverage where affected, without release publication.
If application changes occur, include applicable default/warp_platform checks and
native regression captures. If no UI changed, do not redo the full first-round
visual campaign merely because it existed; run affected checks only.

For every SP scenario record:

| Required field | Meaning |
| --- | --- |
| Source | Exact commit SHA or explicitly uncommitted source; baseline versus changed revision |
| Case | R2/SP ID, setup, injected failure, expected persisted/process/notification facts |
| Execution | Exact command or workflow inputs, OS, transport and enabled fixture/features |
| Result | Actual outcome, executed test count and relevant assertion |
| Evidence | Run/artifact URL or retained nonsecret receipt |
| Limits | Fixture versus live vendor, owned versus attached process, unsupported observation |

Verification procedure:

1. For a reproduced bug, obtain a focused failing test when feasible; fix the
   cause and re-run it. Then execute relevant adjacent first-round regressions.
2. Do not count `--no-run`, zero selected tests or ignored fixtures not enabled
   as executed passes. Helper child fixtures are not independent top-level gates.
3. Assert stored facts and side effects, not only status strings. Use disposable
   databases, repositories, roots and processes with bounded teardown. Never send
   a kill to an unverified PID/group or sweep unrelated processes during cleanup.
4. Cover applicable native macOS/Linux/Windows ownership and local/SSH/direct WSL
   paths through existing suites. Do not infer universal task-job ownership from
   one owned native PTY test. Document unsupported combinations accurately.
5. Preserve screenshots/static-first workflow only when R2 changes UI. If required
   rendering is unavailable, leave the exact gate pending and use static review
   as partial evidence, never as accepted visual fidelity.
6. Run relevant local script/document/JSON/YAML/locale checks and focused formatting;
   do not format unrelated source. `git diff --check` is not a Rust build.
7. Fix check failures and rerun the affected checks. Do not weaken assertions or
   disable a platform to get green. Broaden/repeat only when new changes justify it.
8. Do not publish source to a remote branch without applicable authorization just
   to obtain CI. Prepare the concrete diff and exact checks first; if a new push
   is required but unauthorized, ask specifically for that step. Continue other
   authorized work while the dependent CI gate remains pending.

## 16. Progress, stopping conditions and delivery

Update TODO's R2 section and append its classification/evidence rows as work
proceeds. Preserve first-round checkboxes and receipts. If an inherited checked
item is invalidated by a demonstrated R2 failure, annotate the specific dependency;
do not erase history or mark the whole previous round unfinished.

Close an R2 group only when applicable SP evidence passes. Existing correct code
may close with test evidence; source-only inspection stays unverified. Mark an
unsupported internal vendor capability as an explicit boundary with evidence,
not an excuse to skip host-level invariants. New optional mechanisms are excluded,
not implementation tasks or prerequisites for completing R2.

If blocked, name the exact missing permission/environment or material product
decision, the affected gate and the smallest next action. Finish independent
authorized work. Do not stop for ordinary implementation choices, an initial test
failure or a desire to ask whether to continue. Never repeat uncertain external
writes or destructive process operations to bypass a blocker.

Before final handoff:

1. Review the R2-only diff and its necessary inherited dependencies. Preserve
   prior edits, terminal guardrails, current transport semantics and exclusions.
2. Align changed TECH/API descriptions and tests with final behavior. Add only
   durable R2 decisions/pointers to MEMORY; keep full receipts in specs.
3. Leave a concrete reviewable diff. No merge, force-push, tag, version bump,
   release, installation or personal configuration edits without applicable authorization.
4. Report in Chinese: which R2 gaps were real, what was reused unchanged, what
   changed and why, exact verification passed, pending gates and MEMORY status.
   Distinguish "R2 complete" from final release acceptance if inherited unrelated
   gates are still pending. End with what was not checked and remaining material risk.

The intended outcome is a small second-round correction with stronger evidence,
not a new process framework or a second implementation of the first round.
