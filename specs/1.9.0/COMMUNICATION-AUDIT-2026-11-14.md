# Communication audit after the three execution gaps

Date: 2026-11-14. Reviewed HEAD: `0e19cce5b`; executable baseline: `5e69f5e81`.
Scope: reviewer preservation, follow-up paging, durable supervision and compatibility.
No implementation changes, new CI runs, task acceptance or peer monitoring.

Verdict: **Request changes**. The previous reviewer/cleanup/space-cursor defects
are repaired, and the recorded CI is green. Additional logical gaps remain in
the new recovery index and native backward pagination.

Repair update: the owner authorized direct implementation and pushing. The
shared recovery query now applies reconciliation facts, current Coordinator
authority and archived unresolved facts, and emits recipient-scoped question
rows. Follow-up history stores the current page cursor; the native capture
harness exercises first/second/third/back/back/forward/first on real data.
The native-only follow-up interface is documented without claiming an Agent
tool. CI acceptance for these repairs is tracked in TODO.md; the original
findings below remain the historical reproduction record.

## P1: A request ended by its own issuer remains awaiting a result

`supervision_index` excludes outgoing requests only when a succeeded/failed
reply exists (`storage.rs:1937`). It does not check `followup_reconciled`.
When the issuer explicitly reconciles its own overtaken request, the existing
mutation deliberately emits no self-addressed result. The durable terminal
event is its only closure fact, so this request remains `await_result` forever.

Source-SQL reproduction: insert an acknowledged revision-1 request, a revision-2
task and the matching `followup_reconciled` event. Execute the actual current
supervision CTE. The ended request still appears in its request branch, even
though the follow-up branch correctly excludes it. Turning the task terminal
does not eliminate the request row.

Correction: apply the terminal-event rule to request retirement without losing
real result messages that the requester still needs to handle. Test issuer-end,
other-actor-end, reopen, no self-result, and subsequent result acknowledgement.

## P1: Coordinator handover does not reconstruct inherited task supervision

The non-Operator scope is hardcoded to task issuer/assignee/reviewer
(`storage.rs:1889`). It never checks current Worktree Coordinator authority.
The replacement Coordinator usually occupies none of those historical fields,
so its index misses the previous Coordinator's unfinished tasks. Existing
`task_record` explicitly permits the active Coordinator to read these records
(`storage.rs:1330`), so this is a narrower recovery scope than current authority,
not a reason to expand permissions.

Source-SQL check: an otherwise unfinished task naming the old issuer, worker
and reviewer is absent for a new Coordinator ID. Source review confirms no role
lookup changes that predicate. A native handover session was not executed.

Correction: incorporate the existing active, project/run-fenced Coordinator
authority when deriving the recovery scope. Test old/new Coordinator duties,
stale roles, revoked admission and separation from unrelated projects.

## P2: Archived tasks with unresolved ordinary follow-ups disappear from recovery

The task branch requires `t.archived=0` (`storage.rs:1927`) even if its follow-up
or uncertainty counters are nonzero. Archiving terminal work is permitted while
an ACKed old instruction still needs reconciliation; the repaired cleanup guard
preserves that message. If it is an ordinary message, the request/result branches
cannot compensate for the omitted task.

Source-SQL reproduction: an archived cancelled revision-2 task with an ACKed
unreconciled revision-1 ordinary message returns no recovery row. The same
instruction still exists and is actionable in attention when archived tasks are
included. Recovery must not treat archival as explicit instruction closure.

Correction: include archived tasks when relevant unresolved facts remain, or
expose an explicit default recovery rule that cannot silently hide them. Preserve
cleanup eligibility and avoid re-enrolling fully handled historical tasks.

## P2: Standalone unanswered questions have no recovery action

Questions are counted only by task ID. There is no question branch in the
recovery union. `AgentAsk` permits a question about a standalone request using
`reply_to` without a task ID; its recipient has to answer before the worker can
return success. The index lists the parent request only as `await_result`,
because `supervision_actions` returns that action unconditionally for requests.

Source-SQL reproduction: a standalone request and its unacknowledged question
produce a request row but no question/action descriptor. The question is still
readable in the inbox; data is retained, but the supposedly reconstructable
action set is incomplete.

Correction: expose the caller's authorized unanswered question or correlate an
answer action and bounded question reference onto the request. Preserve sender/
recipient visibility and do not duplicate or discard siblings when one is handled.

## P2: Earlier follow-up page skips the immediate preceding page

`NextFollowups` pushes the current cursor (`panel.rs:1920`), but
`EarlierFollowups` pops that saved value and then uses the new last element
(`panel.rs:1925`). Those are two incompatible history conventions.

Exact state-transition reproduction: starting with `[None]`, advancing twice
gives history `[None, None, A]` and current cursor B. Earlier pops A and selects
None, returning to the first page rather than the second. This affects the
101-entry case at a 50-entry page size.

Correction: consistently store either current-page history or prior-page
history. Test at least first -> second -> third -> second -> first, then forward
again, as well as page-empty reset and context/filter changes. Existing live
capture data has nine entries and does not verify a three-page backward sequence.

## P2: Claimed warp_task_followups tool is absent from the schema

The delivery report, TECH and MEMORY refer to `warp_task_followups`. Current
Operation has no matching variant, and MCP `tools()` derives its list exclusively
from Operation (`mcp.rs:215`). The only production caller of task_followups_page
is the trusted native Panel projection in `transport.rs:1671`.

The native browsing capability is real, but an Agent cannot discover/call the
claimed MCP tool. Correct the contract/reporting, or implement a separately
authorized participant-scoped read if it is needed. Do not expose the existing
Operator projection directly to Agents or expand thread reads to close this gap.

## Verification and confidence

- [Communication 43449498695](https://github.com/OthinusG/warpai/actions/runs/43449498695)
  and [Companion 43449505426](https://github.com/OthinusG/warpai/actions/runs/43449505426)
  were independently queried and both succeeded at `5e69f5e81`.
- Existing tests cover the new happy-path reconstruction, including final-result
  acknowledgement, and follow-up forward paging. Those passes are valid; they
  do not establish the additional terminal-event/handover/archive/question cases.
- Executed the actual supervision CTE against a temporary Python SQLite store
  using the current schema. Confirmed the four recovery query gaps above.
- Replayed the exact native cursor-stack transitions without Rust/UI execution.
  Inspected the serde Operation/MCP schema-generation path for tool exposure.
- No local Rust builds, new CI, vendor sessions, native screenshots or live
  Coordinator handover ran in this review. These checks establish source/query
  behavior; fixes should have operation-level and native interaction regressions.
- Preserve existing budgets, authorization, result handling, cleanup guards,
  immutable records and compatibility. No new scheduler/runtime is required.
