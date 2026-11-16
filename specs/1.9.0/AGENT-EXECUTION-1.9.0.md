# Agent execution prompt: Warpai 1.9.0 reliable supervised-work delivery

Date: 2026-11-08.
Purpose: a detailed execution assignment for an Agent explicitly asked to use
this file. Creating this prompt does not itself start implementation.

## 1. Assignment and success criteria

When the owner explicitly invokes this prompt for implementation, carry the
scoped work through source audit, specification alignment, minimal changes,
verification and a reviewable delivery. Do not stop at a proposal or ask whether
to continue when the next action is clear. Make routine implementation decisions
yourself within these boundaries. Do not interpret this prompt as authorization
to merge, publish, install, modify personal configuration or run paid vendor work.

Your objective is a reliable loop:

`dispatch → explicit execution → question/answer → submission → review → authorized integration or usable deliverable`

It must remain truthful through concurrent work, busy terminals, a new task
revision, Coordinator replacement, process restart, SSH disconnect and WSL
reconnection. Reuse working code. An audit can close an already-correct item with
evidence; a checkbox is not an instruction to rewrite its implementation.

The original historical 1.9.0 target list was unavailable. This file executes the
current scoped plan, not a claim about recovered past decisions. The authoritative
planning package is:

- [PRODUCT.md](specs/1.9.0/PRODUCT.md): behavior invariants BI-01 through BI-12.
- [TECH.md](specs/1.9.0/TECH.md): implementation boundaries and scenarios S01–S14.
- [TODO.md](specs/1.9.0/TODO.md): the single status/evidence ledger.
- [Research report](specs/SUPERAGENT-REVIEW-1.9.0.md): reference evidence only.

Follow current user instructions and applicable AGENTS.md first. Within this
package PRODUCT defines behavior, TECH defines implementation/verification,
TODO records status, and this file defines execution procedure. Correct stale
navigation references against current source. If a conflict changes user-visible
scope or authority, surface that concrete conflict; do not silently choose a
broader design.

## 2. Fixed scope and owner decisions

### Required direction

1. Keep native CLI Agents, local-first Agent Bus, existing SQLite transactions,
   tasks, attempts, questions, review, immutable context and handoffs.
2. Keep Worktree's selected Coordinator and Project's peer delegation model.
3. Improve supervision in the existing collaboration panel, with truthful facts
   and direct existing actions. Do not build a separate dashboard.
4. Correct shared behavior once at its owner; include callers, transports,
   fixtures, documentation and tests actually affected by that correction.
5. Make accepted-task continuation a separately authorized task with an exact
   handoff pin. Preserve existing failed/cancelled retry and review revision paths.
6. Respect existing storage capacity, protected references and archive/export rules.

### Excluded work

- No external event intake: no GitHub/Feishu listeners, webhook server, connector
  setup, source-account configuration or external event routing.
- The owner explicitly says danger-full-access needs no treatment. Do not add
  permission prompts, change vendor permission presets, build a sandbox or add
  access-mode remediation to the backlog. Keep existing credential protections
  and task/project/run/checkout authority checks; these are independent of access mode.
- No bundled AI/model runtime, cloud login/accounts/billing, telemetry, permanent
  sole Router, second task store, generic retry service or new daemon.
- No vector database, full terminal transcript ingestion, autonomous
  self-improvement, cross-host federation or automatic Agent launching.
- No automatic merge engine, prompt-only merge gate, default force-push,
  reviewer-credit history rewriting or broad terminal/UI refactor.
- No personal credentials, .env/private-key reads, personal CLI config writes,
  installed app changes, destructive Git cleanup or speculative dependencies.

Capacity is conditional, not a mandate to add features: inspect existing
automatic admission if present, fix a demonstrated in-scope gap, otherwise record
a justified skip. Do not create automatic dispatch to justify a new capacity UI.

## 3. Non-negotiable execution rules

