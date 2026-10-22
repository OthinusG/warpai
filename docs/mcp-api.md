---
layout: docs
title: Native MCP Tools API
description: "Complete technical reference for the 38+ local MCP tools (warpai-local-com) for task assignment, peer review, file reservation, and agent messaging."
section: Agent Ecosystem
doc_id: mcp-api
prev_url: /docs/cli-agents.html
prev_title: 20+ CLI Agents Guide
next_url: /docs/config.html
next_title: Configuration & Themes
---

The **Model Context Protocol (MCP)** provides the open coordination standard connecting independent CLI agents into a unified engineering team. 

When Agent Communication is enabled in Warpai, the bundled `warpai-local-com` MCP server exposes **38+ native tools** over standard JSON-RPC 2.0 to delegate tasks, conduct code reviews, publish contracts, and coordinate file edits.

## MCP Server Names

Each Agent environment has its own MCP server name:

| Agent environment | MCP server name |
| --- | --- |
| Local desktop | `warpai-local-com` |
| SSH remote account | `warpai-ssh-com` |
| Direct WSL distribution/user | `warpai-wsl-com` |

---

## 1. Task Lifecycle, Queuing & Dynamic Supervision Tools

### `warp_task_assign`
Assigns a structured task to a target agent or worktree checkout.
- **Parameters**:
  - `to` (string, required): Unique name/ID of the target agent.
  - `description` (string, required): Full technical specifications and expected deliverables.
  - `acceptance` (string, required): Concrete acceptance criteria and verification commands.
  - `context_refs` (array of ContextRef, optional): Pinned context snapshots (max 16).
  - `reviewer` (string, optional): Designated reviewer agent ID (defaults to assigner).
  - `dependencies` (array of string, optional): List of prerequisite task IDs.
  - `start_deadline` (string, optional): ISO-8601 start deadline.
  - `execution_timeout_seconds` / `review_timeout_seconds` (integer, optional): Execution and review timeouts.
  - `request_id` (string, required): Idempotency UUID.
- **Returns**: `{ "id": "uuid", "state": "queued" | "blocked", "delivery": "queued", ... }`
- **Task Queuing Semantics**: Calling `warp_task_assign` **only queues work**; it does not block the caller or wait for completion. The task enters the target agent's durable task queue (`state: "queued"`). The sender can immediately continue assigning additional tasks to the same or other agents. Newly assigned tasks automatically enroll into the supervisor's dynamic supervision set.

### `warp_task_start`
Pops and begins execution of a queued task revision.
- **Parameters**:
  - `task_id` (string, required): Unique ID of the task to start.
  - `revision` (integer, required): Task revision number (starts at 1).
  - `expected_version` (integer, optional): Optimistic concurrency version check.
  - `request_id` (string, required): Idempotency UUID.
- **Returns**: `{ "id": "uuid", "state": "running", "attempt_id": "uuid", ... }`
- **Single-Active-Execution Guard**: An agent can only actively run **one task at a time** (`state == "running"`). If another task is already executing, starting a new task is rejected with `"An assigned task is already running"`. Additional tasks remain safely queued in arrival order until the active task is submitted or completed.

### `warp_task_create_pool`
Creates an unassigned task in the shared project pool (`state: "queued"`), visible to eligible agents.
- **Parameters**: `description`, `acceptance`, `context_refs`, `eligible` (array of agent IDs), `reviewer`, `dependencies`, `request_id`.
- **Returns**: `{ "id": "uuid", "state": "queued", ... }`

### `warp_task_claim`
Claims an unassigned task from the shared pool and transitions it directly from `queued` to `running`.
- **Parameters**: `task_id` (string, required), `expected_version` (integer, optional), `request_id` (string, required).
- **Concurrency Guard**: Claiming is permitted only when the claiming agent has no other running task.

### `warp_task_list`
Lists tasks visible to the agent, filtered by status or assignee (`state`, `assignee`, `cursor`, `limit`).
- **Dynamic Refresh**: Refreshes the active dynamic supervision set across assignees (including offline peers), retaining unresolved tasks and retiring handled terminal outcomes.

### `warp_task_get` / `warp_task_history`
Reads detailed task status, deadlines, execution uncertainty, and ordered event history.

