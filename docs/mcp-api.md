---
layout: docs
title: Native MCP Tools API
description: "Complete technical reference for the 38+ local MCP tools (warp-lite-communication) for task assignment, peer review, file reservation, and agent messaging."
section: Agent Ecosystem
doc_id: mcp-api
prev_url: /docs/cli-agents.html
prev_title: 20+ CLI Agents Guide
next_url: /docs/config.html
next_title: Configuration & Themes
---

The **Model Context Protocol (MCP)** provides the open coordination standard connecting independent CLI agents into a unified engineering team. 

When Agent Communication is enabled in Warpai, the bundled `warp-lite-communication` MCP server exposes **38+ native tools** over standard JSON-RPC 2.0 to delegate tasks, conduct code reviews, publish contracts, and coordinate file edits.

---

## 1. Task Lifecycle & Dynamic Supervision Tools

### `warp_task_assign`
Assigns a structured task to a target agent or worktree checkout.
- **Parameters**:
  - `title` (string, required): Short descriptive summary of the task.
  - `description` (string, required): Full specifications, acceptance criteria, and expected deliverables.
  - `assignee` (string, optional): Unique name/ID of the target agent. If omitted, task is added to the project pool.
  - `worktree` (string, optional): Specific Git checkout directory for this task.
  - `dependencies` (array of string, optional): List of prerequisite task IDs.
- **Returns**: `{ "task_id": "uuid", "status": "assigned" }`
- **Dynamic Supervision**: Newly assigned tasks are immediately enrolled into the supervising agent's dynamic supervision list. The supervising agent can continuously dispatch tasks to multiple agents at any time without batch boundaries; tasks remain tracked until finished, and actively exit the supervision list once resolved.

### `warp_task_list`
Lists tasks visible to the agent, filtered by status or assignee.
- **Dynamic Refresh**: Refreshes the active dynamic supervision set across assignees (including offline peers), retaining unresolved tasks and retiring handled terminal outcomes.

### `warp_task_claim`
Claims an unassigned task from the project pool.
- **Parameters**: `task_id` (string, required)
- **Returns**: `{ "task_id": "uuid", "claimed_by": "agent_id", "status": "in_progress" }`

### `warp_task_progress`
Updates the progress percentage and current milestone of an active task.
- **Parameters**:
  - `task_id` (string, required)
  - `percent` (integer, required): 0 to 100
  - `message` (string, optional): Brief update on current activity
- **Returns**: `{ "task_id": "uuid", "updated": true }`

### `warp_task_submit`
Submits completed work for peer review.
- **Parameters**:
  - `task_id` (string, required)
  - `summary` (string, required): Overview of changes made
  - `evidence` (array of string, optional): Test outputs or modified files
- **Returns**: `{ "task_id": "uuid", "status": "submitted_for_review" }`

### `warp_task_review`
Records a code review verdict on a submitted task.
- **Parameters**:
  - `task_id` (string, required)
  - `verdict` (string, required): `"accepted"` or `"needs_changes"`
  - `feedback` (string, required): Actionable review comments
- **Returns**: `{ "task_id": "uuid", "new_status": "accepted" | "in_progress" }`

### `warp_task_integrate` *(Coordinator-only)*
Integrates approved changes from a worker checkout into the main project.
- **Parameters**:
  - `task_id` (string, required)
  - `commit_hash` (string, optional): Final verified Git commit hash
- **Returns**: `{ "task_id": "uuid", "status": "integrated" }`

### `warp_task_cancel` / `warp_task_retry` / `warp_task_reassign`
Administrative controls to cancel abandoned work, retry failed tasks, or transfer ownership to a new agent.

### Dynamic Supervision & Continuous Dispatch Logic
Warpai implements an asynchronous, continuous multi-agent task supervision architecture:
1. **Continuous Dispatch at Any Time**: The primary or coordinating agent does not operate in rigid "batch dispatch" phases. It can dispatch tasks to one or multiple agents at any point in time, even while prior tasks are actively executing.
2. **Dynamic Enrolment in Supervision List**: Every newly dispatched task automatically joins the supervising agent's monitored task list (`dynamic supervision set`), tracking progress, blocking questions, and verification states across turn yields (`warp_agent_ready`).
3. **Active Exit Upon Completion**: Tasks can enter the supervision list at any time, and actively/automatically leave the list as soon as their terminal outcome is handled and accepted, keeping the supervising agent's focus clean without stale polling loops.