- Communicate progress, decisions and final results in Chinese. Code, comments,
  test names, commits, PR text and technical documents use English; product copy
  follows the existing localization system.
- Use karpathy-guidelines for scope/assumptions/verification and ponytail for the
  smallest complete implementation. Read applicable skills, including
  warp-ui-guidelines before UI code. Use ui-design-handoff-fidelity when fixing
  the visual reference and validating the panel.
- Do not launch subagents unless the owner or applicable instructions explicitly
  authorize delegation. A/B/C below are test fixtures, not permission to launch
  paid or real vendor sessions.
- Search code through CodeGraph first only if the repo has `.codegraph/`; otherwise
  use `rg`. Read actual functions and every caller before editing shared logic.
- Batch independent read-only checks; keep dependent writes and validation ordered.
- Preserve unrelated dirty changes. Never reset, clean or force-push to simplify work.
- Avoid new types, wrappers, config, schema fields, dependencies and abstractions
  unless existing mechanisms demonstrably cannot express the requirement.
- Keep terminal core guardrails from README. Never replace core code with stubs
  to make builds pass. Retain WARP_* compatibility and native SSH/WSL semantics.
- No secret values in source, prompts, output, screenshots, fixtures or MEMORY.
  Use disposable accounts/configuration. Never copy user transcripts as fixtures.
- Provide a meaningful update about findings or remaining work at least once per
  minute during ongoing work; do not flood the user with individual tool logs.

## 4. Phase 0 — Establish the current baseline

Perform these steps before modifying application code:

1. Read root AGENTS.md (case-insensitive equivalent if applicable), README.md and
   MEMORY.md, then nested instructions for each directory you will touch. If a
   required root guide is missing, follow the project's template protocol before
   code changes. Do not overwrite an existing guide.
2. Read all four planning documents linked above, active
   `specs/agent-communication-v2/{API,PLAN,WORKTREE-COLLABORATION,HANDOFF}.md`,
   relevant release/verification guidance and referenced current acceptance notes.
   Follow supersession notes; archived enrollment designs are not requirements.
3. Record `git rev-parse HEAD`, branch, `git status --short` and worktree status in
   TODO. Distinguish owner edits from your own changes. If isolation is needed,
   use a new safe checkout/worktree without changing or deleting existing ones.
4. Inspect lockfiles, current crate names and workflows. MEMORY currently requires
   Rust builds/native acceptance on GitHub with caches, and local source/script
   checks only. Re-read that rule; do not start a local Rust build unless the owner
   has superseded it. Do not install or upgrade the toolchain as incidental cleanup.
5. Inspect existing CI status/artifacts read-only if available. Record baseline
   failures and their source SHA; do not attribute them to your work. Missing
   access is a specific validation limitation, not a reason to abandon source audit.

Output: a short baseline entry in TODO with source SHA, dirty paths, toolchain,
available validation paths and known blockers. Never store credential values.
Exit criterion: you know the current source/contract baseline and can preserve
existing work. Do not mark implementation behavior verified at this phase.

## 5. Phase 1 — Trace source and classify actual gaps

Inspect current implementations; paths below are a map, not a guaranteed patch list:

| Responsibility | Starting points |
| --- | --- |
| Operation/Task/Attempt/Evidence contracts | `crates/agent_bus/src/lib.rs` |
| Queue, replay, questions, lifecycle and recovery | `crates/agent_bus/src/storage.rs` |
| Mode/role/integration authority | `crates/agent_bus/src/storage_orchestration.rs`, `worktrees.rs` |
| MCP instruction/tool surface | `crates/agent_bus/src/mcp.rs` |
| Bootstrap, execution binding and reviewed memory | `project_memory.rs`, `project_memory_storage.rs` |
| Immutable context/handoff and sharing | `context.rs`, `context_storage.rs`, `handoff.rs`, `handoff_storage.rs` |
| Native wake and transport | `transport.rs`, `transport_orchestration.rs`, `session.rs`, `readiness.rs`, `native_pty*.rs`, screen detection modules |
| SSH/WSL bridges | `ssh_remote.rs`, `companion_tasks.rs`, related current application peer paths |
| Panel and controls | `app/src/agent_communication.rs`, `app/src/agent_communication/{panel,panel_controls,panel_handoff}.rs` |
| Fixtures and acceptance | `specs/agent-communication-v2/panel-fixtures.json`, `crates/agent_bus/tests/`, inline tests and existing native capture workflows |

