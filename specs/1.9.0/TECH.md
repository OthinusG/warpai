# Warpai 1.9.0 execution specification

Date: 2026-11-08. Status: planning; source audit must precede implementation.
Product behavior: [PRODUCT.md](PRODUCT.md). Work ledger: [TODO.md](TODO.md).
First-round operator instructions: [execution prompt](AGENT-EXECUTION-1.9.0.md).

Owner exclusions: no external event intake/connectors and no danger-full-access
remediation, permission-flow changes or sandbox work. Existing identity and
credential safeguards remain. This plan does not authorize implementation by itself.

## Closed-loop follow-up and attention delta (2026-11-11)

Audit result: revision-pinned follow-ups, stale skipping in native wake/wait/
pending, task/attempt/run/version fences, idempotent receipts and Coordinator
authority were already implemented and are reused unchanged. Two gaps and one
reachability limit were fixed:

1. `agent_send` accepted an explicit `task_id` that disagreed with the task
   inherited from `reply_to`. It now returns `invalid_input`, as `agent_ask`
   already did.
2. Ending an overtaken instruction had no durable operation, so attention could
   report `reconcile` forever. `task_followup_reconcile` revalidates project,
   task version and message revision, acknowledges the original message without
   rewriting it, and — when the actor is not the sender — queues a
   `result_failed` (`Superseded follow-up`) with the current revision to release
   a waiting `expects_result` sender. It changes no ownership, execution or
   permission and is fenced by the existing `request_id` receipt. Sending the
   replacement instruction remains a separate `agent_send`.
3. Attention was capped at 100 rows with no way to reach later tasks.
   `operator_attention_page` builds a shared CTE, computes `total`/`counts` over
   the entire filter, and pages rows by `created_seq`; SQL aggregates at most 8
   raw follow-up bodies per task and the store decodes each structured body to
   its display text, truncating to 512 characters. The panel passes
   `attention_after`, renders the current-vs-message revision and reuses the
   task thread/controls; filters and archive scope reset the cursor.

A second audit identified four remaining gaps; regression tests were added with
the fixes and are pending CI execution:

4. Reconciliation was not terminal. `AgentReply` now refuses any outcome when a
   persisted `followup_reconciled` event names the instruction; using the event
   (looked up through `events(project,resource=task,kind)`) instead of
   `acknowledged` keeps an ordinary read message replyable.
5. A self-issued request had no durable closure. The receipt
   (`result_message_id: null`) plus the event are the terminal record and no
   self-addressed result is queued; a request from another sender still receives
   the superseded `result_failed`.
6. Reconciliation retires unacknowledged `question` rows whose `reply_to` is the
   reconciled message and records `retired_questions`; questions bound to other
   instructions or revisions are unaffected.
7. Attention previews expose the decoded display text plus `truncated`; the
   panel shows `Showing X of Y`, labels a preview as truncated and opens the
   thread by message ID when the follow-up has none. Rendering is still bounded
   at 8 previews — the full structured body comes from `thread_get`, not a
   larger preview.

Reply and reconciliation race inside one transaction: the loser observes the
committed acknowledgement or event and returns `invalid_state`; no second
terminal result exists. No schema, protocol-major or lifecycle change was made.

Compatibility: additive `attention_after` request field and `cursor` response
field; an older Companion ignores the request field and its response lacks
`followups`/`cursor`, which the panel treats as absent (no previews, no next
page). `followup_reconcile` is a local feature, Companion capability and SSH-
required project feature, so unsupported transports refuse the operation instead
of silently ignoring it.
Vendor structured observation remains `existing_unverified` (screen manifests
only); no adapter, retry wrapper or account/model switching was added. Rust
tests, Companion builds and native captures are recorded in [TODO.md](TODO.md).

Capability matrix (all rows: screen manifest present, structured-event adapter
absent, vendor turn/permission/retry/session events `unknown`):

| CLI (manifest) | Screen states | Structured events | Session/run identity | Retry/recovery events |
| --- | --- | --- | --- | --- |
| claude | existing | none verified | unknown | unknown |
| codex | existing | none verified | unknown | unknown |
| gemini | existing | none verified | unknown | unknown |
| cursor | existing | none verified | unknown | unknown |
| qoder | existing | none verified | unknown | unknown |
| amp, antigravity, auggie, copilot, droid, dsh, goose, grok, hermes, omp, opencode, pi, trae, vibe | existing | none verified | unknown | unknown |

`shared.toml` holds common patterns only. Screen text never proves turn start/
end, message consumption, task completion, or vendor retry; only native
lifecycle/MCP operations advance task state. Adding a vendor adapter requires a
documented structured interface plus a running vendor instance for acceptance.

## Implementation status (2026-11-08)

