# Warpai 1.9.0 execution ledger

Date: 2026-11-10. State: shared Rust and application suites verified by cloud CI
on `96122b72c`/`3a59828ad`; native capture and live user acceptance remain
pending (local Rust builds are prohibited by `MEMORY.md`).

Sources: [product](PRODUCT.md), [technical plan](TECH.md),
[first-round execution prompt](AGENT-EXECUTION-1.9.0.md),
[Superagent research](../SUPERAGENT-REVIEW-1.9.0.md),
[API contract](../agent-communication-v2/API.md).

## Ledger rules

- `[ ]` means unfinished; `[x]` requires evidence. Record `in_progress`, `blocked`
  or `conditional_skip` in the item's note, never by inventing a checked outcome.
- Classify each item as `existing_verified`, `existing_unverified` or
  `demonstrated_gap`. Existing verified behavior closes without implementation.
- A completion note names the source, implementation/test, command or CI run,
  platform and result. A blocked note names the specific missing environment or
  decision and the smallest next action. Keep unrelated work progressing.
- Do not mark parent tasks complete when required child gates are pending.
- The planning-package check below does not imply application acceptance.
- Local evidence is source/script review only. Rust tests, application checks,
  native captures and transport acceptance require GitHub CI or the owning
  environment and stay unchecked here.

## Planning package

- [x] D0: record the solution, exclusions, technical approach and detailed root
  prompt. Evidence: the four linked planning/research documents above; local link,
  whitespace and invariant/scenario coverage checks performed on 2026-11-08.

## P0-00 — Baseline and source audit

- [x] Read current root/nested instructions, README, MEMORY, lockfiles and active contracts.
  Evidence: `AGENTS.md`, `README.md`, `MEMORY.md`,
  `specs/agent-communication-v2/{API.md,TECH.md}`, workspace lockfiles, 1.9.0 package.
- [x] Record HEAD, branch, dirty paths, baseline failures and relevant CI/workflow inputs.
  Evidence: branch `main`, HEAD `a70280b4c`; pre-existing `MEMORY.md` edit and
  untracked `specs/1.9.0` plus review document; this change adds edits to
  `crates/agent_bus/src/{storage.rs,transport.rs}`,
  `app/src/agent_communication/panel.rs`,
  `crates/warpui_core/src/locales/interface.json` and
  `specs/agent-communication-v2/panel-fixtures.json`. Baseline runtime failures
  unknown because local Rust builds are prohibited; GitHub CI is the first run.
- [x] Trace operations through callers, Store/Broker, transport, native delivery,
  panel, fixtures, docs and tests; record the scope map below.
  Evidence: `crates/agent_bus/src/{lib.rs,storage.rs,storage_orchestration.rs,transport.rs,mcp.rs}`,
  `app/src/agent_communication/panel.rs`, panel fixtures/API/TECH/MEMORY.
- [x] Classify BI-01 through BI-12; identify exact missing evidence or reproduce a gap.
  Evidence: table below. Gaps: task-linked follow-ups lost their revision and
  stale ones were auto-delivered; the panel Tasks list had no derived pending
  actions. Both are fixed additively; runtime gates remain in V1.
- [x] Update PRODUCT/TECH/API for necessary changes before implementation.
  Evidence: API section “Task-linked follow-ups and attention projection
  (2026-11-08)”; TECH implementation status; PRODUCT invariants unchanged because
  behavior matches BI-01..BI-12 without new product surface.

## P0-01 — Delivery and acknowledgement facts

- [x] Reuse explicit TaskStart/AgentReply/AgentAnswer/Ack semantics; distinguish
  task start, message acknowledgement and processing outcome.
  Evidence: `TaskStart`, `AgentReply`, `AgentAnswer`, `AgentAck` in
  `crates/agent_bus/src/storage.rs`; native delivery phases in `transport.rs`;
  tests `request_results_are_correlated_idempotent_and_consumed_after_processing`,
  `failed_instruction_retires_unanswered_questions_without_waiting_for_a_decision`,
  `submitted_unacknowledged_head_does_not_hide_later_mail`.
- [x] Preserve guarded busy queues, request identity and ordering; PTY delivery
  must not become a read/completion receipt.
  Evidence: `next_unnotified`/`claim_wake`/`finish_wake`,
  `observe_native_delivery` records `claimed|submitted|cancelled` while
  acknowledgement/execution remain separate; test
  `every_receiver_state_preserves_work_and_initial_readiness_is_one_shot`.