For each BI invariant and TODO group:

1. Trace its real call path: MCP/controller → shared handler → storage and
   transaction → notifications/native delivery → projection → UI action.
2. Read success, validation, stale-response, authorization and error branches,
   not only type definitions or comments. Check local/SSH/WSL callers.
3. Inspect existing tests and assertions. Find whether they execute the relevant
   path, merely validate a string, or compile without running.
4. Enter one classification in TODO:
   - `existing_verified`: exact behavior has relevant passing evidence.
   - `existing_unverified`: implementation exists but proof is missing/stale.
   - `demonstrated_gap`: reproduce an incorrect/missing behavior with exact input
     and expected-versus-observed facts, preferably a failing focused test.
5. Record the smallest next action: reuse, run/add missing verification, or fix
   the shared owner. Do not preselect a rewrite or invent an absent subsystem.

Output: fill TODO's scope-classification table, including affected callers,
tests/fixtures/config/exports/contracts and specific user impact.
Exit criterion: every invariant has a classification and an actionable next step.

## 6. Phase 2 — Align the contracts before implementation

1. Update PRODUCT/TECH to reflect demonstrated gaps and the smallest chosen
   approach. Retain BI/S identifiers so evidence remains traceable.
2. Use existing Operation DTOs and state transitions first. Confirm actual
   signatures from source/tool schemas; do not invent arguments from examples.
3. For any necessary API change, update the active API contract before code:
   request/response shape, defaults/omission, validation, caller authority,
   request replay, stale revision/attempt/run handling, size bounds, errors,
   capability detection and older-client/service behavior.
4. For any necessary schema change, explain why current fields/events cannot
   express the behavior. Specify consistent backup, migration, restart, unknown
   execution, protected retention and mixed-version behavior. No automatic major
   version bump or Companion upgrade requirement without that evidence.
5. Define panel facts and count semantics before layout: one task row, multiple
   pending actions, visible scope, loaded-page versus whole-set totals, offline
   tasks, unsupported capability, observation age and action eligibility.
6. Lock visual reference using the existing native collaboration panel, fixtures,
   Appearance tokens, bundled assets and established controls. Record relevant
   source/capture identifiers, target widths and states. Do not imitate the
   Superagent web UI or invent a new visual system.

Output: coherent contracts and a concrete implementation order in TODO.
Exit criterion: behavior, ownership, UI reference and verification are specified.
Use established owner decisions for routine choices; ask only for a truly missing
product decision, necessary permission or real destructive action. If UI reference
inputs cannot be established, pause only visual implementation and continue
independent backend/fixture work.

## 7. Phase 3 — Delivery and explicit acknowledgement

Work through P0-01 and BI-01/BI-04 using existing shared operations:

1. Distinguish task assignment from message sending. `warp_task_assign` creates a
   tracked task; `warp_agent_send`, even with `expects_result`, does not.
2. Keep queue admission and native submission receipts separate. Preserve the
   original UUID, message/task association and ordering through busy delivery.
3. Use `warp_task_start` as the explicit task-start fact. It proves neither that
   every preceding message was read nor that a vendor model consumed PTY text.
4. Use `warp_agent_reply` for processing outcomes on result-bearing requests.
   Preserve atomic final-result/request acknowledgement; blocked remains open.
5. Use `warp_agent_answer` to resolve a structured question. Neither a plain
   message nor a generic acknowledgement may silently discard it.
