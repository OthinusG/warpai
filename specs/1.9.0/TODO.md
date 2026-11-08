# Warpai 1.9.0 execution ledger

Date: 2026-11-08. State: implementation in progress in the working tree; runtime
verification pending CI (local Rust builds are prohibited by `MEMORY.md`).

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
- [ ] Verify S01, S03 and S05, including changed payload replay refusal.
  Blocked: requires CI execution; reuse `cargo test -p warp-agent-bus` and
  `cargo test -p warpai` CI jobs. Smallest next action: run the GitHub workflows.

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
- [ ] Verify S04 and stale-association cases in S07.
  Blocked: requires CI execution of the new focused tests plus the transport suite.

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
- [ ] Verify S02, S03 and S09, including offline workers and Coordinator changes.
  Blocked: requires CI and native multi-agent acceptance.

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
- [ ] Verify S05, S06, S07 and S08 in shared logic and applicable transports.
  Blocked: requires CI plus SSH/WSL acceptance.

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
- [ ] Verify S10 and combined-delivery evidence using disposable repositories/artifacts.
  Blocked: requires CI/native disposable repositories.

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
  filtered set; `tasks` is capped at 100 with `truncated`; absent field on older
  Companions hides the section (no capability/version bump).
- [x] Build static fixtures with existing controls/themes before live data wiring.
  Evidence: existing `tasks` fixture state extended with a “Needs attention”
  section; fixture test asserts action and freshness rows.
- [ ] Inspect static multilingual light/dark captures and fix defects.
  Blocked: native capture requires a local Rust build, prohibited by `MEMORY.md`.
  Smallest next action: run the collaboration capture workflow on GitHub.
- [x] Wire existing projection/operations; revalidate action versions and late responses.
  Evidence: `operator_panel` adds derived `attention`; the panel reuses
  `SelectTask` and current snapshot/version fetches; no new mutation operation.
- [ ] Verify drafts/focus/scroll, error handling, duplicates and new actionable changes.
  Blocked: requires native UI capture/manual acceptance.
- [ ] Verify S11 and S14, then inspect source-matched live captures.
  Blocked: requires CI/native live capture.

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
- [ ] Verify S12 and representative payload bounds; add no new memory/cleanup service.
  Blocked: requires CI execution; no new service was added.

## C1 — Conditional capacity audit

- [x] Inspect existing dispatch admission/startup/queue controls before proposing changes.
  Evidence: `MAX_AGENTS`, `MAX_ACTIVE_TASKS_PER_PROJECT`,
  `MAX_PENDING_PER_AGENT`, `CONTROL_MESSAGE_RESERVE`, `budget_available`,
  `capacity_exceeded`.
- [x] If an in-scope gap exists, fix it without adding automatic launch or an unrelated scheduler.
  Result: `conditional_skip` — no in-scope automatic dispatch path requires a
  capacity change; Warpai has no automatic Agent launching.
- [ ] Verify control/answer delivery under saturation and concurrent startup (S13).
  Blocked: requires CI execution of
  `pagination_holds_at_capacity_scale`,
  `archival_recovers_capacity_and_full_inboxes_preserve_stop_requests`.
- [x] Otherwise record `conditional_skip` with source/test evidence; no new settings.
  Evidence: constants and reserve path above; no new setting, scheduler or launcher.

## V1 — Source-matched verification

- [ ] Focused shared logic tests execute with nonzero matching test counts.
  Pending: new tests
  `task_followups_bind_the_current_revision_and_stale_ones_wait_for_reconciliation`,
  `attention_derives_questions_review_and_unknown_execution_from_current_state`,
  `attention_page_parses_scoped_counts_and_freshness`,
  `collaboration_fixtures_cover_required_states_and_long_content` need CI.
- [ ] Relevant full Agent Bus/protocol suites pass; changed DTO compatibility is covered.
  Pending: additive JSON fields; run CI.
- [ ] Default and warp_platform application checks pass where application changes require them.
  Pending: `panel.rs` changed; run CI.
- [ ] Applicable macOS/Windows/Linux artifact/build coverage passes without publication.
  Pending CI.
- [ ] Local/SSH/direct WSL acceptance passes in supported disposable environments.
  Pending.
- [ ] Static and live native screenshots pass for applicable UI/platform/locale states.
  Pending.
- [x] Relevant localization, format, script/YAML and documentation checks pass.
  Evidence: `python3 script/check-interface-locales.py --strict` → 2981 pairs,
  2949 sites, 0 unclassified; fixture JSON parsed. Rust formatting was not
  applied because the files are not rustfmt-clean upstream.
