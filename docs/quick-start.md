---
layout: docs
title: Quick Start (5 Min)
description: "Launch your first coordinated agent, experience automated state detection, and master the basic multi-agent loop in under 5 minutes."
section: Getting Started
doc_id: quick-start
prev_url: /docs/install.html
prev_title: Installation & Setup
next_url: /docs/concepts.html
next_title: Core Concepts & Safety
---

Get up and running with Warpai in under 5 minutes. Learn how to launch a coordinated CLI agent, observe automated state detection, and explore the integrated developer workbench.

---

## Step 1: Meet the Three-Panel Workspace

When you launch Warpai, you will see a clean, three-column desktop workbench:

```
┌──────────────────┬──────────────────────────────────────────┬──────────────────┐
│  AGENT PANEL     │            GPU TERMINAL                  │  PROJECT PANEL   │
│                  │                                          │                  │
│ • Mode Switch    │ • GPU-accelerated block output           │ • File Explorer  │
│ • Coordinator    │ • Command autocompletion & history       │ • Code Editor    │
│ • Agent Sessions │ • Vertical tab strip                     │   (Vim mode)     │
│ • Active Tasks   │ • Screen lifecycle detection             │ • Live Markdown  │
│ • Quota Footer   │                                          │ • Git Review     │
└──────────────────┴──────────────────────────────────────────┴──────────────────┘
```

1. **Left Panel (Agent Coordination)**: Manage multi-agent collaboration, switch between **Project Mode (omnidirectional messaging for non-coding teams)** and **Worktree Mode (parallel feature coding for Git repos)**, assign tasks, inspect peer reviews, and monitor quota limits.
2. **Center Stage (GPU Terminal)**: A modern, GPU-rendered command line powered by `wgpu`. Features block-based command execution, autocompletion, vertical tabs, and split panes.
3. **Right Panel (Project Workbench)**: In-terminal file tree, lightweight code editor with Vim bindings, Markdown document previewer, and Git Code Review.

---

## Step 2: Enable Agent Communication

Warpai bridges natively with the command-line agents you already have installed:

1. Open **Settings** (`Cmd + ,` on macOS, `Ctrl + ,` on Windows).
2. Navigate to **Features > Agent communication**.
3. Toggle the **Agent communication** switch **ON**.
4. Warpai scans your system `PATH` and presents a checklist of recognized CLI agents (e.g., `Claude Code`, `Codex`, `Gemini`, `Cursor`, `Qoder`, `Trae`, `Antigravity`, etc.).
5. Check the boxes for the agents installed on your machine.

<div class="docs-callout tip">
  <div class="docs-callout-title">Zero Global Pollution</div>
  <p>Checking an agent registers Warpai's local bridge. It <strong>never modifies</strong> your global shell profile (<code>~/.zshrc</code>), PATH, or third-party agent configuration files.</p>
</div>

---

## Step 3: Run Your First Coordinated Agent

1. Click into the center terminal pane.
2. Run your preferred CLI agent command directly:
   ```bash
   # Launch Claude Code
   claude

   # Or launch Codex
   codex

   # Or launch Google Antigravity
   agy
   ```
3. **Watch the Left Panel**: Warpai automatically recognizes the process launch. A participant row appears in the collaboration panel with the agent’s native icon, process identifier, and live state.

---

## Step 4: Automated Screen Detection & Task Queuing

Warpai's built-in VT100 screen observer parses the agent's real-time state:

- **`Working`**: The agent is generating code, running tool calls, or executing tests.
- **`Idle`**: The agent finished its turn and is waiting at the prompt for the next instruction.
- **`WaitingApproval`**: The agent encountered a sensitive tool call and is paused waiting for your explicit approval.

### Try Autonomous Task Queuing:
1. When your agent is busy (`Working`), queued messages and tasks sit safely in its inbox.
2. The moment the agent returns to `Idle`, Warpai's **Guarded PTY Delivery** automatically inputs the pending task into the terminal prompt.
3. **No manual copy-pasting or constant window-switching required!**

<div class="docs-callout note">
  <div class="docs-callout-title">Draft Protection</div>
  <p>If you have typed an unsubmitted draft into the prompt, or if the agent is waiting for permission (<code>WaitingApproval</code>), Warpai pauses automated delivery to protect your input and terminal safety.</p>
</div>

---

## Step 5: Monitor API Quotas & Token Usage

Never run unexpectedly into a rate limit in the middle of a complex multi-step refactor:

1. Look at the **Data Usage** footer at the bottom of the left collaboration panel.
2. It displays live quota consumption bars, remaining hourly/weekly reset windows, and balance information for your connected providers.

<div class="docs-shot-placeholder" data-shot="SHOT-19">
  <div class="shot-header">
    <span class="shot-badge">SHOT-19 (P2)</span>
    <span class="shot-title">Data Usage &amp; Quota Monitor Footer</span>
  </div>
  <p class="shot-desc">多模型额度大盘特写：左侧面板底部展示 4 个供应商（Claude, Codex, Gemini, Cursor）的用量百分比条、5小时/周重置倒计时与美元余额。</p>
  <div class="shot-meta">
    <span>建议尺寸: 800x400 (2:1)</span>
    <span>目标路径: docs/images/data-usage-quota-footer.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>
