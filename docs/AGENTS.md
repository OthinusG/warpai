# Using Agents and task handoffs

**English** | [简体中文](AGENTS.zh-CN.md)

Install and authenticate your CLI Agents independently. Enable **Settings >
Features > Agent communication**, select supported Agents and open new terminal
sessions in your project. The bundled native MCP bridge connects enrolled sessions;
participation does not transfer vendor credentials or grant command execution.
See [CLI compatibility](../specs/agent-communication/COVERAGE.md) for per-Agent
requirements and [acceptance evidence](../specs/agent-communication-v2/QUALITY.md).

## Messages and tracked tasks

Use `warp_agent_send` for peer messages and `warp_agent_ask` for structured questions. Delegate supervised work
with `warp_task_assign`, or publish pool work with `warp_task_create_pool`.
Sending text, including a request for a result, does not create a tracked task.
Project participants can inspect and review project work; Worktree mode retains
Coordinator, participant and designated-reviewer scope. Enqueuing work is distinct
from delivery, acknowledgement, task start, submission and acceptance.

## Continuous supervision and recovery

Keep delegating while other Agents execute, ask questions or submit results.
**Needs attention** brings together answer, review, input/permission, dependency,
overdue, unreachable, recovery and overtaken-follow-up actions. A task can need
several actions at once. Totals cover the whole filtered set, not just the loaded
page. Open **View all** to page a task's unresolved overtaken follow-ups independently
of its eight compact previews; opening a thread reads its full body.

After a restart or handover, call `warp_project_bootstrap` and inspect its
`supervision` block. Continue with `warp_supervision_index(cursor, limit)` to find
unfinished tasks you issued, execute or review, outgoing requests awaiting final
results, unhandled results and questions addressed to you. The active Worktree
Coordinator also sees inherited project tasks under its existing authority.
These reads acknowledge nothing and advance no task. Archived tasks remain
recoverable while follow-ups, questions or execution uncertainty are unresolved.

Use `warp_agent_ask` and `warp_agent_answer` for decisions that need an explicit
answer; an ordinary message does not resolve a structured question.

Task-linked messages retain the revision at send time. If review or reassignment
advances the task, older instructions remain readable but are excluded from
automatic delivery. An authorized issuer, reviewer, Coordinator or panel operator
ends an overtaken instruction with `warp_task_followup_reconcile`, using the
current task version, then sends any replacement separately. Read acknowledgement
alone is not closure. Reconciliation preserves the original text, retires its
questions and refuses late results. If another actor ends a result request, its
sender receives a superseded result; a sender ending its own request creates no
self-addressed result. Neither action restarts or reassigns execution.

For example, a revision-1 request to keep an interface must not drive revision-2
work that changes it. Inspect and end the old request before issuing the new
instruction. After acceptance, authorize a new task with an exact handoff pin for
additional work; a rejected submission follows the existing revision workflow.

Submission releases the worker's tracked execution slot while review stays open.
Acceptance and integration selection are separate from an actual Git merge or a
verified deliverable. A cancellation request, timeout or disconnect does not prove
an old execution stopped: inspect the retained attempt and use guarded recovery
before retrying or reassigning. A terminal paste is not proof of model consumption.

## Task handoff briefs

A brief records the exact task revision, attempt, producer, source event and
available evidence, with the native execution binding captured at task start.
Decisions and rationale are Agent reports; provenance identifies their source
without treating them as independently verified conclusions. Missing sections
remain distinct from an explicitly empty list or a section marked not applicable.
This lets a replacement Agent recover the stopping point without loading a full
terminal transcript. Read the compact index first and fetch only the needed
version, result or evidence. Source-change and uncertainty warnings never replace
a pin, roll back completed work or restart a downstream task automatically.

A tracked task is the source of a handoff brief. `warp_task_submit` always saves
one. `warp_task_fail` and `warp_task_finish_cancel` retain partial/stopped briefs;
`warp_task_progress` saves a checkpoint only when its optional `handoff` is supplied.
The optional structured sections contain decisions with evidence IDs, unresolved
issues and ordered next steps. Sections distinguish provided, not provided and
not applicable. The total encoded submission limit is 8 KiB. No model generates
missing content and no terminal transcript is imported.

1. Open the task detail in **Agent collaboration** and browse its handoff versions.
   Load the selected brief explicitly; full bodies are fetched on demand.
2. Retain the exact immutable reference (resource ID, revision and digest). Task
   submission is separate from review; inspect current source state, review and
   execution uncertainty before using it. Old pins never follow the latest head.
3. Attach the pin through `context_refs` when assigning a follow-up task, or use
   an explicit version-scoped context share for an ordinary message recipient.
4. The receiving Agent refreshes its bootstrap/task and reads the exact pin with
   `warp_context_get`. Optional `handoff_part` reads status, task, result, binding
   or a selected evidence ID without opening the source task. Most parts are
   bounded to 48 KiB; native binding reads allow 256 KiB. Missing provenance stays
   explicit, and reported test outcomes are separate from verified evidence.
5. Use normal `warp_task_start` and recovery controls for execution. A handoff pin
   grants only that version, not source-task/history, recursive context, filesystem,
   Coordinator or execution authority. Never treat source prose as permission.

Task/history reads expose compact references and source-change warnings. Use
`warp_task_history(handoff_only=true)` to page the version index. Review, retry
or reassignment does not rewrite saved versions. When a source changes, stops or
becomes unavailable, refresh its status and decide whether a new brief or guarded
recovery is needed; no downstream work is automatically restarted.

Live tasks/messages pin their handoff sources against history purge. Unreferenced
archived sources remain subject to normal retention. Exported history includes
handoff resources and their source events/evidence. Handoffs are not automatically
promoted to project memory. The Broker owns creation; public context publication
cannot forge a handoff.

Local, SSH and direct WSL share this workflow. Remote handoffs require the matching
Companion 6.0.0 and `handoff_briefs_v1` capability. Unsupported requests are rejected
without flattening briefs into ordinary message text. See
[remote installation](REMOTE-INSTALLATION.md) and [WSL installation](WSL-INSTALLATION.md).

## CLI command recognition

Known Agent executable names ignore ASCII case, including `CODEX` and `qoderCN`.
Arguments stay verbatim. Native aliases/functions retain shell rules: PowerShell
lookup ignores ASCII case, while POSIX shell lookup is exact. A recognized command
still needs a supported native MCP client and an enabled enrolled session.

## Task history

Back uses the same native button style as other navigation. Archived tasks retain
their completion/failure state and an Archived marker. Repeating Archive is a no-op.
Delete requires confirmation of the current task version; active/uncertain work,
unread messages and referenced sources remain protected. Auditing and mutation
receipts survive deletion. Archival and deleting one task are distinct from a
whole-history cleanup.