6. Preserve task-notification transition guards and ordered result handling. Do
   not generalize all acknowledgement kinds into a single auto-ack mechanism.
7. Verify duplicates and same-UUID/changed-content behavior. Never mint a fresh
   UUID merely because a safe transport retry did not return promptly.

How to verify: S01/S03/S05 in TECH, asserting persisted messages, transitions,
receipts and notification counts. Demonstrate that busy PTY delivery leaves the
request pending, then the explicit operation creates only its supported fact.
Exit criterion: P0-01 has passing evidence or a specific remaining blocked gate.

## 8. Phase 4 — Follow-up ownership and continuous supervision

Work through P0-02/P0-03 and BI-02/BI-03/BI-06:

1. Reuse explicit task/thread/reply associations for in-scope follow-ups. Validate
   visibility and identity at the mutation boundary. Do not infer associations
   from names, terminal focus, prose formatting or semantic similarity.
2. Deliver to busy Agents through current guarded queues. Do not cancel their
   turn, insert into an approval prompt or promise next-step steering unsupported
   by the vendor. Preserve authorized native input/draft behavior from the active
   API contract; do not change local/WSL and SSH differences opportunistically.
3. Handle state explicitly:
   - Running task, same authorized scope: a linked follow-up remains that task's input.
   - Submitted task: inspect before review. If changes are needed, use existing
     review rejection/revision behavior; do not mutate the submitted result silently.
   - Accepted task, further requested work: create a new authorized task and pin
     the exact previous handoff; keep the original result/history intact.
   - Failed/cancelled task: use existing retry only when eligibility and unresolved
     attempt checks pass. A cancellation request is not a completed cancellation.
   - New scope: create separately authorized work or ask one necessary concrete
     question. A handoff/context reference itself grants no new execution scope.
4. Handle a revision that changes while a follow-up is queued: retain its original
   task/thread evidence, reject obsolete mutations and expose the unresolved
   association. Do not silently retarget instructions to the new revision/run.
5. Maintain supervision by task ID and result-bearing request ID. Refresh all
   pages and offline task owners where applicable; do not watch just one Agent.
6. Dispatch new authorized work whenever useful, including while other work runs.
   Handle questions and results across the whole set. Submitted work awaits review;
   blocked/cancel-requested/uncertain work remains outstanding.
7. When no immediate action remains and no work executes in the supervisor's own
   run, use existing ready/yield behavior. Preserve the supervision set for wakes.
   Do not introduce indefinite polling, sleep loops or another scheduler.
8. Preserve Project/Worktree mode authority and Coordinator handover. A restart
   with the same display name cannot inherit a role; focus changes cannot revoke it.

How to verify: S02/S04/S09 and relevant S07 cases. Use deterministic A/B/C fixtures:
A submits while B asks, dispatch C before B finishes, yield/resume the supervisor,
then drain every pending question/result and review. Assert IDs, associations,
permission checks and lack of duplicate tasks or result-reply loops.
Exit criterion: existing modes complete this interleaving without a new user nudge.

## 9. Phase 5 — Recovery, cancellation and stale callbacks

Work through P0-04 and BI-05/BI-11:

1. Locate the shared transaction and idempotency receipt boundary. Keep task,
   event and notification changes atomic; failed validation must roll everything back.
2. Exercise interruption at each meaningful boundary: queued dispatch, notification
   submission, explicit start, progress, result submission, review and cancellation.
   Include external work that may have succeeded before a reply was lost.
3. On restart/reconnect, read persisted task/attempt facts. Retain unknown or
   interrupted certainty. Absence of a window/socket/connection is not process exit.
4. Guard late requests by expected version/revision/attempt, exact owner run,
   project/checkout and confirmed remote context/generation as appropriate.
   Never let an old response clear or advance replacement work.
