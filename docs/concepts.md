---
layout: docs
title: Core Concepts & Safety Guarantees
description: "Understand Warpai's foundational design principles: agents as units of work, project vs worktree modes, and guarded PTY delivery."
section: Core Architecture
doc_id: concepts
prev_url: /docs/quick-start.html
prev_title: Quick Start (5 Min)
next_url: /docs/features.html
next_title: Features Matrix & Comparison
---

Understanding Warpai’s underlying design principles will help you unlock the full power of autonomous multi-agent engineering.

---

## 1. The Agent Session as the Unit of Work

In traditional IDEs and chat-centric AI tools, an AI assistant is typically confined to a floating chat box or sidebar. Every request is an ephemeral conversational thread, and the human developer remains the sole router responsible for copying code, switching files, and validating changes.

**Warpai treats an Agent session as an autonomous unit of work**:
- An agent has its own execution process, working directory, and environment context.
- It can receive structured tasks with deliverables and acceptance criteria.
- It can report progress, ask other agents for clarification, request code reviews, and deliver verified diffs.
- Warpai provides the terminal substrate and coordination bus to let multiple agents collaborate like an organized engineering squad.

---

## 2. Dual Collaboration Modes: Project vs. Worktree

Warpai supports two complementary collaboration models, toggled at the top of the collaboration panel:

```
┌─────────────────────────────────┬─────────────────────────────────┐
│          PROJECT MODE           │          WORKTREE MODE          │
│                                 │                                 │
│ • Flat, peer-to-peer network    │ • Hierarchical Coordinator flow │
│ • No Git repository required    │ • Requires a Git repository     │
│ • Shared single workspace       │ • Isolated physical worktrees   │
│ • Ideal for: Research, design,  │ • Ideal for: Feature build,     │
│   documentation, data analysis  │   refactoring, parallel tests   │
└─────────────────────────────────┴─────────────────────────────────┘
```

### Project Mode (Shared Research & Brainstorming)
In **Project Mode**, all active agents in the project communicate in a shared collaborative context.
- Agents can message any peer, exchange information, and delegate subtasks.
- Tasks, context resources, and project memories are visible to all project participants.
- Operates in any folder (Git or non-Git), making it ideal for research notes, specs, and architectural exploration.

### Worktree Mode (Parallel Software Engineering)
When working on real codebases, having multiple agents edit the same physical files at the same time leads to file conflicts, compilation race conditions, and context poisoning.

**Worktree Mode solves this using Git's native worktree mechanism**:
- Each feature, module, or test suite is assigned its own isolated checkout branch in a separate directory.
- Agents operate in physical isolation. Changes in Worktree A cannot break the build in Worktree B.
- A designated **Coordinator Agent** presides over the team, orchestrating tasks, reviewing submitted diffs, and integrating approved code into the main branch.

---

## 3. Screen State Machine & Lifecycle Detection

CLI agents are terminal applications running over a pseudo-terminal (PTY). To coordinate them automatically without invasive wrapper hacks, Warpai implements an in-engine **VT100 screen observer**.

Using dedicated behavioral manifests (`crates/agent_bus/src/screen_detection/manifests/*.toml`), Warpai continuously samples the terminal output stream and derives the agent's exact lifecycle state:

<div class="table-wrapper">
<table>
  <thead>
    <tr>
      <th>Lifecycle State</th>
      <th>Description</th>
      <th>Warpai Action</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong><code>Working</code></strong></td>
      <td>Agent is running, executing bash commands, or querying models.</td>
      <td>Queues incoming messages and tasks safely into the agent's private inbox.</td>
    </tr>
    <tr>
      <td><strong><code>Idle</code></strong></td>
      <td>Agent finished its turn and is sitting at the command prompt.</td>
      <td>Triggers <strong>Guarded PTY Delivery</strong> to automatically dispatch queued tasks.</td>
    </tr>
    <tr>
      <td><strong><code>WaitingApproval</code></strong></td>
      <td>Agent triggered a safety prompt (e.g., confirming file writes or high-risk commands).</td>
      <td>Holds delivery; pulses tab coral and alerts the developer for interactive review.</td>
    </tr>
  </tbody>
</table>
</div>

<div class="docs-shot-placeholder" data-shot="SHOT-12">
  <div class="shot-header">
    <span class="shot-badge">SHOT-12 (P1)</span>
    <span class="shot-title">Agent Screen Detection Three-State</span>
  </div>
  <p class="shot-desc">Agent 状态机三态对比特写：左侧协作面板中清晰展示 Working（黄色执行）、Idle（绿色就绪）、WaitingApproval（红色等待授权）三种状态图标与标记。</p>
  <div class="shot-meta">
    <span>建议尺寸: 800x600 (4:3)</span>
    <span>目标路径: docs/images/agent-lifecycle-states.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 4. Guarded PTY Delivery

Unlike naive script injection that blindly sends characters over a pipe—often resulting in truncated commands or interrupted outputs—Warpai enforces **Guarded PTY Delivery**:

1. **Prompt Synchronization**: Commands are injected only when the VT100 parser verifies an active, stable prompt line.
2. **Draft Collision Guard**: If you have begun typing a manual command, automated delivery halts immediately so your input is never overwritten or submitted prematurely.
3. **Approval Interception**: When an agent requests user consent, the session stays strictly interactive until you approve or reject.

<div class="docs-shot-placeholder" data-shot="SHOT-14">
  <div class="shot-header">
    <span class="shot-badge">SHOT-14 (P1)</span>
    <span class="shot-title">Guarded PTY Automated Delivery</span>
  </div>
  <p class="shot-desc">守护式任务自动排队执行：忙碌时排队入队，一旦检测到空闲且终端无未敲完草稿，自动把任务注入 PTY 执行的流水线过程。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1200x650</span>
    <span>目标路径: docs/images/agent-guarded-inbox-delivery.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>
