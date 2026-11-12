# Communication branch audit

Date: 2026-11-12. Review only; no application changes or new CI dispatches.

## Scope and baselines

- Reviewed branch: `r2-agent-lifecycle-ci`, `999a1a16f9e6d679c9cb83eed4c171346f71a9b8`.
- Main: `a70280b4c07efff08e57badb92fdf649e188ec7e`.
- GitHub branch heads matched the local checkout and `origin/main` at review time.
- Official dsh reference: `deepseek-ai/dsh-superagent-prototype`, current main
  `8c32a9dc5d72c033816c6a8110b37f40cf94fba4`.
- Reviewed shared storage/delivery, operator projection, panel controls, MCP and
  Companion capability changes, tests, workflow changes and acceptance receipts.

## Findings

### P1: Row-bounded attention can exceed the remote response byte limit

`storage.rs` returns up to eight follow-ups per task and truncates each body with
`display.chars().take(512)`. `transport.rs::operator_panel` requests 50 task rows.
`companion_tasks.rs` serializes the entire panel and rejects results above
`MAX_RESULT = 512 * 1024`. There is no aggregate byte budget for attention.

Reproduced by extracting the actual attention CTE and page SQL into Python
SQLite, seeding 50 queued revision-2 tasks with eight unread revision-1 messages
each, and using 512 Chinese characters per body. The preview arrays alone were
662,421 UTF-8 bytes, above 524,288 before other panel fields or `truncated` flags.
The fixture fits the 1,000-message recipient limit. This is a source-SQL and
serialization-bound reproduction, not an executed Rust/SSH acceptance test.

Impact: a valid busy project can make SSH/direct-WSL panel refresh fail with
`ManagedCapacityExceeded`; task-count pagination does not guarantee reachability.

Smallest repair: enforce an aggregate serialized-byte budget, return a cursor
for rows omitted by that budget, and account for other fields sharing the frame.
Prefer existing bounded-response helpers where compatible. Add one actual
Companion panel regression with multibyte bodies, near-limit content and cursor
continuation; increasing the transport limit alone leaves the root cause.

### P2: Read task follow-ups cannot be explicitly ended when overtaken

`AgentAck` permits acknowledging ordinary `message` entries. `AgentReply`
deliberately leaves these messages replyable until a final result or a durable
`followup_reconciled` event exists. However, `TaskFollowupReconcile` rejects
`message.acknowledged`, and both attention follow-up queries require
`acknowledged=0`.

Sequence: send a task-linked ordinary message at revision 1; recipient ACKs it;
the task advances to revision 2; issuer attempts to end the obsolete instruction.
It is absent from reconciliation attention and the mutation returns
`invalid_state`. A later result remains eligible because no reconciliation event
exists. This does not bypass task-revision mutation fences: the reply retains
its original revision. The gap is explicit instruction closure and visibility.

The source-SQL check confirmed an acknowledged stale message disappears from
attention without any terminal event. Handler review establishes the rejection
and reply eligibility; a complete Rust sequence was not run locally.

Smallest repair: distinguish read acknowledgement from terminal resolution in
the existing queries and reconcile guard, using final-result/reconciliation
facts already stored. Preserve ordinary read-message replies before explicit
closure. Cover ACK -> revision bump -> reconcile -> refused late reply, plus a
completed message that must not be reconciled again.

## Actual improvements over main

1. Task-linked sends pin a revision and reject an explicit task differing from
   `reply_to`. Wake and wait delivery skip overtaken instructions while retaining
   history. Main did not provide this complete path.
2. Explicit follow-up reconciliation checks task authority/version, records a
   durable terminal event, resolves derived questions and refuses later results.
   Waiting non-actor request senders receive a superseded result; self-issued
   requests produce no self-result. The MCP tool and remote capability gate
   share the same operation.
