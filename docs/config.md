---
layout: docs
title: Configuration & Appearance
description: "Reference for Warpai settings.toml, theme customization, keybindings, and agent communication options."
section: Configuration & Help
doc_id: config
prev_url: /docs/mcp-api.html
prev_title: Native MCP Tools API
next_url: /docs/troubleshooting.html
next_title: Troubleshooting & FAQ
---

Warpai provides both a graphical Settings panel (`Cmd + ,` on macOS, `Ctrl + ,` on Windows) and a declarative `settings.toml` configuration file. All data resides strictly within your local user directory.

---

## Configuration File Locations

- **macOS / Linux**: `~/.config/warpai/settings.toml`
- **Windows**: `%USERPROFILE%\.config\warpai\settings.toml`

Warpai watches this file for changes. Edits made in `settings.toml` or via the GUI sync bidirectionally in real time without requiring an application restart (except interface language changes).

---

## Settings: Agent Communication Toggle

To allow local agents to communicate over the embedded MCP bus:
1. Open **Settings > Features > Agent communication**.
2. Toggle the switch ON.
3. Check the CLI agents installed on your system.

<div class="docs-shot-placeholder" data-shot="SHOT-20">
  <div class="shot-header">
    <span class="shot-badge">SHOT-20 (P2)</span>
    <span class="shot-title">Settings: Agent Communication Toggle</span>
  </div>
  <p class="shot-desc">设置面板中的 Agent 通信配置页：展示 Master 开关开启状态，以及列出的 20+ 款主流 Agent 勾选清单（Claude, Codex, Gemini 等）。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1100x750</span>
    <span>目标路径: docs/images/settings-agent-communication.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## Theme Customization: Claude Warm Light

Warpai ships with the signature **Claude Warm Light** theme as its default light appearance, offering warm sepia paper tones, subtle borders, and readable syntax colors.

<div class="docs-shot-placeholder" data-shot="SHOT-21">
  <div class="shot-header">
    <span class="shot-badge">SHOT-21 (P2)</span>
    <span class="shot-title">Claude Warm Light Theme Presentation</span>
  </div>
  <p class="shot-desc">默认 Claude Warm Light 经典暖光浅色主题全景展示：呈现舒适优雅的暖调质感与高对比度语法配色。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/appearance-claude-warm-light.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## Essential Keyboard Shortcuts

<div class="table-wrapper">
<table>
  <thead>
    <tr>
      <th>Action</th>
      <th>macOS</th>
      <th>Windows</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>New Tab</strong></td>
      <td><code>Cmd + T</code></td>
      <td><code>Ctrl + T</code></td>
    </tr>
    <tr>
      <td><strong>Close Tab</strong></td>
      <td><code>Cmd + W</code></td>
      <td><code>Ctrl + W</code></td>
    </tr>
    <tr>
      <td><strong>Split Vertically</strong></td>
      <td><code>Cmd + D</code></td>
      <td><code>Ctrl + Shift + D</code></td>
    </tr>
    <tr>
      <td><strong>Split Horizontally</strong></td>
      <td><code>Cmd + Shift + D</code></td>
      <td><code>Ctrl + Shift + E</code></td>
    </tr>
    <tr>
      <td><strong>Toggle Vertical Tabs</strong></td>
      <td><code>Cmd + Shift + V</code></td>
      <td><code>Ctrl + Shift + V</code></td>
    </tr>
    <tr>
      <td><strong>Command Palette</strong></td>
      <td><code>Cmd + P</code></td>
      <td><code>Ctrl + P</code></td>
    </tr>
    <tr>
      <td><strong>Search Command History</strong></td>
      <td><code>Cmd + R</code></td>
      <td><code>Ctrl + R</code></td>
    </tr>
    <tr>
      <td><strong>Toggle Project Explorer</strong></td>
      <td><code>Cmd + B</code></td>
      <td><code>Ctrl + B</code></td>
    </tr>
    <tr>
      <td><strong>Settings</strong></td>
      <td><code>Cmd + ,</code></td>
      <td><code>Ctrl + ,</code></td>
    </tr>
  </tbody>
</table>
</div>