- [ ] Exact-source receipts are recorded; fixture and live-vendor claims remain separate.
  Pending commit/CI receipts.

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
| Rust source | working tree | `rustfmt --check --config skip_children=true` (parse only) | macOS | Parsed; formatting not applied (files not rustfmt-clean upstream) | No compile/test claim; CI gate pending |

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
| R2-01 | existing_unverified | `TaskStart` requires assignee/revision/version, serializes per assignee and returns the task payload; `capture_thread_binding` records attempt/revision/run/terminal/program/workspace; transport rejects stale runs. Tests: `native_thread_bindings_survive_terminal_reuse_and_never_grant_restart_ownership`, `supervision_allows_dispatch_and_completion_while_other_tasks_are_running`. | SP01 |
| R2-02 | existing_unverified | `NativePty::{exit_code,is_active,stop}` plus Unix group/Windows Job Object ownership; submit/review paths do not call stop; Companion release requires leader exit, inactive owned group and closed output. Test: `owned_group_can_stop_background_children_after_observing_leader_exit`. | SP02, SP03 |
| R2-03 | existing_unverified | Launch, binding, registration and readiness remain separate; `prepare_binding` rejects reused terminals; remote launch reconciles exact run/root; registration recovery does not overwrite a checkout. Test: `socket_wait_handoff_and_expired_run_rejection`. | SP04 |
| R2-04 | existing_unverified | Bounded inbox/request stores, `CONTROL_MESSAGE_RESERVE`, per-assignee serialization, ordered wake claim/finish and guarded native input. Tests: `archival_recovers_capacity_and_full_inboxes_preserve_stop_requests`, `every_receiver_state_preserves_work_and_initial_readiness_is_one_shot`. | SP05 |
| R2-05 | existing_unverified | Structured domain errors and run/attempt fences; no terminal-text retry, account/model switch or whole-task replay. Tests: `unknown_outcomes_require_the_unchanged_original_owner_attempt_and_run`, `native_permission_and_disconnect_observations_survive_broker_restart`. | SP06 |
| R2-06 | demonstrated_gap (fixed) | `sweep_at` deleted expired reservations linked to unresolved `unknown` attempts, contradicting `Reservation.abandoned`; fixed to retain active or unresolved-unknown rows while collecting resolved rows. New test: `expired_reservations_keep_unknown_owners_until_execution_resolves`. Conflict checks remain TTL-only and advisory. | SP07 |
| R2-07 | existing_unverified | Monotonic event/message sequences, generation-fenced wakes, current-revision `observe_tasks`, derived attention and durable inbox pagination. Tests: `dormant_agents_wake_without_an_open_wait_call`, `cooperative_wait_delivers_later_results_without_discarding_the_head`. | SP08, SP09 |
| R2-08 | existing_unverified | Binding exposes attempt/revision/agent/terminal/program/run/workspace and task context/handoff pins. Executable, argv, model/prompt and vendor conversation identity are not stored; record as capability limits, not invented fields. No UI change was required. | SP10 |

- [x] R2-06 source correction and focused regression added.
  Evidence: `crates/agent_bus/src/storage.rs` (`sweep_at`, test
  `expired_reservations_keep_unknown_owners_until_execution_resolves`);
  source-only until CI executes it.
- [x] R2 local source/document checks passed.
  Evidence: Rust source parse via `rustfmt --edition 2021 --emit stdout`;
  `python3 script/check-interface-locales.py --strict` (2981 pairs);
  panel fixture JSON parse; `git diff --check`. These are not runtime tests.
- [ ] R2-01: audit task/attempt/run/conversation separation and reused-process
  context; verify SP01 without forcing a new process per task.
- [ ] R2-02: audit task/turn/leader/owned-child lifetimes and human review handoff;
  verify SP02/SP03 for actual native ownership paths, without killing user jobs.
- [ ] R2-03: audit existing workspace/launch/registration/readiness stages and
  partial failures; verify SP04 without adding automatic launching.
- [ ] R2-04: verify existing queue backpressure, fairness and control handling
  under saturation/replay storms (SP05). New slot scheduling remains conditional;
  no automatic launcher is not grounds to skip existing queue correctness.
- [ ] R2-05: distinguish failure classes and retry owners; verify independent
  peer health and old-run error fencing (SP06); no layered vendor-task retries.
- [ ] R2-06: verify separate runs do not imply separate writable checkouts;
  exercise native worktrees/advisory reservations for SP07. Source fix added:
  expired reservations linked to unresolved-unknown attempts now retain their
  `abandoned` warning; CI execution of the focused regression is pending.
- [ ] R2-07: verify duplicate/replaceable notice handling and reconstruction of
  the whole supervision set after context loss/handover (SP08/SP09).
- [ ] R2-08: audit bounded launch/config provenance and supported CLI capabilities;
  verify SP10 without model/config rewrites or invented conversation identity.
- [ ] Record source-matched CI/test evidence for SP01 through SP10, distinguishing
  source presence, owned native process fixtures and actual vendor capability.

New pause controls, automatic provisioning and role/model/prompt editors remain
separate product decisions. External events and access-mode remediation remain
excluded. The first-round prompt is retained at
[its existing spec location](AGENT-EXECUTION-1.9.0.md); the R2 prompt is a separate
root file and does not recreate or restart that first-round assignment.

## Exclusions

External event intake/connectors; danger-full-access remediation; new permission
flows/sandbox; new model runtime, telemetry, cloud/billing, permanent Router,
cross-host federation, transcript ingestion, vector storage, new daemon and
automatic launching. Do not turn a conditional item into a new product feature.