3. Operator attention derives overlapping actions without inventing task states.
   Totals/counts cover the filtered set; task rows have stable cursor pagination.
   The UI exposes thread reads, truncation and explicit confirmation. Per-task
   follow-ups beyond eight are reached by ending earlier entries and refreshing,
   rather than by independently browsing their pages.
4. Expired reservations retain warnings for unresolved unknown executions.
   Reservations remain advisory and TTL does not become an OS write lock.
5. Focused regressions, fixture states, localization and native capture assertions
   were added. Existing task/attempt/run fences, durable requests, review,
   handoffs, native ownership and worktree isolation mostly predate this branch.

## Remaining differences from dsh

- **Structured execution observation:** dsh owns Harness sessions and consumes
  session/turn/step/retry events. Warpai has explicit MCP facts and native/screen
  observation but no verified vendor-event adapter. Quota data does not identify
  a task's provider failure. Integrate one documented vendor interface at a time;
  retain native run/attempt fencing and unknown states for unsupported providers.
- **Deterministic supervision:** dsh has runtime-owned serialized/coalesced
  passes and durable effects. Warpai persists tasks/messages, but supervisor
  decision making depends on an external Agent following MCP instructions and
  yielding safely. This is weaker liveness assurance, not loss of durable work.
- **Background-job association:** dsh knows session-owned jobs/subagents and uses
  them for slot retention and human handoff. Warpai's owned process-group/Job
  Object lifetime is useful, but does not establish universal task-to-job
  identity for user-owned CLIs or a reused process.
- **Capacity visibility and human intervention:** dsh reserves a continuation
  slot and exposes executor load/pause. Warpai has queue/control reserves and
  guarded delivery, but no measured end-to-end response guarantee or global
  dispatch pause/drain. Do not infer latency guarantees from admission tests.
- **Context inspection:** Warpai pins context/handoffs and launch identity but
  cannot generally attest a reused CLI session's exact model, prompt, consumed
  history or compaction. dsh controls its own Session prompt/model initialization.
- **Independent follow-up browsing:** only eight stale follow-ups per task are
  previewed. Handling older entries reveals later ones, but operators cannot
  freely inspect all later entries before deciding which to end.

Automatic worker provisioning, global scheduling/pause and role/model presets
require product decisions. GitHub/Feishu intake, cross-host federation and new
permission/sandbox flows remain excluded from current scope. Do not classify
these exclusions as implementation regressions or copy dsh's automatic restart
semantics over uncertain writes. dsh is a research prototype, not a reliability
benchmark or proof that its public implementation is production-ready.

## Verification evidence and limits

- Communication run [42972072518](https://github.com/OthinusG/warpai/actions/runs/42972072518)
  at `64dce58ee`: macOS/Windows application and both protocol jobs succeeded;
  system-network was skipped. Native capture steps succeeded. Existing receipts
  record 1,134 PNGs per OS. This review did not download or inspect those PNGs.
- Companion run [42957976165](https://github.com/OthinusG/warpai/actions/runs/42957976165)
  at `7cf9b1437`: Ubuntu/macOS/Windows remote-wire jobs succeeded. Later executable
  changes include native capture code; do not describe this as a remote run of
  the latest code SHA.
- Changes after `64dce58ee` are limited to MEMORY/TECH/TODO documentation.
- Local strict localization check passed: 2,993 translation pairs, 2,958 marked
  display sites, zero unclassified direct display literals. Branch diff check
  passed. The two targeted SQLite/source checks above passed.
- No local Rust builds/tests, new CI runs, live vendor sessions, new screenshots
  or Linux desktop packaging acceptance were performed. Existing V1 Linux desktop
  coverage and live vendor acceptance gates remain open.
- TODO contains historical pending statements later superseded by P0-08. Use
  dated receipts and exact source SHAs, not isolated checkboxes, for acceptance.

Recommended order: repair the response byte budget; resolve read-versus-terminal
follow-up semantics; complete those focused remote regressions and Linux desktop
gates; then select one supported structured vendor observation adapter. Broader
automatic scheduling should follow demonstrated product need.
