---
layout: docs
title: "Deep Dive: Multi-Agent Workflows"
description: "Orchestrate multi-agent teams with Worktree isolation, coordinator task pools, human-in-the-loop approvals, peer review, and native MCP synchronization."
section: Using Warpai
doc_id: using-warpai-agents
prev_url: /docs/using-terminal.html
prev_title: Terminal & Command Blocks
next_url: /docs/cli-agents.html
next_title: 20+ CLI Agents Guide
---

This guide is an actionable manual for orchestrating AI coding agents with Warpai. It walks you through starting agent sessions, managing autonomous task queues, running multi-agent research teams in **Project Mode**, leading parallel software engineering in **Worktree Mode**, and coordinating remote agents over SSH and WSL 2.

---

## Part 1: Setting Up Agent Communication

Before running collaborative agents, enable the communication bridge:

1. Open **Settings** (`Cmd + ,` on macOS, `Ctrl + ,` on Windows).
2. Go to **Features > Agent communication**.
3. Toggle the **Agent communication** switch **ON**.
4. Review the detected CLI agent checklist (e.g., `Claude Code`, `Codex`, `Gemini`, `Cursor`, `Qoder`, `Trae`, `Antigravity`, etc.) and ensure the agents you use are checked.
5. Close Settings. Open a fresh terminal tab (`Cmd + T` / `Ctrl + T`) to ensure bridge environment variables (`WARP_AGENT_ENDPOINT`) are active.

---

## Part 2: Running & Supervising Single Agents

### 1. Launching an Agent
In any Warpai terminal pane, type your agent’s standard CLI command and press Enter:
```bash
claude   # Claude Code
codex    # OpenAI Codex
agy      # Google Antigravity
```

### 2. Reading the Agent's Live State
In the **Left Collaboration Panel**, your agent appears as an active participant showing:
- **Agent Icon & Name**: (e.g., Claude, Codex, Gemini).
- **Process ID & Checkout Path**: Verifying which directory it is operating within.
- **Screen State Indicator**:
  - 🟢 **`Idle`**: Completed last turn; sitting at prompt waiting for work.
  - 🟡 **`Working`**: Actively generating code, executing bash commands, or querying models.
  - 🔴 **`WaitingApproval`**: Paused at a safety barrier (e.g. confirming command execution or file edits).

<div class="docs-shot-placeholder" data-shot="SHOT-13">
  <div class="shot-header">
    <span class="shot-badge">SHOT-13 (P1)</span>
    <span class="shot-title">Human-in-the-Loop Approval Prompt</span>
  </div>
  <p class="shot-desc">人机审批拦截交互：终端内 Agent 触发高危操作提示确认（Do you want to run...? [y/n]），左侧面板锁定 WaitingApproval，展示安全防护。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1000x500 (2:1)</span>
    <span>目标路径: docs/images/agent-waiting-approval-terminal.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## Part 3: Project Mode (Flat Multi-Agent Collaboration)

**Project Mode** connects multiple agents inside a single project folder into a collaborative research or design squad. No Git repository is required.

- **Activate**: At the top of the left collaboration panel, click **Project**.
- **Cross-Agent Messaging**: Agents discover each other via `warp_agent_list` and communicate with `warp_agent_ask` and `warp_agent_reply`.
- **Shared Memory**: Agents publish architecture contracts using `warp_context_publish` for peers to retrieve via `warp_context_get`.

<div class="docs-shot-placeholder" data-shot="SHOT-15">
  <div class="shot-header">
    <span class="shot-badge">SHOT-15 (P1)</span>
    <span class="shot-title">Project Mode Flat Collaboration</span>
  </div>
  <p class="shot-desc">Project 模式多智能体协同：非 Git 目录下多个 Agent（如 Claude + Codex）通过消息通信、互答疑问与发布共享上下文契约。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/project-mode-collaboration.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## Part 4: Worktree Mode & Coordinator Delegation

In software development, multiple agents editing the same local branch causes git merge hell and broken builds. **Worktree Mode** isolates each agent into its own Git worktree directory.

### 1. Setting Up Worktrees
```bash
git worktree add ../project-api feature/api
git worktree add ../project-ui feature/ui
git worktree add ../project-tests feature/tests
```

### 2. Selecting the Coordinator
1. Switch to **Worktree** mode at the top of the collaboration panel.
2. Choose one agent from the **Coordinator** dropdown.
3. The Coordinator automatically oversees all worktrees, dispatches tasks, and manages peer reviews.

<div class="docs-shot-placeholder" data-shot="SHOT-02">
  <div class="shot-header">
    <span class="shot-badge">SHOT-02 (P0)</span>
    <span class="shot-title">Worktree Mode with Coordinator</span>
  </div>
  <p class="shot-desc">Worktree 并行研发核心流：顶部选定 Coordinator，展示 feature/api、feature/ui、feature/test 三个物理分支 Worktree 各自独立绑定不同 Agent 并行施工。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/warpai-worktree-coordinator.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

### 3. Assigning Tasks
The Coordinator or developer can assign tasks with inputs, deliverables, and dependencies.

<div class="docs-shot-placeholder" data-shot="SHOT-16">
  <div class="shot-header">
    <span class="shot-badge">SHOT-16 (P1)</span>
    <span class="shot-title">Task Assignment &amp; Pool Dialog</span>
  </div>
  <p class="shot-desc">Coordinator 任务分发对话框：展示任务标题、指派目标分支、详细说明与前置拓扑依赖设置界面。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1100x700</span>
    <span>目标路径: docs/images/worktree-task-assignment.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## Part 5: Peer Review & Integration Pipeline

1. **Submission**: Worker finishes coding and calls `warp_task_submit` with git diff details.
2. **Review Assignment**: Coordinator designates another peer agent as reviewer (`warp_task_review`).
3. **Verdict**:
   - `needs_changes`: Reviewer rejects with actionable items; worker refactors.
   - `accepted`: Reviewer approves.
4. **Integration**: Coordinator merges branch, executes tests, and commits via `warp_task_integrate`.

<div class="docs-shot-placeholder" data-shot="SHOT-03">
  <div class="shot-header">
    <span class="shot-badge">SHOT-03 (P0)</span>
    <span class="shot-title">Peer Review &amp; Integration Verdict</span>
  </div>
  <p class="shot-desc">同行审查闭环：展示 Worker 提交任务后，Reviewer Agent 给出审查意见（Accepted / Needs Changes），右侧展现 Git 差异审查面板高亮对比。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/warpai-peer-review-diff.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>