5. Preserve cancel-requested until the existing owning-run stop confirmation or
   guarded operator reconciliation occurs. Retry/reassign cannot bypass unresolved
   attempts; uncertainty overrides stay explicit with recorded reason/evidence.
6. Retry only known-safe protocol work with its original identity. Do not
   automatically repeat uncertain file saves, shell commands, commits or remote writes.
7. Verify queues and history survive disconnect. Reconnect must not cross SSH
   host/account/project or WSL distribution/user boundaries, and a pending remote
   context cannot fall back to a local checkout.

How to verify: S05–S08 with disposable DB/process/transport fixtures. Use barriers,
fake clocks or injectable failure points already present; avoid timing-only tests.
Check stored facts, retained evidence, old/new authority and notification effects.
Do not claim a real vendor recovered merely because a deterministic fixture did.
Exit criterion: no tested stale mutation or recovery path starts duplicate work.

## 10. Phase 6 — Delivery evidence and integration

Work through P0-05 and BI-07:

1. Trace submission/review/integration operations and evidence ownership. Preserve
   Agent-reported versus verified outcomes and producing attempt/revision bindings.
2. Require relevant code-delivery descriptors using existing Evidence fields:
   base/head/commit, checks and result. Do not fabricate a commit, successful check
   or clean worktree. For non-coding tasks use the actual artifact and its validation.
3. Prove review rejection/resubmission cannot reuse obsolete approval or delayed
   evidence. Keep current Project review authority and Worktree designated-review
   rules; do not impose a new universal reviewer policy.
4. Keep `warp_task_integrate` as accepted-result selection. Its
   `git_execution_implied=false` must never become a merge-success indication.
5. If authorized integration is part of the implementation task, the selected
   Coordinator inspects current checkout/branches, integrates reviewed content
   through existing Git commands, resolves only relevant conflicts, runs combined
   checks and records actual integration evidence. Pushing/merging externally
   still follows applicable authorization; do not use the prompt to expand it.
6. For UI work, provide real before/after evidence of the same scenario, size,
   theme and state. A mockup is not the baseline application. Refresh captures if
   subsequent changes invalidate them.

How to verify: S10 with disposable repositories and real commit IDs, plus a
non-Git artifact case. Show that selection alone leaves actual Git history unchanged.
Exit criterion: task acceptance and actual delivery cannot be confused in UI/reporting.

## 11. Phase 7 — Existing-panel attention UI

Work through P0-06 and BI-08/BI-09 only after the static reference is established:

1. Use existing native section/row/button controls, icon assets and unchanged
   button themes. Keep the current panel shell, spacing conventions and rounded
   container/focus behavior. Do not edit shared themes for this feature.
2. Define a derived pending-action collection per visible task. Reuse supervision
   facts; do not persist a competing task lifecycle enum. One task can require an
   answer, review or recovery simultaneously; preserve each fact.
3. Define the count/list query once at its existing owner. Filter authority before
   counting/pagination. A loaded-page count must be labeled as such, or obtain the
   bounded scoped total through the current projection. Never advertise a false total.
4. Prepare deterministic static fixtures for: empty, working, question, review,
   overlapping actions, dependency wait, input/permission observation, failed
   delivery, offline/unknown execution, unsupported service, stale/loading/error
   state and a list spanning multiple pages.
5. Capture/render static fixtures in normal/narrow widths and light/dark themes,
   with English, simplified Chinese and traditional Chinese. Use available native
   desktop runners, not a browser mock pretending to be the native application.
6. Inspect screenshots with image viewing tools. Check at least layout/density,
   typography/wrapping, colors/contrast, spacing/container shape, icon treatment
   and action accessibility. Fix overlap, truncation and hidden primary controls
   before live wiring. Record reference/capture/fix in TODO or its evidence link.
7. Wire the existing live projection and operations. Before a mutation, refresh
   and submit its required version/run facts; never reuse a stale fixture's values.
   Fence asynchronous responses by context, generation, task and exact context pin.
