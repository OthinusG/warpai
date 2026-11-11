# Using Agents and task handoffs

**English** | [简体中文](AGENTS.zh-CN.md)

Install and authenticate your CLI Agents independently. Enable **Settings >
Features > Agent communication**, select supported Agents and open new terminal
sessions in your project. The bundled native MCP bridge connects enrolled sessions;
participation does not transfer vendor credentials or grant command execution.
See [CLI compatibility](../specs/agent-communication/COVERAGE.md) for per-Agent
requirements and [acceptance evidence](../specs/agent-communication-v2/QUALITY.md).

## Messages and tracked tasks

Use `warp_agent_send` for peer messages and questions. Delegate supervised work
with `warp_task_assign`, or publish pool work with `warp_task_create_pool`.
Sending text, including a request for a result, does not create a tracked task.
Project participants can inspect and review project work; Worktree mode retains
Coordinator, participant and designated-reviewer scope. Enqueuing work is distinct
from delivery, acknowledgement, task start, submission and acceptance.

Task-linked mail records the task revision when it is sent. When the task revision
changes, an older message or request is overtaken: it stays in history, is not
delivered as current work and must not be executed; the recipient reports it to the
issuer. An authorized issuer, reviewer
or Coordinator ends it with `warp_task_followup_reconcile` using a fresh
`expected_version`, then sends the replacement instruction at the current revision.
An overtaken result request is answered with a superseded result. A plain message
or a request you sent yourself produces no result; the receipt and the recorded
reconciliation are the terminal fact. A reconciled follow-up cannot later accept a
succeeded, failed or blocked result, while a message that was only read remains
answerable. Questions asked against the reconciled follow-up are retired with it.
Reconciling preserves the original message and never restarts, reassigns or
re-permissions the task.

The **Needs attention** section shows at most eight follow-up previews per task
with a shown/total count, and marks a preview that was truncated; open the
follow-up thread (or the message when it has no thread) for the full body. Pages
are re-read from current facts rather than a snapshot, so end the shown
follow-ups and refresh to load later ones.

## Task handoff briefs

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