Additive changes are in the working tree: task-linked `agent_send` messages now
record the current task revision and are not auto-delivered after a revision
bump (they remain in the inbox and surface as the derived `reconcile` attention
action); `operator_panel` derives a bounded, filter-scoped attention projection
and the existing collaboration panel renders it with existing task buttons. No
operation, schema, protocol-major or lifecycle state was added. Runtime tests,
application checks, native captures and transport acceptance are recorded as
pending in [TODO.md](TODO.md) because local Rust builds are prohibited by
`MEMORY.md`.

## R2 execution delta (2026-11-08)

The R2 source audit classified R2-01 through R2-08 against current source; no CI
or native acceptance has run. One demonstrated gap was fixed at the shared Store
owner: `sweep_at` deleted an expired advisory reservation when its linked attempt
was `unknown`, while `Reservation.abandoned` defines that row as an
unknown-owner warning. Expired rows now remain when the linked attempt is
`active` or `unknown`; finished, interrupted and unlinked rows are collected as
before. TTL expiry still permits conflicting new
reservations: the retained row is an observation, not an OS lock. Regression:
`expired_reservations_keep_unknown_owners_until_execution_resolves`. No
API/schema/protocol/UI change. R2 classifications and pending SP gates are in
[TODO.md](TODO.md).

## Context

The research baseline is `a70280b4c`; re-read current source before using these
locations. Line references are navigation hints, not immutable patch locations.

| Area | Existing implementation to inspect |
| --- | --- |
| Operations and identities | `crates/agent_bus/src/lib.rs:210` (`Operation`), `:803` (`Attempt`), `:816` (`Task`) |
| Shared transactions and recovery | `crates/agent_bus/src/storage.rs:1070` (`recover`), `:1429` (`questions`), `:1444` (`task_supervision`); task operation match arms in the same file |
| Integration selection | `crates/agent_bus/src/storage_orchestration.rs:303` (`integrate_task`) |
| Guidance and bootstrap | `crates/agent_bus/src/mcp.rs:63`; `project_memory_storage.rs` (`project_bootstrap`, `execution_thread`) |
| Context/handoff | `crates/agent_bus/src/{context,context_storage,handoff,handoff_storage,project_memory,project_memory_storage}.rs` |
| Native delivery | `crates/agent_bus/src/{transport,transport_orchestration,session,readiness,native_pty}.rs`; screen observation modules and corresponding application wake paths |
| Native UI | `app/src/agent_communication.rs`, `app/src/agent_communication/{panel,panel_controls,panel_handoff}.rs` |
| Static reference | `specs/agent-communication-v2/panel-fixtures.json`, existing native panel, shared Appearance controls/themes and source-matched captures |
| Existing regression harness | `crates/agent_bus/tests/{coordination,managed_agent,readiness,native_pty,ssh_companion,ssh_native_mcp,wsl_workspace}.rs`, inline Store tests and `handoff_storage` tests |
| CI and localization | `.github/workflows/validate-agent-communication.yml`, `validate-remote-companion.yml`, `script/check-interface-locales.py` |

Active contracts remain `specs/agent-communication-v2/{API,PLAN,WORKTREE-COLLABORATION,HANDOFF}.md`.
Read dated corrections and supersession notes before applying older text.

## Proposed changes

1. Create a capability audit in this ledger before changing application code.
   For each invariant, identify callers, shared handler, persistence, projection,
   transport, fixture and test; classify existing behavior as verified, unverified
   or a demonstrated gap. A verified item can close with evidence and no code.
2. Prefer existing operations over new DTOs: `TaskStart`, task-linked `AgentSend`,
   `AgentAsk/Answer/Reply`, `TaskSubmit/Review/Retry/Reassign`, context/handoff reads
   and `TaskIntegrate`. Preserve request replay and atomic state/event/notification
   boundaries. Do not infer execution authority from a message body.
3. Repair gaps in the shared Store/Broker/native-delivery owner. Trace all local,
   SSH and WSL callers first. Add transport-specific code only for a proven
   transport-specific gap. Do not emulate vendor internal history or next-step
   steering through terminal parsing.
4. Keep completed results immutable through follow-up work. Accepted work gets a
   new task with an exact handoff pin; rejection advances the current task through
   its existing review path. Do not broaden retry eligibility to reopen accepted
   work. For a queued follow-up overtaken by a revision change, retain its original
   association and show/reconcile it; never silently execute it against a new revision.
5. Represent attention as a derived collection of pending actions, not a new
   task lifecycle. Reuse existing supervision facts. If the panel projection lacks
   a required field, add the smallest scoped optional projection and document
   capability behavior. Define count semantics across pagination; a loaded-page
   count must not appear to be the total for the whole project.
6. Use existing panel sections, row controls and button themes. Establish static
   fixtures/screenshots before wiring real data. Preserve focus, scrolling,
   drafts, lazy handoff loading and stale-response fences. Existing permission
   observations may be displayed; no new permission handling flow is introduced.