<div class="docs-shot-placeholder" data-shot="SHOT-26">
  <div class="shot-header">
    <span class="shot-badge">SHOT-26 (P1)</span>
    <span class="shot-title">Continuous Dispatch & Dynamic Supervision List</span>
  </div>
  <p class="shot-desc">动态任务监督与随时派发：协同面板 Tasks 区域特写，展示主 Agent 动态监督列表接纳随时派发的多分支任务，呈现 in_progress 实时进展及完成后自动退出。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1200x650</span>
    <span>目标路径: docs/images/mcp-dynamic-task-supervision.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 2. Agent Communication & Readiness Tools

### `warp_agent_ready`
Yields the current turn when no immediate actions remain and the agent owns no executing work.
- **Parameters**: None.
- **Returns**: `{ "acknowledged": true, "pending_count": integer }`
- **Supervision Invariant**: Outstanding supervision is preserved across yields—yielding does not mean delegated work is complete. Call as your final tool action before returning to the prompt.

### `warp_agent_wait`
Waits up to 20 seconds for the next notification or event not yet delivered in this run.
- **Parameters**: `timeout_ms` (integer, optional)
- **Supervision Handling**: Used only when no immediate action is available. On notification or timeout, the agent reassesses the dynamic supervision set, dispatches any newly authorized work, and removes resolved items before deciding whether to wait again or yield with `warp_agent_ready`.

### `warp_agent_list`
Discovers all currently active agents in the project.
- **Parameters**: None.
- **Returns**: List of agent objects with `name`, `role`, `status` (`working` | `idle`), and `checkout_root`.

### `warp_agent_send`
Sends a structured message to a peer agent.
- **Parameters**:
  - `to` (string, required): Destination agent name
  - `body` (string, required): Message text
  - `expects_result` (boolean, optional): If `true`, requires an explicit callback and adds the message ID to the sender's dynamic supervision set. Senders continue dispatching or messaging other agents at any time while the result is pending.
- **Returns**: `{ "message_id": "uuid", "delivery": "delivered" | "queued" }`

### `warp_agent_reply`
Provides a definitive callback to an `expects_result` request.
- **Parameters**:
  - `request_id` (string, required)
  - `outcome` (string, required): `"succeeded"`, `"failed"`, or `"blocked"`
  - `body` (string, required): Explanation or payload
- **Returns**: `{ "request_id": "uuid", "acknowledged": true }`

### `warp_agent_inbox` / `warp_agent_ack`
Retrieves pending notifications and unacknowledged messages from the agent's private queue. Handle observations, then explicitly acknowledge via `warp_agent_ack`.

---

## 3. Shared Context & Project Memory

### `warp_context_publish`
Publishes shared architecture decisions, schemas, or API contracts.
- **Parameters**:
  - `key` (string, required): Context key identifier
  - `content` (string, required): Markdown or JSON string content
  - `pinned` (boolean, optional): Keep pinned across project sessions
- **Returns**: `{ "key": "string", "stored": true }`

### `warp_context_get` / `warp_context_list`
Retrieves a specific context resource by key or lists all published contexts in the project.

### `warp_project_bootstrap` / `warp_project_memory_update`
Reads or persists shared project-level memories across multiple agent restarts.

---

## 4. Concurrent File Reservations (Locks)

### `warp_file_reserve`
Reserves a file before beginning modifications to prevent multi-agent edit conflicts.
- **Parameters**:
  - `file_path` (string, required): Path relative to repository root
  - `duration_seconds` (integer, optional): Lock duration (default: 300s)
  - `reason` (string, optional): Feature or refactor description
- **Returns**: `{ "file_path": "string", "reserved": true, "expires_at": timestamp }`

### `warp_file_renew` / `warp_file_release` / `warp_file_reservations`
Renews an active lock, releases it upon task completion, or lists all active file locks in the workspace.
