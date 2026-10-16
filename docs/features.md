---
layout: docs
title: Features Matrix & Warp Comparison
description: "Comprehensive breakdown of Warpai features, privacy improvements, stripped bloat, and architectural comparisons with Warp upstream and traditional terminal emulators."
section: Core Architecture
doc_id: features
prev_url: /docs/concepts.html
prev_title: Core Concepts & Safety
next_url: /docs/using.html
next_title: User Guide Overview
---

Warpai is the local-first, privacy-focused modern terminal built around autonomous AI coding agents. This document provides a complete breakdown of its features, architectural differences from upstream Warp, and comparisons with traditional terminals.

---

## 1. Upstream Warp vs. Warpai Comparison

<div class="table-wrapper">
<table>
  <thead>
    <tr>
      <th>Dimension</th>
      <th>Warp Upstream</th>
      <th>Warpai</th>
      <th>User Benefit / Distinction</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Cloud Dependency</strong></td>
      <td>Requires Warp Cloud service connection.</td>
      <td><strong>100% Local-First</strong></td>
      <td>No single point of failure; works offline and behind firewalls.</td>
    </tr>
    <tr>
      <td><strong>Account &amp; Login</strong></td>
      <td>Mandatory account sign-up and login gate.</td>
      <td><strong>Zero Login Gate</strong></td>
      <td>Instant launch out of the box; no email, password, or account needed.</td>
    </tr>
    <tr>
      <td><strong>Telemetry &amp; Privacy</strong></td>
      <td>Collects telemetry, Sentry crashes, usage metrics.</td>
      <td><strong>Zero Telemetry (0 KB)</strong></td>
      <td>All telemetry and tracking stripped. Commands and code never leave your machine.</td>
    </tr>
    <tr>
      <td><strong>License &amp; Pricing</strong></td>
      <td>Proprietary with tier limits and cloud billing.</td>
      <td><strong>Free &amp; Open Source (AGPL-3.0)</strong></td>
      <td>No paywalls, no monthly subscription, no artificial throttling.</td>
    </tr>
    <tr>
      <td><strong>AI Architecture</strong></td>
      <td>Locked to Warp's proprietary cloud AI.</td>
      <td><strong>Open Ecosystem (20+ CLI Agents)</strong></td>
      <td>Use Claude Code, Codex, Gemini, Cursor, Antigravity, etc., with your own keys.</td>
    </tr>
    <tr>
      <td><strong>Multi-Agent Coordination</strong></td>
      <td>Single chat assistant sidebar only.</td>
      <td><strong>Dual Architecture (Worktree Mode + Project Mode)</strong></td>
      <td>Isolated Git worktrees for concurrent code development and peer reviews; omnidirectional messaging &amp; dynamic Commander/Worker roles for non-coding workflows (research, design, finance, IP).</td>
    </tr>
    <tr>
      <td><strong>Screen State Detection</strong></td>
      <td>None for external CLI processes.</td>
      <td><strong>VT100 Screen Observer</strong></td>
      <td>Real-time parsing of <code>Working</code>, <code>Idle</code>, and <code>WaitingApproval</code> states.</td>
    </tr>
    <tr>
      <td><strong>Automated Dispatch</strong></td>
      <td>Manual copy-pasting across windows.</td>
      <td><strong>Guarded PTY Delivery</strong></td>
      <td>Queued tasks automatically inject when agent reaches an idle prompt.</td>
    </tr>
    <tr>
      <td><strong>Coordination API</strong></td>
      <td>Proprietary closed protocols.</td>
      <td><strong>38+ Native MCP Tools</strong></td>
      <td>Open Model Context Protocol tools for tasks, messaging, memory, and lock reservations.</td>
    </tr>
    <tr>
      <td><strong>Developer Workbench</strong></td>
      <td>Terminal only.</td>
      <td><strong>Terminal + Project Explorer + Editor + Git Review</strong></td>
      <td>Integrated file tree, lightweight code editor with Vim bindings, Markdown preview, and diff review.</td>
    </tr>
    <tr>
      <td><strong>Remote Environments</strong></td>
      <td>Standard SSH.</td>
      <td><strong>Self-contained Companion + WSL 2</strong></td>
      <td>Remote companion embeds private Git runtime; no Node.js or Python required on server.</td>
    </tr>
    <tr>
      <td><strong>Quota &amp; Balances</strong></td>
      <td>Only tracks official Warp credits.</td>
      <td><strong>Multi-Provider Quota Dashboard</strong></td>
      <td>Real-time quota bars and reset windows for Claude, OpenAI, Gemini, Cursor, etc.</td>
    </tr>
  </tbody>