8. Retain drafts, focus, selected task and useful scroll position during refresh
   and failed actions. Show denied, unsupported, stale conflict and transport
   failure distinctly. Do not auto-resubmit an uncertain mutation.
9. Keep progress in history. Notify once for a new actionable fact, not each poll;
   a genuinely new question/recovery condition cannot be suppressed by coarse dedup.
10. Re-capture affected live states. Compare with accepted static reference and
    verify keyboard/focus behavior and actual existing action handlers.

How to verify: S11/S14 plus localization checks. Resolve visual defects before
calling UI complete. If rendering is genuinely unavailable, finish source/fixture
work, report the exact pending visual gate and do not describe the UI as accepted.
Exit criterion: static and live native evidence passes for applicable targets.

## 12. Phase 8 — Continuation, memory and retention

Work through P1-01 and BI-10:

1. Reuse compact project bootstrap and lazy exact-version context/handoff reads.
   Do not place full resource bodies or entire terminal history in every message.
2. Keep immutable brief content separate from current task/review/uncertainty.
   Refresh current source status before execution. No automatic repinning to a
   new head, silent acceptance or authority transfer from a shared brief.
3. Preserve not-provided versus not-applicable sections and missing/withdrawn/
   stale/denied/unsupported resource states. Do not invent a decision or next step
   from unstructured terminal output.
4. Reuse reviewed project memory. A proposed finding is not automatically trusted
   memory; follow the existing task review promotion rules.
5. Reuse existing archive/export/purge. Verify that active target tasks/messages
   pinning a source protect its provenance and that eligible unreferenced history
   can be reclaimed. Do not add an independent retention daemon.
6. Measure representative encoded bootstrap/detail pages against existing bounds.
   Test pagination/truncation/UTF-8 cases where changes affect them; do not claim
   token savings from byte counts alone.

How to verify: S12 with exact context refs, source revisions and positive/negative
sharing cases, then restart/cleanup. A replacement fixture should locate only the
necessary inputs and resume through normal authorized task operations.
Exit criterion: continuation is bounded and truthful with retained provenance.

## 13. Phase 9 — Conditional capacity decision

1. Inspect current per-Agent serialization, queue limits and automatic admission
   paths. Record which capacity is actually controlled and which vendor background
   work is not observable.
2. If no demonstrated in-scope gap exists, record `conditional_skip` for C1 with
   evidence and add no option, setting, scheduler or UI.
3. If a gap exists in an existing in-scope automatic path, fix reservation at its
   owner before asynchronous startup. Do not interrupt a running task to free a slot.
4. Keep answer/control/cancel delivery independent of worker admission capacity;
   preserve protocol ordering and existing native input fences. Do not paste a
   cancel instruction into a busy prompt to manufacture responsiveness.
5. Validate saturation and concurrent starts without claiming a host-wide process,
   CPU, memory or token budget. Do not copy Superagent's extra-slot constant blindly.

How to verify: S13 or a source/test-backed conditional skip.
Exit criterion: a clear, evidenced decision rather than speculative implementation.

## 14. Phase 10 — Verification and evidence collection

Use current workflows and the pinned toolchain. Verify all command names, test
filters, features, runner conditions and fixture counts against current source.
Run the smallest meaningful checks first; broaden when relevant changes justify it.

Current known entry points, to run in the project's prescribed CI environment:

```sh
cargo test -p warp-agent-bus --locked
cargo test -p remote_protocol --locked
cargo check -p warpai --bin warpai --locked
cargo check -p warpai --bin warpai --features warp_platform --locked
python3 script/check-interface-locales.py --strict
```

Existing workflows:

- `.github/workflows/validate-agent-communication.yml`: protocol suites,
  desktop checks, WSL acceptance and native captures through its current inputs.
- `.github/workflows/validate-remote-companion.yml`: shared protocol and
  Linux/macOS/Windows Companion/remote checks.