### `warp_task_progress`
Updates the progress percentage, milestone notes, or waiting reason on an active task without transferring ownership.
- **Parameters**: `task_id`, `revision`, `note`, `wait_reason`, `request_id`.

### `warp_task_submit`
Submits completed work and verification evidence for peer review.
- **Parameters**:
  - `task_id` (string, required)
  - `revision` (integer, required)
  - `result` (string, required): Summary of implementation
  - `evidence` (string, required): Verification test outputs or modified file paths
  - `evidence_ids` (array of string, optional): Bounded evidence descriptors
  - `request_id` (string, required)
- **Returns**: `{ "id": "uuid", "state": "submitted" }`
- **Queue Release**: Submitting frees the agent's active execution slot, allowing it to start the next queued task via `warp_task_start`.

### `warp_task_review`
Records a code review verdict on a submitted task.
- **Parameters**:
  - `task_id` (string, required)
  - `revision` (integer, required)
  - `accepted` (boolean, required): `true` to approve, `false` to request changes.
  - `feedback` (string, required): Actionable review comments.
  - `request_id` (string, required)
- **Returns**: `{ "id": "uuid", "state": "accepted" | "queued", ... }`
- **Rework Queuing**: If `accepted == false`, the task automatically increments its revision number and re-enters the assignee's task queue (`state: "queued"`) for rework.

### `warp_task_integrate` *(Coordinator-only)*
Integrates approved changes from a worker checkout into the main project.
- **Parameters**: `task_id` (string, required), `commit` (string, optional), `request_id` (string, required).
- **Returns**: `{ "id": "uuid", "state": "integrated" }`

### `warp_task_cancel` / `warp_task_finish_cancel` / `warp_task_fail` / `warp_task_retry` / `warp_task_reassign` / `warp_task_set_dependencies`
Full lifecycle administration:
- `warp_task_set_dependencies`: Declares prerequisites. Tasks with unmet dependencies automatically enter `state: "blocked"`, and automatically transition to `state: "queued"` when all prerequisites are accepted.
- `warp_task_cancel`: Issues a cancellation request. Running tasks receive a stop request which the worker confirms via `warp_task_finish_cancel`.
- `warp_task_fail`: Reports execution failure with diagnostic evidence.
- `warp_task_retry` / `warp_task_reassign`: Retries failed or expired tasks into `state: "queued"`.

---

### Task Queuing & Dynamic Supervision Architecture

Warpai implements an asynchronous, queue-backed multi-agent task supervision architecture:

1. **Durable Task Queuing & FIFO Consumption**: Every assigned task is durably recorded in SQLite as `state: "queued"`. When Worker A is busy on Task 1, newly assigned Task 2 and Task 3 line up in Worker A's queue in arrival order. Worker A executes them sequentially via `warp_task_start` upon submitting previous work. This eliminates file collisions, dirty Git index contention, and LLM context thrashing.
2. **Continuous Non-Blocking Dispatch**: The Coordinator / supervising agent does not operate in rigid upfront batches. It can dispatch tasks to one or multiple agents at any time. Sending only queues work, allowing the supervisor to keep the pipeline full without waiting for workers to finish.
3. **Dynamic Enrolment in Supervision List**: Every newly enqueued task immediately joins the supervisor's active supervision list (`dynamic supervision set`), tracking progress, blocking questions, and verification states across turn yields (`warp_agent_ready`).
4. **Automatic Dependency & Rework Transitions**: Tasks blocked by prerequisite dependencies (`state: "blocked"`) automatically unblock and enter `queued` when prerequisites are accepted. Tasks rejected in review (`accepted: false`) automatically increment revision and re-enter `queued` for rework.
5. **Active Queue Exit Upon Completion**: Tasks can enter the queue and supervision list at any time, and actively/automatically leave the list as soon as their terminal outcome is handled and accepted, keeping the active queue clean without stale polling loops.

<div class="docs-shot-placeholder" data-shot="SHOT-26">
  <div class="shot-header">
    <span class="shot-badge">SHOT-26 (P1)</span>
    <span class="shot-title">Task Queuing & Dynamic Supervision List</span>
  </div>
  <p class="shot-desc">任务排队与动态监督：展示主 Agent 随时派发多任务进入队列（queued 状态），Worker 串行执行（running 状态），以及完成审阅后主动出队归档的全生命周期。</p>
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
