---
layout: docs
title: Documentation Overview
description: "Warpai documentation overview: the agent-native, local-first GPU terminal built for autonomous coding agents."
section: Getting Started
doc_id: overview
next_url: /docs/install.html
next_title: Installation & Setup
---

Welcome to the official documentation for **Warpai** — the local-first, privacy-focused modern terminal built around autonomous AI coding agents.

Warpai elevates independent CLI Agent sessions into first-class units of work. It allows you to orchestrate 20+ command-line agents across isolated Git worktrees, supervise parallel feature implementation, conduct automated peer reviews, and integrate changes within a high-performance, GPU-accelerated terminal.

<div class="docs-shot-placeholder" data-shot="SHOT-01">
  <div class="shot-header">
    <span class="shot-badge">SHOT-01 (P0)</span>
    <span class="shot-title">Warpai All-in-One Hero Workspace</span>
  </div>
  <p class="shot-desc">三栏全景视觉：左侧 Agent 协同面板（多 Agent 在线状态与任务池）、中间 GPU 终端执行代码块（Claude Warm Light 主题 + 垂直标签页）、右侧展示 Project Explorer 与代码文件。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1920x1080 (16:9)</span>
    <span>目标路径: docs/images/warpai-hero-showcase.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## Quick Navigation

<div class="table-wrapper">
<table>
  <thead>
    <tr>
      <th>Category</th>
      <th>Document</th>
      <th>Summary</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Get Started</strong></td>
      <td><a href="{{ '/docs/install.html' | relative_url }}">Installation Guide</a></td>
      <td>Direct downloads (macOS &amp; Windows), package managers, Gatekeeper/SmartScreen verification, and Linux Companion.</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/docs/quick-start.html' | relative_url }}">Quick Start (5 Min)</a></td>
      <td>Fast-track setup: launch your first agent, understand the interface, and monitor data quotas.</td>
    </tr>
    <tr>
      <td><strong>Architecture</strong></td>
      <td><a href="{{ '/docs/concepts.html' | relative_url }}">Core Concepts &amp; Safety</a></td>
      <td>Deep dive into guarded PTY delivery, VT100 in-memory screen observation, and Worktree isolation.</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/docs/features.html' | relative_url }}">Features Matrix &amp; Warp Comparison</a></td>
      <td>Comprehensive comparison of Warpai vs. upstream Warp and traditional terminal emulators.</td>
    </tr>
    <tr>
      <td><strong>How to Use</strong></td>
      <td><a href="{{ '/docs/using.html' | relative_url }}">User Guide Overview</a></td>
      <td>High-level workflows for daily engineering tasks, navigation, and multi-agent coordination.</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/docs/using-terminal.html' | relative_url }}">Terminal &amp; Command Blocks</a></td>
      <td>Master command blocks, smart autocompletion, vertical tabs, project explorer, Vim editor, and Git diff review.</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/docs/using-warpai-agents.html' | relative_url }}">Multi-Agent Workflows</a></td>
      <td>Coordinate agent teams, run Worktree branches, assign tasks, conduct peer reviews, and connect over SSH.</td>
    </tr>
    <tr>
      <td><strong>Ecosystem</strong></td>
      <td><a href="{{ '/docs/cli-agents.html' | relative_url }}">20+ CLI Agents Guide</a></td>
      <td>End-to-end installation and authentication guide for Claude, Codex, Gemini, Antigravity, Cursor, and 15+ more.</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/docs/mcp-api.html' | relative_url }}">Native MCP Tools API</a></td>
      <td>Reference for the 38+ embedded MCP coordination tools (<code>warp-lite-communication</code>).</td>
    </tr>
    <tr>
      <td><strong>Configuration</strong></td>
      <td><a href="{{ '/docs/config.html' | relative_url }}">Configuration &amp; Themes</a></td>
      <td>Settings reference, appearance themes (Claude Warm Light), custom keybindings, and keep-awake guard.</td>
    </tr>
    <tr>
      <td></td>
      <td><a href="{{ '/docs/troubleshooting.html' | relative_url }}">Troubleshooting &amp; FAQ</a></td>
      <td>Diagnostics, common terminal issues, SSH Remote permissions, and WSL 2 troubleshooting.</td>
    </tr>
  </tbody>
</table>
</div>

---

## Guiding Principles

1. **Local-First & Absolute Privacy**: Zero telemetry, zero cloud accounts, and no closed-source models. Your code, shell commands, and agent prompts never leave your trusted environment without your explicit instruction.
2. **Open Agent Ecosystem**: We do not lock you into a proprietary AI. Use your existing CLI agents (Claude Code, Codex, Cursor, Gemini, Qoder, etc.) with your own keys and accounts.
3. **Engineered for Scale**: Real-world software engineering cannot be solved in a single chat box. Parallel Git worktrees, isolated branches, and designated Coordinator supervision provide the structure needed for autonomous multi-agent delivery.