7. Reuse evidence descriptors for base/head/commit/checks and actual deliverables.
   Do not relabel `git_execution_implied=false` as a successful merge. Inspect
   current review-to-evidence binding before deciding any new field is necessary.
8. Keep compact bootstrap, pinned handoffs and reviewed project memory. Preserve
   source authorization, inaccessible versus absent states and protected retention.
   No new summaries inferred from terminal output, semantic store or cleanup daemon.
9. Capacity is an audit-and-fix branch only. If the current dispatch path is
   sufficiently bounded and no in-scope admission change needs more, record a
   conditional skip with evidence and add no setting or scheduler.

Before any API change, update the active API contract and this specification with
field semantics, defaults, serialization/omission, validation, authorization,
bounds, stale/replay behavior, capability negotiation and mixed-version cases.
Before any schema change, prove existing fields cannot represent the requirement;
specify consistent backup, migration, restart, capacity and retention tests. No
protocol major, Companion version or database migration is automatic scope.

## Testing and validation

| Scenario | Invariants | Required observation |
| --- | --- | --- |
| S01: notify a busy worker, then explicitly start | BI-01, BI-02 | Notification cannot consume the request or assert execution; guarded queue drains later |
| S02: A submits, B asks, C is dispatched, supervisor yields/resumes | BI-03, BI-04 | All IDs remain tracked; questions/results are processed without exclusive watching or reply loops |
| S03: ordinary message versus structured answer/final result | BI-04 | Question remains open until answer; final reply atomically consumes only its original request; blocked remains outstanding |
| S04: follow-up during review and after acceptance | BI-02, BI-07 | Correct revision route; accepted original unchanged; new authorized task pins exact brief |
| S05: duplicate request and changed payload with same UUID | BI-01, BI-05 | Original receipt replays; payload mismatch rejected; no duplicate task/result/notification effect |
| S06: restart/disconnect before and after start/submit | BI-05, BI-11 | Retained history and unknown execution survive; no automatic write replay or new owner |
| S07: stale run/revision/attempt/checkout/remote response | BI-05, BI-06 | No mutation of current work; current panel cannot consume late data from another scope |
| S08: cancel requested, late result, guarded retry/reassign | BI-05 | Stop confirmation and unresolved-attempt rules hold; override is explicit and auditable |
| S09: Coordinator change, mode/focus change, non-Git project | BI-03, BI-06 | Correct authority and preserved task/reviewer provenance; no automatic role election |
| S10: submit, reject, resubmit, accept, select integration | BI-07 | New content cannot reuse obsolete approval; selection does not imply actual Git execution |
| S11: question/review/recovery overlap and paginated list | BI-08, BI-09 | One task row with multiple actions; authorized counts agree; no hidden later-page work |
| S12: stale/withdrawn/denied/unsupported handoff and cleanup | BI-10 | Exact pins, bounded reads and protected sources; no authority escalation |
| S13: saturated queues and concurrent dispatch/start | BI-12 | Control/answers remain deliverable; admitted capacity is respected, or conditional skip is proven |
| S14: native static then live captures | BI-08, BI-09, BI-11 | Narrow/normal light/dark and all three languages; no clipped controls, lost drafts or stale actions |

Use deterministic scheduling, barriers/fake clocks and disposable databases,
accounts and repositories. Do not introduce sleeps that hide races. At each crash
boundary assert both stored facts and observable notifications, not just a final
state label. Use existing test fixtures rather than paid vendor calls.

Rust builds/native acceptance run in the existing GitHub workflows per MEMORY.md;
local checks are limited to source/document/script checks. Verify current workflow
inputs and trigger conditions first. Do not use `--no-run` or zero matching tests
as execution evidence. Relevant CI commands include:

```sh
cargo test -p warp-agent-bus --locked
cargo test -p remote_protocol --locked
cargo check -p warpai --bin warpai --locked
cargo check -p warpai --bin warpai --features warp_platform --locked
python3 script/check-interface-locales.py --strict
```

Run focused filters before broader relevant suites, with real pass counts. Use
existing feature flags/WSL fixtures where required; verify exact test names and
supported OS from current workflows. Preserve Linux desktop build coverage in
the existing artifact path when application changes affect it. Do not publish.

For each gate record commit SHA, command/workflow inputs, OS, executed tests,
status, run/artifact URL and limitations in `TODO.md`. Real vendor acceptance,
deterministic fixtures, compilation and visual acceptance are separate gates.
No performance/reliability claim is permitted without measurements.

## Completion rule

All applicable P0/P1 items need linked evidence. A proven conditional capacity skip
is acceptable; missing required environments remain blocked/pending. Documentation
or a green subset cannot stand in for missing transport or visual acceptance.
The implementation Agent must leave a concrete reviewable diff and report every
remaining gate without claiming overall completion prematurely.