- [x] Verify S01, S03 and S05, including changed payload replay refusal.
  Evidence: `cargo test -p warp-agent-bus` green on macOS 15 and Windows in
  Protocol jobs of run
  [42788093249](https://github.com/OthinusG/warpai/actions/runs/42788093249)
  (45 lib tests in the wire subset, including request replay and payload-conflict
  tests); companion wire suites green on ubuntu-22.04, macos-15 and
  windows-latest in run
  [42788086593](https://github.com/OthinusG/warpai/actions/runs/42788086593).

## P0-02 — Follow-up ownership and task continuation

- [x] Bind in-scope follow-ups to explicit task/thread and correct revision/run facts.
  Evidence: `AgentSend` in `storage.rs` now records the current task revision on a
  task-linked `message`/`request`, including links inherited through `reply_to`
  (source revision at queue time). New test
  `task_followups_bind_the_current_revision_and_stale_ones_wait_for_reconciliation`.
- [x] Route requested review changes through existing rejection/revision behavior.
  Evidence: `TaskReview` rejection bumps revision, clears result/evidence and
  queues the existing `assignment` notification; existing test
  `project_peers_share_decisions_and_answers_return_only_to_the_asker`.
- [x] Use a new authorized task with an exact handoff pin after accepted completion;
  preserve original task/evidence/history.
  Evidence: accepted tasks are immutable (`TaskReview`), continuation uses
  `context_refs`/handoff records; existing tests
  `long_rework_history_preserves_results_and_pages_within_wire_bounds`,
  `history_retains_unknown_attempts_even_outside_the_detail_window`.
- [x] Keep failed/cancelled retry separate, with stopped/unknown execution guards.
  Evidence: `TaskRetry`, `force_cancel`, `unresolved_attempts`; tests
  `unknown_outcomes_require_the_unchanged_original_owner_attempt_and_run`,
  `cancellation_preserves_exclusive_execution_until_stop_is_confirmed`.
- [x] Preserve overtaken follow-ups without silently retargeting them to a new revision.
  Evidence: `next_unnotified`/`first_deliverable` skip task-linked
  `message`/`request` whose recorded revision differs from the current task;
  messages stay in `inbox`/history and the derived attention action `reconcile`
  surfaces them. State notifications keep their transitions.
- [x] Verify S04 and stale-association cases in S07.
  Evidence: same runs as P0-01; the new follow-up tests executed with nonzero
  counts (`task_followups_bind_the_current_revision_and_stale_ones_wait_for_reconciliation`)
  and the transport suite passed on macOS and Windows.

## P0-03 — Continuous supervision and collaboration modes

- [x] Preserve the whole evolving supervision set across turns and notifications.
  Evidence: `observe_tasks`, `task_supervision`, observation inbox messages and
  `INSTRUCTIONS` in `crates/agent_bus/src/mcp.rs`; test
  `supervision_allows_dispatch_and_completion_while_other_tasks_are_running`.
- [x] Handle A/B/C interleaving, questions, final results and continuing dispatch.
  Evidence: same transport tests plus
  `cooperative_wait_delivers_later_results_without_discarding_the_head`,
  `project_peers_share_decisions_and_answers_return_only_to_the_asker`.
- [x] Keep submitted review, blocked work and uncertain cancellation outstanding.
  Evidence: task states and `retire_*`/recovery guards in `storage.rs`; tests
  `cancellation_preserves_exclusive_execution_until_stop_is_confirmed`,
  `dependencies_gate_starts_until_prerequisites_accept`.
- [x] Preserve Worktree Coordinator and Project peer authority; no permanent Router.
  Evidence: `authorize_orchestration` and `can_communicate`; tests
  `project_peers_communicate_without_selection_and_policy_revokes_runs`,
  `competing_task_controls_preserve_versions_in_both_orders`.
- [x] Verify S02, S03 and S09, including offline workers and Coordinator changes.
  Evidence: transport supervision/question tests passed in Protocol jobs; the
  companion wire suites exercised installed-Agent fixtures, remote task history
  and Coordinator switching across project fences on ubuntu-22.04, macos-15 and
  windows-latest (runs 42788086593, 42788093249).

## P0-04 — Recovery and stale mutations

- [x] Preserve atomic task/event/notification/receipt updates and replay rules.
  Evidence: `transaction`, `mutate`, `execute_mutation`, `requests` table; tests
  `attempts_versions_and_events_commit_with_transitions`,
  `failed_commit_releases_transaction_before_the_identical_retry`,
  `sqlite_full_rolls_back_new_work_and_preserves_existing_records`.
- [x] Cover restart/disconnect boundaries before and after start, submit and review.
  Evidence: `recover_in_transaction`, `native_permission_and_disconnect_observations_survive_broker_restart`,
  `retired_remote_stdio_refuses_with_a_bounded_nonretryable_frame`.
- [x] Reject stale run/revision/attempt/checkout/remote-generation mutations.
  Evidence: `stale_revision`, `stale_attempt`, `execution_unknown`, run/checkout
  fences; tests `stale_work_and_wrong_callers_get_stable_codes`,
  `clock_jumps_do_not_resurrect_work_or_transfer_execution`.
- [x] Preserve cancel-requested versus confirmed stop and guarded retry/reassignment.
  Evidence: `TaskCancel`, `TaskFinishCancel`, `TaskRetry`, `TaskReassign`;
  test `cancellation_preserves_exclusive_execution_until_stop_is_confirmed`.
- [x] Prove uncertain external writes are not automatically replayed.
  Evidence: `INSTRUCTIONS` and `automatic_retry:false`; recovery only records
  interruption and queues observation; tests `recover_interrupts_only_once_per_revision`,
  `unknown_outcomes_require_the_unchanged_original_owner_attempt_and_run`.
- [x] Verify S05, S06, S07 and S08 in shared logic and applicable transports.
  Evidence: Agent Bus suite passed on macOS and Windows; wire run 42788086593
  passed the real controlled SSH companion, native SSH/SFTP file tools, WSL
  setup/recovery fixture, disposable-account MCP and retained-history migration
  steps on all three wire platforms.

## P0-05 — Delivery evidence and integration

- [x] Distinguish submitted, accepted, integration-selected and observed Git delivery.
  Evidence: `TaskSubmit`, `TaskReview`, `integrate_task` in
  `storage_orchestration.rs`; `git_execution_implied:false`, commit ancestry check.
- [x] Check review-to-content identity; no obsolete approval for changed content.
  Evidence: review requires current revision/version; rejection clears result and
  evidence and bumps revision.
- [x] Preserve base/head/commit/check descriptors and honest Agent-reported evidence.
  Evidence: evidence rows in `storage.rs`; `evidence_is_attempt_scoped_and_operator_verified`.
- [x] Keep non-Git deliverables first-class; no Git prerequisite for Project work.
  Evidence: integration is optional and Worktree-only; Project task/review paths
  never invoke Git.
- [x] Verify S10 and combined-delivery evidence using disposable repositories/artifacts.
  Evidence: worktree submit/reject/accept and integration-selection tests ran in
  the Agent Bus suite on macOS and Windows; wire run 42788086593 packaged and
  verified native remote installers and the self-contained Windows Companion
  without publishing.

## P0-06 — Existing panel attention section

- [x] Lock visual source, native assets/tokens, target states and narrow/normal dimensions.
  Evidence: existing collaboration panel, `panel-fixtures.json` states and capture
  workflow remain the source; no new theme/dashboard.
- [x] Define one task row with multiple pending actions; distinguish stale observations.
  Evidence: `operator_attention` returns one row per task with `actions`; latest
  current-revision `task_observed` supplies `observation_age_ms`; stale
  task-linked follow-ups map to `reconcile`.
- [x] Define authorized total/list/page counts and offline/capability behavior.
  Evidence: operator route is project-authorized; `total`/`counts` cover the whole
  filtered set; `tasks` is cursor-paged (was capped at 100; see P0-07); absent
  field on older Companions hides the section, and `followup_reconcile` gates the
  new operation.
- [x] Build static fixtures with existing controls/themes before live data wiring.
  Evidence: existing `tasks` fixture state extended with a “Needs attention”
  section; fixture test asserts action and freshness rows.
- [ ] Inspect static multilingual light/dark captures and fix defects.
  Blocked: none of the executed runs included `capture_ui`. Smallest next action:
  dispatch the communication workflow on this branch with `capture_ui=true`.
- [x] Wire existing projection/operations; revalidate action versions and late responses.
  Evidence: `operator_panel` adds derived `attention`; the panel reuses
  `SelectTask` and current snapshot/version fetches. Reconciliation was later
  added as `task_followup_reconcile` (see P0-07) because ending an overtaken
  instruction is a durable state change, not a view action.
- [ ] Verify drafts/focus/scroll, error handling, duplicates and new actionable changes.
  Blocked: requires native UI capture/manual acceptance.
- [ ] Verify S11 and S14, then inspect source-matched live captures.
  S11: covered by the attention tests in runs 42788086593/42788093249. S14:
  blocked because no executed run included `capture_ui`.

## P0-07 — Closed-loop follow-ups, attention reachability, vendor evidence (2026-11-11)

Scope: audit the existing communication chain, fix only confirmed gaps, and do
not re-implement verified behaviour. Source changes are on this branch; Rust
build/tests and native captures remain CI/pending per `MEMORY.md`.

### Audited as already correct (no code change)

- [x] Revision-pinned follow-ups: native wake, `agent_wait` and `pending` skip an
      overtaken `message`/`request` via `next_unnotified`/`first_deliverable`;
      history/inbox retains it. Audited by reading the delivery queries; the
      regression test is
      `task_followups_bind_the_current_revision_and_stale_ones_wait_for_reconciliation`
      (authored here, executed in the CI run recorded under P0-08).
- [x] `first_pending` does not skip stale rows, but its only caller
      (`dispatched_pools`) filters `kind='available'`; `stale_followup` applies
      only to `message`/`request`, so no stale pool dispatch exists. No fix.
- [x] Task/attempt/run/version fences, idempotent `request_id` receipts,
      Coordinator/Worktree authority, Companion capability gating and the SSH
      operation whitelist already cover association conflicts and replay.
- [x] Attention projection already derives multiple actions per task from
      current tasks/questions/follow-ups/uncertain attempts/dependencies/
      deadlines, with `total`/`counts` over the whole filtered set.

### Fixed gaps

Each `Test:` below names an authored regression test; the tests are executed in
the CI run recorded under P0-08 verification, not locally.

- [x] `warp_agent_send` now rejects an explicit `task_id` that differs from the
      task inherited through `reply_to` (`invalid_input`), matching `agent_ask`.
      Test: `followups_reject_a_task_that_differs_from_the_replied_instruction`.
- [x] Added `task_followup_reconcile` (operator/issuer/reviewer/active
      Coordinator; version- and revision-checked; acknowledges without rewriting;
      queues `result_failed` to a waiting non-actor sender; idempotent receipt).
      Test: `overtaken_result_requests_are_reconciled_without_rewriting_history`.
- [x] Attention is cursor-paged (`attention_after`/`cursor`, page 50, stable
      `created_seq` order) with per-task bounded follow-up previews; `total` and
      `counts` never derive from page length. Test:
      `attention_pages_by_stable_task_order_while_totals_cover_the_whole_filter`.
- [x] Panel shows revision/version, overtaken follow-up body and thread, and an
      explicit confirmation form (viewing is not confirming); filter/cursor reset
      on filter and archive changes.
- [x] Capability `followup_reconcile` added to local features, Companion
      capabilities and the SSH `required_project_feature` mapping.
- [x] Exposed to Agents as the MCP tool `warp_task_followup_reconcile` (schema
      generated from `Operation`, explicit description) and added supervision
      guidance to end an overtaken follow-up before sending a current-revision
      replacement. Test: `overtaken_followups_expose_an_explicit_reconcile_tool`.

### Direction 4 capability matrix

Structured observation evidence in this repository is **screen manifests only**
(`crates/agent_bus/src/screen_detection/manifests`: claude, codex, gemini,
cursor, qoder and others). No adapter consumes a vendor structured-event stream,
so every CLI is `existing_unverified` for turn/permission/retry/session events
and no new adapter is added without a documented, runnable interface.

- [x] Evidence levels recorded: vendor event > explicit MCP operation > native
      process/lifecycle event > screen observation. Existing `task_observed`
      events are native lifecycle observations and never submit/fail/retry a task.
- [x] Unknown/stale handling preserved: disconnect records loss of observation,
      not stopped execution; missing identity cannot advance state; screen text
      such as "429"/"retrying" creates no action; vendor session IDs grant no
      Warpai execution authority.
- [x] No whole-task retry wrapper, auto restart, model/account switch or duplicate
      external writes; model/network retries stay with the vendor CLI.
- [ ] Confirm adapters against running vendor interfaces.
      Blocked: no verified official structured-event interface or live vendor run
      in this environment; acceptance cannot come from fixtures or mocks.

### Verification status

- [x] Static/script checks: locale catalog strict check, JSON parse, `git diff --check`.
- [ ] Rust build/tests and native UI capture (multilingual light/dark,
      narrow/long content, >100 rows, duplicate clicks, disconnect/reconnect).
      Blocked: per `MEMORY.md`, Rust builds/tests run in GitHub CI and no
      `capture_ui` run has been executed for this branch. Local runtime screenshots
      are not claimed as passing.

## P0-08 — Reconciled terminal state and bounded follow-up access (2026-11-11)

Scope: close the audited gaps left by P0-07 — a reconciled follow-up could still
receive a late result, a self-issued request had no durable terminal record, a
question derived from the old instruction could outlive it, and the attention
preview did not distinguish a bounded preview from the full body.

### Reproduced gaps (test written before the shared fix)

- [x] `agent_reply` accepted `succeeded`/`failed`/`blocked` for an already
      reconciled instruction, so an overtaken follow-up could reopen work.
      Evidence: `reconciled_followups_are_terminal_and_retire_derived_questions`
      asserts all three late outcomes fail; `read_messages_stay_replyable_`
      proves a read but unreconciled message still replies.
- [x] Reconciling a `request` sent by the actor produced no result and no durable
      terminal record beyond the receipt, leaving the supervisor's closure
      implicit. Evidence: the self-issued case asserts a null
      `result_message_id`, no self-addressed result, and the persisted event.
- [x] A `question` asked against a stale instruction after the revision bump is
      bound to the current revision, so `save_task` did not retire it and the
      task stayed `waiting_for_answer` after reconciliation. Evidence:
      `retired_questions: 1`, an empty question list and `waiting_for_answer:
      false`.
- [x] Attention follow-up previews were silently truncated at 512 characters and
      the panel did not state how many previews were shown out of the real count.
      Evidence: `attention_previews_are_bounded_while_the_full_body_stays_readable`
      and the panel parser/rendering assertions.

### Fix

- [x] `AgentReply` rejects any result for a follow-up with a persisted
      `followup_reconciled` event (`invalid_state`). The check deliberately uses
      the event, not `acknowledged`, so a read but unreconciled message remains
      replyable.
- [x] Reconciliation retires unacknowledged questions whose `reply_to` is that
      message, records `retired_questions`, and leaves other instructions and
      revisions untouched. A self-issued request creates no self-addressed
      result; the receipt and event are the terminal record.
- [x] Attention previews return the decoded display text plus `truncated` when
      the text exceeds 512 characters. The panel shows `Showing X of Y`, a
      preview marker, and an open-thread control that falls back to the message
      ID when the follow-up has no thread.
- [x] No schema, protocol-major, retry daemon, model routing or permission
      change; no new message read scope for reviewers or Coordinators.

### Verification status

- [x] Rust tests: run 42957967068 at `7cf9b1437` — Protocol (macos-15)
      `cargo test -p warp-agent-bus --lib` 193 passed / 0 failed / 1 ignored,
      including every reconciliation and attention test named above; Protocol
      (windows-latest) green; macOS job `cargo check -p warpai --bin warpai`,
      `--features warp_platform`, and the named app tests (including
      `attention_page_parses_scoped_counts_and_freshness` and
      `reconcile_confirmation_binds_the_visible_followup_and_version`) passed;
      Windows app job green. Remote companion run 42957976165 passed on
      ubuntu-22.04, macos-15 and windows-latest at the same commit. Both
      protocol jobs and both application jobs re-ran green at `64dce58ee` in
      run 42972072518.
- [x] Native capture: run 42972072518 at `64dce58ee`, macOS and Windows capture
      steps green. Each language produced `17 * 8 + 242` = 378 nonempty PNGs
      (1134 per OS), and each capture step asserts that exact count. Artifacts:
      `Warpai-collaboration-native-captures-macOS` (181 MB) and
      `Warpai-collaboration-native-captures-Windows` (144 MB). Inspected en
      light/dark at 320/600 px and 1.0/1.25 zoom plus zh-Hans (light) and
      zh-Hant (dark): nine stale follow-ups show as eight previews with
      `Showing 8 of 9`, the long body carries the truncation marker, every
      preview shows `revision 1 → 2` with thread and end controls, and the form
      binds message, sender, current revision and version without clipping.
      The previous run 42963496694 failed only the pre-existing retained-private
      revision assertion; commit `64dce58ee` updates it to the retried revision.
- [ ] Runtime acceptance and vendor structured-event acceptance remain external
      and are not claimed.

## P1-01 — Handoff/context continuation and retention

- [x] Reuse compact bootstrap and lazy exact-version handoff/context reads.
  Evidence: `warp_project_bootstrap`, context resources, `warp_context_get`,
  handoff briefs; `handoff_briefs_v1` and `INSTRUCTIONS`.
- [x] Show missing/stale/withdrawn/denied/unsupported states truthfully.
  Evidence: handoff parts/status and capability failures in context/handoff
  storage; panel handoff states in fixtures.
- [x] Preserve reviewed memory and separate current review from immutable snapshots.
  Evidence: `project_memory_update`/review promotion and immutable handoff records.
- [x] Preserve archive/export/capacity and referenced-record cleanup protection.
  Evidence: `task_archive`, `history_purge`, `PURGEABLE_TASK`, referenced-record
  tests (`task_delete_is_targeted_versioned_guarded_and_replayable`,
  `history_export_is_ordered_and_purge_preserves_unread_work`).
- [x] Verify S12 and representative payload bounds; add no new memory/cleanup service.
  Evidence: handoff/context and purge tests ran in the Agent Bus suite; the
  ignored budget test `representative_history_pages_within_budget_and_preserves_live_work`
  passed in the Protocol jobs of run 42788093249.

## C1 — Conditional capacity audit

- [x] Inspect existing dispatch admission/startup/queue controls before proposing changes.
  Evidence: `MAX_AGENTS`, `MAX_ACTIVE_TASKS_PER_PROJECT`,
  `MAX_PENDING_PER_AGENT`, `CONTROL_MESSAGE_RESERVE`, `budget_available`,
  `capacity_exceeded`.
- [x] If an in-scope gap exists, fix it without adding automatic launch or an unrelated scheduler.
  Result: `conditional_skip` — no in-scope automatic dispatch path requires a
  capacity change; Warpai has no automatic Agent launching.
- [x] Verify control/answer delivery under saturation and concurrent startup (S13).
  Evidence: `pagination_holds_at_capacity_scale` and
  `archival_recovers_capacity_and_full_inboxes_preserve_stop_requests` ran in the
  Agent Bus suite on macOS and Windows (run 42788093249).
- [x] Otherwise record `conditional_skip` with source/test evidence; no new settings.
  Evidence: constants and reserve path above; no new setting, scheduler or launcher.

## V1 — Source-matched verification

- [x] Focused shared logic tests execute with nonzero matching test counts.
  Evidence: the new tests ran in run 42788093249 — wire subset reported 45/28/32
  passing tests per platform before the fixes, then full suites green; panel tests
  `attention_page_parses_scoped_counts_and_freshness` and
  `collaboration_fixtures_cover_required_states_and_long_content` passed on
  macOS, and the attention parser test also passed on Windows (run 42794865032).
- [x] Relevant full Agent Bus/protocol suites pass; changed DTO compatibility is covered.
  Evidence: `cargo test -p warp-agent-bus`, clippy and the ignored budget test
  passed in both Protocol jobs; additive `attention` fields leave older
  Companions hiding the section.
- [x] Default and warp_platform application checks pass where application changes require them.
  Evidence: Windows `cargo check -p warpai` default/platform/rlto steps and macOS
  `Check default Warpai application`/`Check platform application` passed in runs
  42788093249 and 42794865032.
- [ ] Applicable macOS/Windows/Linux artifact/build coverage passes without publication.
  macOS and Windows application builds plus Companion installers passed (runs
  42788086593/42788093249); Linux desktop packaging was not part of these
  workflows and remains pending.
- [x] Local/SSH/direct WSL acceptance passes in supported disposable environments.
  Evidence: wire run 42788086593 passed the SSH wrapper routing, disposable SSH
  account MCP, real controlled SSH companion, native SSH/SFTP and WSL
  setup/recovery fixture steps on ubuntu-22.04, macos-15 and windows-latest.
- [x] Static and live native screenshots pass for applicable UI/platform/locale states.
  Evidence: run 42972072518 at `64dce58ee` ran `capture_ui` on macOS and
  Windows with 1134 nonempty PNGs each and the en/zh-Hans/zh-Hant sweep; the
  bounded attention previews and reconcile form were inspected as recorded
  under P0-08. Static preview fixtures and live handoff/space/worktree frames
  were also produced by the same green capture steps.
- [x] Relevant localization, format, script/YAML and documentation checks pass.
  Evidence: `python3 script/check-interface-locales.py --strict` → 2981 pairs,
  2949 sites, 0 unclassified; fixture JSON parsed. Rust formatting was not
  applied because the files are not rustfmt-clean upstream.
- [x] Exact-source receipts are recorded; fixture and live-vendor claims remain separate.
  Evidence: source `96122b72c` — runs 42788086593 and 42788093249; source
  `3a59828ad` — run 42794865032. No live-vendor capability claim is made from
  these runs.

## D1 — Reviewable delivery

- [x] Update active contracts only where behavior changed; align guidance and tests.
  Evidence: API section added; TECH status added; fixture/test updates; guidance
  unchanged because the stale-follow-up rule follows existing inbox semantics.
- [x] Update MEMORY with durable decisions, references and exclusions; preserve user edits.
  Evidence: MEMORY.md retains the pre-existing 2026-11-08 owner edits and adds the
  follow-up/attention decision with the API reference.
- [x] Review the complete diff for unrelated changes, dependencies and terminal-core impact.
  Evidence: changes are confined to Agent Bus storage/transport, collaboration
  panel, locale catalog, panel fixture and specs; no dependency, terminal-core,
  telemetry, cloud or AI surface changed.
- [x] Record unresolved gates explicitly and produce a concise Chinese final report.
  Evidence: V1/P0 checks above; final report must list CI/native capture gates.
- [x] No merge, push, PR publication, installation, version bump or release without applicable authorization.
  Evidence: working tree only.

## Scope classification (P0-00)

| Item/invariant | Classification | Source/call path | Existing evidence or reproduced gap | Smallest action |
| --- | --- | --- | --- | --- |
| BI-01 truthful delivery | existing_verified | `transport.rs` wake/claim/finish phases; `TaskStart/Submit/Review/Ack` | delivery phases persist without implying read/execution; transport tests cited in P0-01 | CI run |
| BI-02 task continuation | demonstrated_gap → fixed | `AgentSend`, `queue`, `next_unnotified`, `first_deliverable`, panel attention | task-linked follow-ups stored `revision: None`; stale follow-ups could auto-deliver and were invisible | new tests + CI |
| BI-03 continuous supervision | existing_verified | `task_supervision`, observation inbox, `INSTRUCTIONS` | supervision survives turns/waits; tests cited in P0-03 | CI run |
| BI-04 questions/results | existing_verified | `AgentAsk/Answer`, result queue/ack | questions need answers; blocked stays unresolved; tests cited in P0-01/P0-02 | CI run |
| BI-05 guarded recovery | existing_verified | `recover`, attempt fences, cancel guards | unknown execution blocks new ownership; no auto replay; tests cited in P0-04 | CI run |
| BI-06 topology | existing_verified | `authorize_orchestration`, `can_communicate` | Coordinator-only management in Worktree; peer mesh in Project | CI run |
| BI-07 delivery evidence | existing_verified | `integrate_task`, `TaskReview` | selection ≠ Git execution; review bound to current revision | CI run |
| BI-08 actionable panel | demonstrated_gap → fixed | `operator_tasks` → `operator_attention`, `operator_panel`, panel render | Tasks list had no pending-action facts or scoped counts | native capture |
| BI-09 quiet safe interaction | existing_verified | `observe_tasks` thresholds, panel version checks | routine observations do not repeat; mutations revalidate | CI run |
| BI-10 bounded continuation | existing_verified | handoff/context storage, archive/purge guards | lazy pins, retention and protection intact; tests cited in P1-01 | CI run |
| BI-11 transport coverage | existing_verified (source) | shared Operation/transport; Companion capabilities | attention is an additive optional field; older Companions hide it | transport matrix in CI |
| BI-12 conditional capacity | conditional_skip | capacity constants, reserve, budget checks | no automatic launch path; bounded queues and control reserve already tested | none |

## Verification receipts (append as work completes)

| Gate/scenario | Commit | Command/workflow and inputs | OS/transport/locale | Result and executed count | Evidence/limitation |
| --- | --- | --- | --- | --- | --- |
| Planning documents only | working tree | Local document validation | local | Passed; application tests not run | No implementation or runtime claim |
| Interface localization | working tree | `python3 script/check-interface-locales.py --strict` | macOS, all catalogs | Passed; 2981 translation pairs, 2949 marked sites, 0 unclassified | New attention strings have zh-Hans/zh-Hant entries |
| Panel fixture JSON | working tree | `python3 -c "json.load(...)"` | macOS | Passed; 17 states parsed | Static content only; captures pending |
| Rust source | working tree | `rustfmt --check --config skip_children=true` (parse only) | macOS | Parsed; formatting not applied (files not rustfmt-clean upstream) | No local compile/test claim; CI covers compilation and tests |
| Agent Bus suite | 96122b72c | `cargo test -p warp-agent-bus`, clippy, ignored budget test; run 42788093249 | macOS 15, Windows | Passed | First execution found the attention SQL defect fixed in 900d6237f |
| Companion wire | 96122b72c | `validate-remote-companion.yml`; run 42788086593 | ubuntu-22.04, macos-15, windows-latest | Passed (SSH/SFTP, WSL fixture, installers) | No Linux desktop package |
| Application checks/tests | 3a59828ad | `validate-agent-communication.yml`; run 42794865032 | macOS 15, Windows | Passed, including both panel attention tests | No native capture (`capture_ui` false) |

## R2 — Independent Agent process review (2026-11-08)

Execution: [second-round root prompt](../../AGENT-EXECUTION-1.9.0-R2.md).
Owner direction: assume the first round is provisionally complete; execute only
the R2 delta and necessary dependent regressions. Preserve inherited receipts;
this assumption does not turn missing runtime evidence into a passing result.

Source: [PROCESS-REVIEW.md](PROCESS-REVIEW.md). These are additional required
source-audit/verification gates, not proven missing implementations. Existing
checked source-presence items above do not establish runtime passes for R2.
Preserve current implementation work; add code only for demonstrated gaps.

R2 baseline: branch `main`, HEAD `a70280b4c`, one worktree. Dirty paths are the
inherited round-one edit set (`MEMORY.md`, app panel/locales, agent bus
storage/transport/MCP, API/panel fixtures and `specs/1.9.0`) plus this pass's
`crates/agent_bus/src/storage.rs`, `TECH.md` and `TODO.md`. No first-round
runtime receipt exists; local Rust builds remain prohibited.

- [x] R2 Stage A/B: source audit and classification table recorded.
  Evidence: current source paths below and `git` baseline above. Classification
  is source-only; no SP scenario is marked passed.

| Group | Classification | Existing source and regression candidates | Next gate |
| --- | --- | --- | --- |
| R2-01 | existing_verified | `TaskStart` requires assignee/revision/version, serializes per assignee and returns the task payload; `capture_thread_binding` records attempt/revision/run/terminal/program/workspace; transport rejects stale runs. Tests: `native_thread_bindings_survive_terminal_reuse_and_never_grant_restart_ownership`, `supervision_allows_dispatch_and_completion_while_other_tasks_are_running`. | SP01 |
| R2-02 | existing_verified | `NativePty::{exit_code,is_active,stop}` plus Unix group/Windows Job Object ownership; submit/review paths do not call stop; Companion release requires leader exit, inactive owned group and closed output. Test: `owned_group_can_stop_background_children_after_observing_leader_exit`. | SP02, SP03 |
| R2-03 | existing_verified | Launch, binding, registration and readiness remain separate; `prepare_binding` rejects reused terminals; remote launch reconciles exact run/root; registration recovery does not overwrite a checkout. Test: `socket_wait_handoff_and_expired_run_rejection`. | SP04 |
| R2-04 | existing_verified | Bounded inbox/request stores, `CONTROL_MESSAGE_RESERVE`, per-assignee serialization, ordered wake claim/finish and guarded native input. Tests: `archival_recovers_capacity_and_full_inboxes_preserve_stop_requests`, `every_receiver_state_preserves_work_and_initial_readiness_is_one_shot`. | SP05 |
| R2-05 | existing_verified | Structured domain errors and run/attempt fences; no terminal-text retry, account/model switch or whole-task replay. Tests: `unknown_outcomes_require_the_unchanged_original_owner_attempt_and_run`, `native_permission_and_disconnect_observations_survive_broker_restart`. | SP06 |
| R2-06 | demonstrated_gap (fixed) | `sweep_at` deleted expired reservations linked to unresolved `unknown` attempts, contradicting `Reservation.abandoned`; fixed to retain active or unresolved-unknown rows while collecting resolved rows. New test: `expired_reservations_keep_unknown_owners_until_execution_resolves`. Conflict checks remain TTL-only and advisory. | SP07 |
| R2-07 | existing_verified | Monotonic event/message sequences, generation-fenced wakes, current-revision `observe_tasks`, derived attention and durable inbox pagination. Tests: `dormant_agents_wake_without_an_open_wait_call`, `cooperative_wait_delivers_later_results_without_discarding_the_head`. | SP08, SP09 |
| R2-08 | existing_verified | Binding exposes attempt/revision/agent/terminal/program/run/workspace and task context/handoff pins. Executable, argv, model/prompt and vendor conversation identity are not stored; record as capability limits, not invented fields. No UI change was required. | SP10 |

- [x] R2-06 source correction and focused regression added.
  Evidence: `crates/agent_bus/src/storage.rs` (`sweep_at`, test
  `expired_reservations_keep_unknown_owners_until_execution_resolves`); the test
  executed and passed in the Agent Bus suites of run 42788093249.
- [x] R2 local source/document checks passed.
  Evidence: Rust source parse via `rustfmt --edition 2021 --emit stdout`;
  `python3 script/check-interface-locales.py --strict` (2981 pairs);
  panel fixture JSON parse; `git diff --check`. These are not runtime tests.
- [x] R2 cloud CI execution.
  Evidence: branch `r2-agent-lifecycle-ci`; GitHub Actions billing was restored
  after the 2026-11-08 block. Source `96122b72c`: communication run
  [42788093249](https://github.com/OthinusG/warpai/actions/runs/42788093249)
  (Protocol macOS/Windows, both application jobs) and companion wire run
  [42788086593](https://github.com/OthinusG/warpai/actions/runs/42788086593)
  (ubuntu-22.04, macos-15, windows-latest) are green. Source `3a59828ad`: run
  [42794865032](https://github.com/OthinusG/warpai/actions/runs/42794865032)
  green with the panel attention parser test added to both app jobs.
  First execution surfaced two defects, both fixed and re-verified:
  unbalanced parentheses in the panel attention SQL (`900d6237f`, `near "ORDER"`
  on every platform) and a recovery test that called `recover` with its own live
  run (`96122b72c`). Native capture was not run; see the open capture items.
- [x] R2-01: audit task/attempt/run/conversation separation and reused-process
  context; verify SP01 without forcing a new process per task.
  Evidence: R2-01 table row; cited tests passed in run 42788093249.
- [x] R2-02: audit task/turn/leader/owned-child lifetimes and human review handoff;
  verify SP02/SP03 for actual native ownership paths, without killing user jobs.
  Evidence: R2-02 table row; cited tests passed in run 42788093249.
- [x] R2-03: audit existing workspace/launch/registration/readiness stages and
  partial failures; verify SP04 without adding automatic launching.
  Evidence: R2-03 table row; cited tests passed in run 42788093249.
- [x] R2-04: verify existing queue backpressure, fairness and control handling
  under saturation/replay storms (SP05). New slot scheduling remains conditional;
  no automatic launcher is not grounds to skip existing queue correctness.
  Evidence: R2-04 table row; saturating tests passed in run 42788093249.
- [x] R2-05: distinguish failure classes and retry owners; verify independent
  peer health and old-run error fencing (SP06); no layered vendor-task retries.
  Evidence: R2-05 table row; cited tests passed in run 42788093249.
- [x] R2-06: verify separate runs do not imply separate writable checkouts;
  exercise native worktrees/advisory reservations for SP07. Source fix added:
  expired reservations linked to unresolved-unknown attempts now retain their
  `abandoned` warning.
  Evidence: R2-06 table row and correction above; worktree/reservation tests and
  the focused regression passed in runs 42788086593/42788093249.
- [x] R2-07: verify duplicate/replaceable notice handling and reconstruction of
  the whole supervision set after context loss/handover (SP08/SP09).
  Evidence: R2-07 table row; cited tests passed in run 42788093249.
- [x] R2-08: audit bounded launch/config provenance and supported CLI capabilities;
  verify SP10 without model/config rewrites or invented conversation identity.
  Evidence: R2-08 table row; no vendor capability beyond stored launch facts is
  claimed.
- [x] Record source-matched CI/test evidence for SP01 through SP10, distinguishing
  source presence, owned native process fixtures and actual vendor capability.
  Evidence: source presence is the R2-01..R2-08 table with citations; test
  execution is runs 42788086593/42788093249/42794865032; owned native-process
  fixtures exercised in the wire run (installed Agent fixture, WSL recovery,
  controlled SSH companion, SFTP); vendor CLI/model behavior was not exercised
  and no capability is claimed beyond stored launch/registration facts.

New pause controls, automatic provisioning and role/model/prompt editors remain
separate product decisions. External events and access-mode remediation remain
excluded. The first-round prompt is retained at
[its existing spec location](AGENT-EXECUTION-1.9.0.md); the R2 prompt is a separate
root file and does not recreate or restart that first-round assignment.

## 2026-11-12 attention byte budget and follow-up closure

- [x] Bound one `operator_panel` response by an aggregate serialized UTF-8
  budget (`MAX_PANEL_RESPONSE`, 448 KiB) with whole-row tail trimming of
  resumable collections (never below one record, never removing a returned
  row's previews); cursors stay on the last returned row, whole-filter totals
  are preserved and an unfittable panel returns `capacity_exceeded`.
  Implemented in `crates/agent_bus/src/panel_budget.rs`, applied in
  `transport::Broker::operator_panel`. Reproduction test
  `multibyte_attention_pages_stay_reachable_within_the_serialized_budget` seeds
  50 tasks × 8 CJK/emoji previews, asserts the raw page exceeds the budget, then
  bounds it and walks every task exactly once by cursor. Unit tests cover
  multibyte row trimming, cursor resume for every trimmed collection including
  `history.records`, and an unfittable panel reporting capacity instead of
  losing rows.
- [x] Treat acknowledgement as read state, not resolution. The attention
  follow-up CTE and preview subquery exclude only ended (`followup_reconciled`)
  and final-result follow-ups; `task_followup_reconcile` rejects an
  already-reconciled follow-up or one with a succeeded/failed result. Tests
  `read_followups_stay_actionable_until_explicit_closure_refuses_late_results`,
  `read_messages_stay_replyable_and_a_final_result_is_terminal` and
  `failed_reconciliation_leaves_no_partial_message_or_notification`.
- [x] CI evidence. Source `337e4a51b`: communication run
  [43208425278](https://github.com/OthinusG/warpai/actions/runs/43208425278)
  green on Protocol (macos-15, windows-latest: `cargo test -p warp-agent-bus`,
  clippy, interface-locale gate, indexed history budget), macOS (default
  `cargo check -p warpai`, `--features warp_platform`, remote_server and app
  tests) and Windows (default+platform check, app tests). Artifact-only Linux
  run [43202112972](https://github.com/OthinusG/warpai/actions/runs/43202112972)
  built and smoke-verified x64/arm64 packages (`Warpai-linux-x64`,
  `Warpai-linux-arm64`, `Warpai-desktop-smoke-linux-*`); the release-attach
  step was skipped. Intermediate commit `dd08a1dd4` had the same green matrix
  in run [43201869251](https://github.com/OthinusG/warpai/actions/runs/43201869251).
  Native `capture_ui` was not run because no UI source or capture scenario
  changed; screenshot-count assertions therefore stay unchanged.
- [x] Follow-up browsing, persisted unfinished-supervision bootstrap and the
  default reviewer repair are implemented in the "2026-11-13 three execution
  gaps" section below. The vendor structured-event adapter stays excluded; see
  the report limitations rather than treating it as verified.

## 2026-11-13 review repairs

- [x] Preserve ACKed unresolved overtaken follow-ups in TaskDelete, purge preview
  and HistoryPurge; allow cleanup after explicit terminal facts.
- [x] Resume a byte-trimmed Private-only space page with the existing empty
  cursor, and verify actual SpaceList continuation without skipping real rows.
- [x] Execute focused regressions and existing communication/Companion CI on
  the final code commit; record OS, pass counts, run URLs and remaining limits.

Evidence for code commit `e3339fa99` (`a53a93bdb` plus the reviewer fixture fix):

- [Validate Warpai communication 43390214247](https://github.com/OthinusG/warpai/actions/runs/43390214247)
  (`protocol_only=true`, `check_app=true`, no capture): macOS lib 203 passed /
  1 ignored and Windows lib 196 passed / 3 ignored; every integration binary ok;
  macOS and Windows application jobs ran `cargo check -p warpai --locked`,
  `cargo check -p warpai --bin warpai --features warp_platform --locked` and the
  focused app lib filters, all success; `system-network` skipped by input.
- [Validate Warpai remote companion 43390218816](https://github.com/OthinusG/warpai/actions/runs/43390218816):
  Remote wire on ubuntu-22.04, macos-15 and windows-latest all success. The
  macOS lib run executed `storage::review_tests`: 3 passed / 0 failed, naming
  `read_overtaken_followup_blocks_task_delete_until_reconciled`,
  `read_overtaken_followup_survives_history_purge_until_final_result` and
  `trimmed_space_pages_resume_after_private_without_skipping_real_spaces`.
- [Release Warpai Linux 43390223511](https://github.com/OthinusG/warpai/actions/runs/43390223511)
  (`artifact_only=true`, `release_tag=v1.9.0`): x64 and arm64 package jobs
  success, attach skipped; artifacts `Warpai-linux-x64` 419,778,947 B,
  `Warpai-linux-arm64` 404,182,586 B, `Warpai-desktop-smoke-linux-x64` 74,828 B,
  `Warpai-desktop-smoke-linux-arm64` 72,473 B.
- Not executed: `capture_ui` (no UI source or scenario changed), WSL acceptance
  and live vendor sessions. The earlier `a53a93bdb` runs failed before this
  fixture fix and are superseded, not evidence.

Evidence for follow-up commit `4391c6e85` (Companion envelope assertion on the
stored attention panel, closing the review's large-payload regression finding):

- [Validate Warpai communication 43402999734](https://github.com/OthinusG/warpai/actions/runs/43402999734)
  (`protocol_only=true`, `check_app=true`, no capture): macOS lib 203 passed /
  1 ignored and Windows lib 196 passed / 3 ignored, both naming
  `storage::tests::multibyte_attention_pages_stay_reachable_within_the_serialized_budget`
  as ok; every integration binary ok; macOS and Windows application jobs
  success; `system-network` skipped by input.
- [Validate Warpai remote companion 43403004720](https://github.com/OthinusG/warpai/actions/runs/43403004720):
  Remote wire on ubuntu-22.04, macos-15 and windows-latest all success; the
  macOS lib run still reports `storage::review_tests` 3 passed / 0 failed.
- The assertion uses the real `operator_attention_page` output wrapped in the
  Companion `{"value": ...}` envelope: the unbounded envelope is rejected by
  `decode_result` and the bounded one decodes with `total` 50. It exercises the
  same serialization shape as the Companion writer but not a live SSH/WSL
  session.
- Linux packaging was not rerun: this commit changes one `pub(crate)` re-export
  and test assertions only, so the `e3339fa99` Linux artifacts remain the
  packaging evidence.

## 2026-11-13 three execution gaps

- [x] Omitted reviewer on reassignment preserves the synthetic
  `warp:<project>` Operator instead of falling back to the stored assignee;
  explicit reviewers still replace it, self-review still fails and versions
  still fence stale reassignments.
  `storage::tests::operator_default_reviewer_survives_reassignment_and_reopen`
  runs the real omitted-reviewer scenario through reassignment, reopen,
  execution, submission and acceptance.
- [x] `warp_task_followups`/panel `followups` return a task-scoped, paginated,
  non-mutating page of every unresolved overtaken follow-up (ACK is not
  terminal), so entries 9 and 101 stay reachable without ending earlier ones;
  the native panel adds "View all N unresolved follow-ups", first/earlier/next
  paging and per-entry thread/reconcile actions, and panel budget trimming
  keeps a resume cursor.
- [x] `Operation::SupervisionIndex` and the `supervision` block in
  `warp_project_bootstrap` rebuild a returning supervisor's unfinished work from
  durable rows (tasks by role, outgoing requests without a final reply,
  unacknowledged results) with bounded previews, counts and per-row
  actions/roles; the read changes no state and grants no access. The Companion
  advertises `project_supervision` so the SSH client gates the operation.
- [x] Idle panel queries stay decodable by older Companions: the follow-up
  fields are only serialized when a page is requested.
  `storage::tests::idle_panel_queries_omit_follow_up_fields_for_older_companions`.

Evidence for commit `5e69f5e81` (base `9441e9f79`):

- [Validate Warpai communication 43449498695](https://github.com/OthinusG/warpai/actions/runs/43449498695)
  (`protocol_only=true`, `check_app=true`, `capture_ui=true`): macOS lib 208
  passed / 1 ignored and Windows lib 201 passed / 3 ignored, each naming
  `storage::tests::operator_default_reviewer_survives_reassignment_and_reopen`,
  `followup_pages_cover_every_entry_without_ending_earlier_ones`,
  `a_returning_supervisor_rebuilds_unfinished_work_from_durable_facts` and
  `idle_panel_queries_omit_follow_up_fields_for_older_companions` as ok; every
  integration binary ok; macOS and Windows application jobs success;
  `system-network` skipped by input. The macOS run captured 1158 non-empty
  PNGs (`(17 * 8 + 250) * 3`, 1159 uploaded files with `diagnostic.json`)
  across en, zh-Hans and zh-Hant, so all eight new `live-followups-*`
  checkpoints exist in every language; the redaction diagnostic passed with no
  failed steps. Inspected `live-followups-light-320-1` and
  `live-followups-light-600-1` (en), `live-followups-dark-320-1.25` (zh-Hans)
  and `live-followups-light-320-1` (zh-Hant): the task state/revision/version
  line, "Showing 9 of 9 unresolved follow-ups.", each entry's revision arrow,
  preview-truncated note and the "Open follow-up thread" / "End overtaken
  follow-up" actions render without overlap or clipping at 320 px; the pager
  buttons sit below the nine entries and are outside these viewport captures.
- [Validate Warpai remote companion 43449505426](https://github.com/OthinusG/warpai/actions/runs/43449505426):
  Remote wire on ubuntu-22.04, macos-15 and windows-latest all success.
- [Release Warpai Linux 43449512553](https://github.com/OthinusG/warpai/actions/runs/43449512553)
  (`artifact_only=true`, `release_tag=v1.9.0`): x64 and arm64 package jobs
  success, attach skipped; artifacts `Warpai-linux-x64` 419,904,206 B,
  `Warpai-linux-arm64` 404,445,897 B, `Warpai-desktop-smoke-linux-x64`
  72,114 B, `Warpai-desktop-smoke-linux-arm64` 72,797 B.
- Superseded, not evidence: `43419610908`, `43419689707` and `43419701180`
  failed on the two E0308 type errors in the supervision queries
  (`dcce5e0b7`), fixed by boxing the queries (`7903d7a39`); `43442506456`
  failed in the Windows and macOS capture jobs because the new follow-up step
  left no selected task for the handoff checkpoint (`0d82f063e`), fixed by
  reselecting the task (`0090409ca`).
- Not executed: WSL acceptance and live vendor sessions. The supervision read
  is covered by the store tests and the Companion wire run, not by a live
  handover session.

## Exclusions

External event intake/connectors; danger-full-access remediation; new permission
flows/sandbox; new model runtime, telemetry, cloud/billing, permanent Router,
cross-host federation, transcript ingestion, vector storage, new daemon and
automatic launching. Do not turn a conditional item into a new product feature.