</table>
</div>

---

## 2. Feature Inventory by Tier

### Tier 1: Core Killer Differentiators

#### 1. 20+ Native CLI Agents Managed
- Automatic detection and lifecycle tracking for **Claude Code, Codex, Gemini CLI, Cursor CLI, Antigravity (agy), Qoder, QoderCN, Trae, Mistral Vibe, Goose, Hermes, DeepSeek Harness (dsh-tui), Grok, Copilot CLI, Pi, oh-my-pi (omp), OpenCode, Amp, Auggie, Droid**.
- Custom manifest configuration for any third-party terminal agent.
- **Zero Global Pollution**: No changes to `~/.zshrc`, `PATH`, or agent global config.

#### 2. Dual Collaboration Modes: Project vs. Worktree
- **Project Mode (Heavily Supporting Non-Coding Workflows)**: Eliminates rigid master-worker hierarchies. All agents act flexibly as Commanders or Workers, freely sending messages, initiating structured Q&A, and pulling tasks from shared pools. Tailored for research writing, creative design, quant finance, patent compliance, media operations, and due diligence without Git constraints.
- **Worktree Coding Mode (For Git Repositories)**: Beyond the Coordinator who plans and integrates, each agent develops independent features concurrently in isolated physical checkouts, fully preventing lock contention and build cache contamination.
- **Coordinator & Automated Peer Review**: The lead agent supervises branch dependencies, dispatches tasks, facilitates automated peer reviews (`needs_changes` vs. `accepted`), and executes mainline integration commits.

#### 3. Screen State Machine & Guarded PTY Delivery
- Built-in VT100 parser tracks ANSI rendering stream into three states: `Working`, `Idle`, and `WaitingApproval`.
- **Guarded PTY Delivery**: Automatically injects queued tasks when the agent stabilizes at an idle prompt. Pauses when unsubmitted user drafts are detected or when approval is pending.

#### 4. 38+ Native MCP Tools (`warp-lite-communication`)
- Full task lifecycle and dynamic supervision (`warp_task_assign`, `warp_task_list`, `warp_task_claim`, `warp_task_progress`, `warp_task_submit`, `warp_task_review`, `warp_task_integrate`). Agents can continuously dispatch tasks to multiple peers at any time; new tasks automatically join the dynamic supervision list and actively exit upon final resolution.
- P2P messaging (`warp_agent_send`, `warp_agent_ask`, `warp_agent_reply`, `warp_agent_inbox`, `warp_agent_ready`).
- Shared context and memory (`warp_context_publish`, `warp_project_bootstrap`).
- Concurrent file reservations (`warp_file_reserve`, `warp_file_release`).

---

### Tier 2: Modern GPU Terminal Core

- **GPU Acceleration**: Built on `wgpu`, supporting Apple Metal on macOS, DirectX 12 / Vulkan on Windows, and Vulkan / OpenGL on Linux.
- **Atomic Command Blocks**: Commands and their outputs are treated as individual, interactive cards. Copy input, copy clean output, search outputs, and fold long runs.
- **Smart Autocompletion**: Interactive parameter suggestions with inline documentation for hundreds of popular CLI commands.
- **Global Command Palette**: `Cmd + P` (macOS) / `Ctrl + P` (Windows) for instant fuzzy search across actions, settings, and tabs.
- **Split Panes & Vertical Tabs**: Arrange terminal panes horizontally or vertically with keyboard shortcuts.

---

### Tier 3: In-Terminal Developer Workbench

- **Project Explorer**: Full directory tree synced with terminal working directory, file operations (create, rename, delete), and path copying.
- **Lightweight Code Editor**: Built-in editor with syntax highlighting, soft wrapping, and modal Vim bindings (`:w`, `:q`, normal/insert modes).
- **Live Markdown & Image Preview**: In-app rendering of GFM tables, math formulas, checklists, and local/remote images.
- **Terminal-Native Git Code Review**: Side-by-side diff viewer with color-coded additions and deletions right inside the terminal.
