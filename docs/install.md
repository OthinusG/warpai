---
layout: docs
title: Installation & Setup
description: "Step-by-step installation instructions for macOS, Windows, Linux Companion, package managers, and security verifications."
section: Getting Started
doc_id: install
prev_url: /docs/
prev_title: Documentation Overview
next_url: /docs/quick-start.html
next_title: Quick Start (5 Min)
---

Warpai is a native desktop application built in Rust, targeting **macOS** and **Windows**, with first-class remote support for **Linux**, **macOS**, and **Windows** over SSH and Windows **WSL 2**.

---

## Desktop Installation

### 1. macOS (Apple Silicon & Intel)

- **System Requirements**: macOS 11.0 (Big Sur) or newer.
- **Direct Downloads**:
  - [Download Warpai for Apple Silicon (DMG)](https://github.com/OthinusG/warpai/releases/latest)
  - [Download Warpai for Intel x86_64 (DMG)](https://github.com/OthinusG/warpai/releases/latest)
- **Homebrew Cask**:
  ```bash
  brew install --cask warpai
  ```

#### Installation Steps:
1. Open the downloaded `.dmg` disk image.
2. Drag **Warpai.app** into your `/Applications` folder.
3. Launch **Warpai** from Launchpad or Spotlight.

<div class="docs-callout important">
  <div class="docs-callout-title">macOS Gatekeeper Note (Ad-hoc Signature)</div>
  <p>Warpai desktop builds are ad-hoc signed. On first launch, macOS may present a dialog saying <em>"Warpai cannot be opened because the developer cannot be verified"</em>.</p>
  <ul>
    <li><strong>Option A (System Settings)</strong>: Open <strong>System Settings > Privacy &amp; Security</strong>, scroll to Security, and click <strong>Open Anyway</strong>.</li>
    <li><strong>Option B (Terminal Command)</strong>: Run <code>xattr -d com.apple.quarantine /Applications/Warpai.app</code></li>
  </ul>
</div>

<div class="docs-shot-placeholder" data-shot="SHOT-22">
  <div class="shot-header">
    <span class="shot-badge">SHOT-22 (P2)</span>
    <span class="shot-title">macOS Gatekeeper 'Open Anyway'</span>
  </div>
  <p class="shot-desc">macOS 初次安装信任指引：展示系统设置“安全性与隐私”中点击“仍要打开”（Open Anyway）的操作界面截图。</p>
  <div class="shot-meta">
    <span>建议尺寸: 900x550</span>
    <span>目标路径: docs/images/macos-gatekeeper-open-anyway.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

### 2. Windows (x64)

- **System Requirements**: Windows 10 (version 1903+) or Windows 11; 64-bit x86 architecture.
- **Direct Download**:
  - [Download Warpai Windows Installer (.exe)](https://github.com/OthinusG/warpai/releases/latest)
- **Windows Package Manager (winget)**:
  ```powershell
  winget install Warpai.Warpai
  ```

#### Installation Steps:
1. Double-click the installer executable.
2. If Windows Defender SmartScreen displays *"Windows protected your PC"*, click **More info** followed by **Run anyway**.
3. Follow the wizard. A desktop shortcut and Start Menu entry will be created.

<div class="docs-shot-placeholder" data-shot="SHOT-23">
  <div class="shot-header">
    <span class="shot-badge">SHOT-23 (P2)</span>
    <span class="shot-title">Windows SmartScreen 'Run Anyway'</span>
  </div>
  <p class="shot-desc">Windows Defender SmartScreen 拦截指引：展示点击“更多信息”后出现的“仍要运行”按钮操作截图。</p>
  <div class="shot-meta">
    <span>建议尺寸: 800x450</span>
    <span>目标路径: docs/images/windows-smartscreen-run-anyway.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## Remote Workspaces & Companions

Warpai enables full terminal, file browsing, in-app editing, and agent collaboration across remote servers and containers without altering your remote environment, powered by the standalone **Warpai Companion** binary.

### Architecture Highlights:
- **Private Bundled Git**: Companion embeds its own private Git runtime. Your remote server **does not require** Git, Node.js, Python, tmux, or socat to be pre-installed.
- **Zero Background Resource Waste**: Companion starts on demand upon terminal connection and automatically terminates after 60 seconds of idle time.

<div class="table-wrapper">
<table>
  <thead>
    <tr>
      <th>Remote Platform</th>
      <th>Package</th>
      <th>Installation Command</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Linux x64</strong> (Ubuntu 20.04+, Debian, RHEL)</td>
      <td><a href="https://github.com/OthinusG/warpai/releases/latest">WarpaiCompanion-linux-x64.run</a></td>
      <td><code>chmod +x WarpaiCompanion-*.run &amp;&amp; ./WarpaiCompanion-*.run</code></td>
    </tr>
    <tr>
      <td><strong>macOS Apple Silicon</strong></td>
      <td><a href="https://github.com/OthinusG/warpai/releases/latest">WarpaiCompanion-macos-arm64.dmg</a></td>
      <td>Copy <code>warpai-companion</code> to <code>~/.config/warpai/bin/</code></td>
    </tr>
    <tr>
      <td><strong>Windows x64</strong></td>
      <td><a href="https://github.com/OthinusG/warpai/releases/latest">WarpaiCompanion-windows-x64-setup.exe</a></td>
      <td>Run setup installer on remote host.</td>
    </tr>
  </tbody>
</table>
</div>

<div class="docs-shot-placeholder" data-shot="SHOT-17">
  <div class="shot-header">
    <span class="shot-badge">SHOT-17 (P1)</span>
    <span class="shot-title">Remote SSH Project Workspace</span>
  </div>
  <p class="shot-desc">SSH 远程无缝漫游：终端 SSH 连入云服务器并 cd 进工程，右侧自动列出远程 SFTP 文件树，左侧联动接管远程 Linux 上的 Agent。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/remote-ssh-workspace.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## Windows WSL 2 Integration

Warpai on Windows integrates directly with WSL 2 distributions without running a separate network bridge or requiring third-party X11 servers.

<div class="docs-shot-placeholder" data-shot="SHOT-18">
  <div class="shot-header">
    <span class="shot-badge">SHOT-18 (P1)</span>
    <span class="shot-title">Windows WSL 2 Direct Integration</span>
  </div>
  <p class="shot-desc">Windows 下 WSL 2 深度集成：终端运行 Ubuntu bash 并启动 Linux CLI Agent，Windows 本地 GUI 完美编排调度。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/windows-wsl2-integration.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## Configuration & User Data Paths

Warpai maintains all durable data, settings, and caches locally on your machine:

- **macOS / Linux**: `~/.config/warpai/`
  - Preferences: `~/.config/warpai/settings.toml`
  - Agent Communication Cache: `~/.config/warpai/agent_bus/`
  - Local Database: `~/.config/warpai/warpai.sqlite`
- **Windows**: `%APPDATA%\Warpai\`
  - Preferences: `%APPDATA%\Warpai\settings.toml`
  - Local Database: `%APPDATA%\Warpai\warpai.sqlite`