- Existing Linux artifact workflow when changed application code needs Linux
  build coverage. Do not run a publishing release workflow to obtain a test binary.

Rules for every run:

1. Use the exact implementation source, not a previously green unrelated SHA.
   Record command/workflow inputs, commit, platform, test count, outcome, URL and
   retained artifacts in TODO. Reused old evidence needs an explicit relevance check.
2. Assert focused tests actually ran. `--no-run`, zero selected tests and ignored
   tests not explicitly enabled are compilation/selection results, not pass evidence.
3. Preserve local/SSH/direct WSL gates and supported desktop targets. Inspect
   existing WSL feature flags and disposable Windows profile prerequisites.
4. Keep domain fixtures, native process tests, live vendor sessions, compilation
   and screenshot acceptance separate. Do not claim real vendor behavior from mocks.
5. For changed scripts run syntax checks matching their shebang; validate workflow
   YAML if changed. Check localized strings using the existing script. Run scoped
   formatting/lint and `git diff --check`; do not format unrelated source files.
6. When a check fails, identify the failing assertion/stage, fix the cause and
   re-run that check. Preserve evidence of baseline/environment failures. Do not
   weaken assertions, skip required tests or drop a target just to obtain green CI.
7. Do not modify the owner's installed app, credentials or personal configuration
   for acceptance. Use disposable fixture accounts/roots and existing capture paths.
8. Before remote source publication, determine whether branch push is authorized.
   If a new push is required but unauthorized, prepare the complete reviewable
   diff/contracts/check list first, then ask specifically for that action. Do not
   push because CI would be convenient. Existing authorized read/CI work may continue.

Output: source-matched verification receipts for all applicable S01–S14 and build,
transport, localization and native visual gates. Missing runtime/access is pending,
not success. Do not expand to unrelated whole-repository audits.

## 15. Ledger, stopping conditions and final handoff

Update TODO at each completed phase. For each item record classification, exact
change or reuse decision, evidence and remaining limitations. Keep only one status
ledger; this prompt is the procedure, not a duplicate progress log.

Use these completion rules:

- `existing_verified`: mark complete with evidence and no unnecessary code changes.
- `existing_unverified`: keep pending until the required checks actually pass.
- `demonstrated_gap`: close only after the focused failure is fixed and relevant
  regressions/transport/UI gates pass.
- `conditional_skip`: only for genuinely conditional scope, with a reason and
  source/test evidence. It cannot hide a required failed gate.
- `blocked`: name missing permission/environment or a material unresolved product
  conflict. Continue independent authorized work and make the blocked result reviewable.

Do not stop merely because implementation is difficult, routine design choices
exist, or one test failed. Stop the dependent action only for missing necessary
permissions/credentials, an unavailable required runtime, an actual unresolved
product decision, a genuine destructive risk or a hard applicable project rule.
Name the exact reason and complete all remaining independent work first.

Before final delivery:

1. Reconcile PRODUCT/TECH/API with actual behavior. Remove abandoned design text
   that would mislead future Agents. Keep scope exclusions explicit.
2. Review the complete diff for unrelated changes, accidental dependencies,
   secrets, permission changes, terminal-core regressions and missing callers.
3. Update MEMORY with durable decisions and a short pointer to this ledger;
   preserve user changes. Keep detailed receipts in specs, not MEMORY, and follow
   its size guidance without discarding unrelated owner information.
4. Leave a concrete reviewable implementation. Do not create a PR, push, merge,
   tag, change versions, publish releases or install packages without applicable
   authorization. If explicitly asked for a PR, use the repository create-pr skill.
5. Report in Chinese: what changed, why it matters, exact verification passed,
   remaining gates/risks and whether MEMORY was updated. End with what was not
   checked. Never claim the entire milestone complete while required gates remain.

The smallest complete diff wins. Existing working capabilities should become
better proven and easier to supervise, not be replaced to resemble another project.
