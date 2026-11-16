# Communication completion implementation review

Date: 2026-11-13. Reviewed HEAD: `92f32b3efaf5ac36984a2bac69f6a0d663379fdb`.
Code baseline: `999a1a16f`; final implementation commit: `337e4a51b`.

Verdict: **Request changes**. The implementation repairs important parts of A/B,
but introduces a pagination edge case and does not carry the new unresolved
follow-up semantics into retention. The full A–G brief is not complete.

Repair update (2026-11-13): the owner subsequently authorized fixing these two
findings. Shared cleanup eligibility now protects acknowledged unresolved old
instructions, and Private-only budget pages use the existing empty cursor.
`storage_review_tests.rs` covers both cleanup release paths and real SpaceList
continuation at the production byte budget. Cloud verification is tracked in
TODO.md; this repair does not claim completion of the remaining A–G features.

## P1: Retention can delete an acknowledged but unresolved follow-up

Changed attention SQL in `storage.rs:1667` correctly ignores read acknowledgement
and classifies a stale message as unresolved until reconciliation or a final
succeeded/failed result. However, `PURGEABLE_TASK` at `storage.rs:57` still
protects task messages only when `acknowledged=0`.

An archived terminal task with an acknowledged old instruction and no final
result/reconciliation is therefore both actionable and purgeable. `TaskDelete`
uses the same predicate, and `purge_task_record` deletes its messages. Bulk
HistoryPurge has the same gap. This contradicts the new API retention statement.

Reproduction: extracted the current attention CTE, `PURGEABLE_TASK` and
`CERTAIN_TASK` directly from source and executed them in SQLite. A cancelled,
archived revision-2 task with an acknowledged revision-1 ordinary message and
no attempts, results or reconciliation event appeared in both query results.
The handler read confirms the eventual deletion path. No Rust execution is
claimed for this reproduction.

Required correction: make deletion/cleanup guards preserve nonterminal
instructions according to the same event/result semantics. Exercise TaskDelete,
purge preview and HistoryPurge; after explicit closure, cleanup should regain
eligibility without weakening dependency, uncertainty or handoff protection.

## P2: Trimming spaces to Private creates an invalid resume cursor

`panel_budget.rs:27` treats the space name as its universal cursor key.
`trim_collection` at lines 64–78 sets the cursor from the last remaining row.
But `space_list` prepends a synthetic Private row before real spaces sorted by
name. Private is a sentinel, not a member of that sort order.

When budget pressure leaves only Private, `space_cursor` becomes `Private`.
The next `WHERE name > ?` read silently skips actual spaces such as Alpha.
Returning to the first page under unchanged pressure repeats the omission.

Reproduction: source-equivalent tail trimming of `[Private, Alpha, Zulu]` to one
row produced cursor `Private`; the actual SQL comparison returned Zulu and
omitted Alpha. This is a projection/SQLite check, not Rust or native acceptance.

Required correction: preserve the actual scan position for the synthetic row,
or provide an explicit sentinel-aware continuation rule. Cover the case with
the actual SpaceList projection and production budget under aggregate pressure.
The existing generic collection test actually pressures events/history and
does not exercise this special space cursor.

## Scope acceptance

| Brief item | Current review result |
| --- | --- |
| A: aggregate byte budget | Implemented shared 448 KiB budget; cursor bug above; actual large-payload Companion regression absent |
| B: read versus terminal | Main reply/reconcile behavior repaired; retention incomplete |
| C: independent follow-up browsing | Not implemented; no new panel or per-task page path |
| D: panel/recovery experience | Existing behavior retained; requested navigation and UX completion not delivered |
| E: supervision reconstruction/diagnostics | No new implementation or complete current-scope acceptance map |
| F: structured observations/vendor adapter | No adapter; no new capability-validation or complete downgrade implementation |
| G: verification | New protocol/application CI and Linux artifacts pass; latest remote-wire and native screenshot acceptance absent |

The original brief permits existing verified capabilities to close without new
code, but this needs an explicit mapping and matching evidence. An unchecked
remaining-scope note is honest disclosure, not completion or an external blocker.

## Verified evidence

- [Communication run 43208425278](https://github.com/OthinusG/warpai/actions/runs/43208425278)
  matches `337e4a51b`: both protocol and both application jobs succeeded;
  system-network was skipped. Both native capture steps were skipped.
- Downloaded CI log confirms the added budget and ACK tests actually executed.
  Agent Bus unit results: macOS 200 passed / 1 ignored; Windows 193 passed /
  3 ignored. This is observed prior CI evidence, not a local rerun.
- [Linux run 43202112972](https://github.com/OthinusG/warpai/actions/runs/43202112972)
  matches `337e4a51b`: x64 and arm64 package jobs succeeded; release attachment
  was skipped. This closes Linux build/package coverage for this implementation.
- HEAD `92f32b3ef` adds only CI documentation after that code commit.
- The added Store budget test manually calls `bound_panel` on a constructed
  projection. Its continuation pages are read directly from Store without
  bounding/Companion serialization on each iteration. Do not label this an
  executed end-to-end remote byte-budget test.
- Branch diff whitespace check passed. No local Rust builds, new CI dispatches,
  live vendor sessions or screenshot downloads were performed in this review.

## Handoff

Requested a per-item delivery report from `qoder-cn-6f542659` through Warpai MCP,
request ID `a35461d0-2c39-4eb4-ad46-5d0b4e093b93`, and asked the agent to keep
the code fixed while under review. Both findings were sent for response. The
agent returned its delivery report in result message
`3e23bd58-c013-4ac2-83e7-8cbcda952d46`; the result was handled and acknowledged.

The report confirms C/E/F and the remaining D work were not delivered, and G
covers A/B only. Its proposed C operation and an E derived projection do not
automatically require user confirmation: the brief authorized API design and
implementation while retaining current permissions. Only necessary permission
expansion was conditional. Stopping all Operator browsing and supervision work
for API approval was not required by the brief. F remains unverified: no
version-specific official-interface feasibility evidence was supplied, so lack
of an adapter is not established as an unavoidable external blocker.

Reported CI test totals include ignored cases: observed Agent Bus unit results
are 200 passed + 1 ignored on macOS and 193 passed + 3 ignored on Windows.
The reported shared-layer budget test is explicitly not a live SSH/WSL
large-payload acceptance test. The review verdict and scope corrections were
sent back to the implementing agent without authorizing additional edits during
this fixed-version review.
