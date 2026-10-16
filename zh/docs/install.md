---
layout: docs
lang: zh
title: 安装指南
description: "Warpai 桌面端（macOS Apple 芯片、Windows x64、Linux 桌面端）、Linux 远程伴侣安装说明、安全性信任操作及环境配置。"
section: 快速入门
doc_id: install
prev_url: /docs/
prev_title: 文档概览
next_url: /docs/quick-start.html
next_title: 快速上手 (5分钟)
---

Warpai 是一款基于 Rust 构建的高性能纯原生桌面应用程序，专为 **macOS（Apple 芯片）**、**Windows（x64）** 与 **Linux 桌面端（x64）** 平台提供原生支持，同时通过轻量级远程伴侣（Companion）全面覆盖 SSH 远端 **Linux**、**macOS** 与 **Windows** 环境以及 Windows **WSL 2**。

---

## 桌面端安装

### 1. macOS (Apple 芯片)

- **系统要求**：macOS 11.0 (Big Sur) 或更高版本；Apple 芯片（M1 / M2 / M3 / M4 系列）。
- **Homebrew Cask (Tap 源)**：
  ```bash
  brew install --cask OthinusG/tap/warpai
  ```
- **安装包直链**：
  - [下载 Warpai macOS 版 (DMG)](https://github.com/OthinusG/warpai/releases/download/v1.7.0/Warpai-1.7.0-macos-arm64.dmg)

#### 安装步骤：
1. 双击打开下载的 `.dmg` 磁盘镜像文件。
2. 将 **Warpai.app** 拖拽至 `/Applications`（应用程序）文件夹中。
3. 从“启动台”或“聚焦搜索（Spotlight）”中启动 **Warpai**。

<div class="docs-callout important">
  <div class="docs-callout-title">macOS Gatekeeper 安全提示（临时签名）</div>
  <p>Warpai 桌面构建版本采用临时签名（Ad-hoc Signed），暂未加入苹果付费公证。在 macOS 上首次启动时，系统可能弹出提示：<em>“无法打开 Warpai，因为无法验证开发者”</em>。</p>
  <ul>
    <li><strong>方式 A（系统设置授权）</strong>：打开 <strong>系统设置 > 隐私与安全性</strong>，向下滑动至“安全性”小节，找到拦截提示并点击 <strong>“仍要打开”（Open Anyway）</strong>。</li>
    <li><strong>方式 B（终端命令清除隔离属性）</strong>：在现有终端中执行：<code>xattr -d com.apple.quarantine /Applications/Warpai.app</code></li>
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

- **系统要求**：Windows 10 (1903 及更高版本) 或 Windows 11；64 位 x86 处理器架构。
- **安装包直链**：
  - [下载 Warpai Windows 安装包 (.exe)](https://github.com/OthinusG/warpai/releases/download/v1.7.0/WarpaiSetup-1.7.0-windows-x64.exe)

#### 安装步骤：
1. 双击运行下载的 `WarpaiSetup-1.7.0-windows-x64.exe` 安装文件。
2. 若 Windows Defender SmartScreen 弹出拦截窗 <em>“Windows 已保护你的电脑”</em>，请点击 <strong>“更多信息”</strong>，随后点击 <strong>“仍要运行”</strong>。
3. 按照安装向导完成安装，桌面快捷方式与开始菜单将自动生成。

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

### 3. Linux 桌面端 (x64)

- **系统要求**：Ubuntu 20.04+、Debian 11+、Fedora 38+、Arch Linux 等主流发行版；64 位 x86 架构；支持 Wayland 与 X11 显示环境。
- **安装包直链**：
  - [下载 Warpai Linux AppImage 镜像](https://github.com/OthinusG/warpai/releases/download/v1.7.0/Warpai-1.7.0-linux-x86_64.AppImage)
  - [下载 Warpai Debian / Ubuntu 安装包 (.deb)](https://github.com/OthinusG/warpai/releases/download/v1.7.0/warpai_1.7.0_amd64.deb)
  - [下载独立可执行压缩包 (.tar.gz)](https://github.com/OthinusG/warpai/releases/download/v1.7.0/warpai-1.7.0-linux-x86_64.tar.gz)

#### 安装与运行步骤：

- **AppImage（免安装通用格式）**：
  ```bash
  chmod +x Warpai-1.7.0-linux-x86_64.AppImage
  ./Warpai-1.7.0-linux-x86_64.AppImage
  ```
- **Debian / Ubuntu 安装包 (.deb)**：
  ```bash
  sudo dpkg -i warpai_1.7.0_amd64.deb || sudo apt-get install -f
  ```
- **独立解压包**：
  ```bash
  tar -xzf warpai-1.7.0-linux-x86_64.tar.gz
  ./warpai/warpai
  ```

<div class="docs-shot-placeholder" data-shot="SHOT-24">
  <div class="shot-header">
    <span class="shot-badge">SHOT-24 (P2)</span>
    <span class="shot-title">Linux 桌面端原生界面</span>
  </div>
  <p class="shot-desc">Linux 桌面端原生界面：在 Linux（Ubuntu GNOME/Debian/Fedora）桌面运行 Warpai 原生应用，展示原生窗口阴影、垂直标签栏及 GPU 渲染终端命令行。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/linux-desktop-native.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 远程工作区与 Companion 伴侣

Warpai 支持在不污染和改动远程服务器环境的前提下，无缝漫游连接远程 Linux/macOS/Windows 节点，享受完整终端、SFTP 文件树与 Agent 协同。这一切由独立的 **Warpai Companion** 伴侣进程提供驱动。

### 架构特色：
- **自带私有 Git 运行时**：Companion 自带精简版内部 Git 实现，远端宿主机**无需**预装 Git、Node.js、Python、tmux 或 socat。
- **按需唤醒，零常驻损耗**：当终端连接时自动拉起，连接断开后空闲 60 秒自动退出，不消耗远端系统常驻内存。

<div class="table-wrapper">
<table>
  <thead>
    <tr>
      <th>远端平台</th>
      <th>安装包下载</th>
      <th>安装与部署说明</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Linux x64</strong> (Ubuntu 20.04+, Debian, RHEL)</td>
      <td><a href="https://github.com/OthinusG/warpai/releases/latest">WarpaiCompanion-linux-x64.run</a></td>
      <td><code>chmod +x WarpaiCompanion-*.run &amp;&amp; ./WarpaiCompanion-*.run</code></td>
    </tr>
    <tr>
      <td><strong>macOS Apple 芯片</strong></td>
      <td><a href="https://github.com/OthinusG/warpai/releases/latest">WarpaiCompanion-macos-arm64.dmg</a></td>
      <td>将 <code>warpai-companion</code> 二进制复制到 <code>~/.config/warpai/bin/</code></td>
    </tr>
    <tr>
      <td><strong>Windows x64</strong></td>
      <td><a href="https://github.com/OthinusG/warpai/releases/latest">WarpaiCompanion-windows-x64-setup.exe</a></td>
      <td>在远端 Windows 机器上双击运行安装向导。</td>
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

## Windows WSL 2 深度集成

在 Windows 平台上，Warpai 通过标准 stdio 管道直连本地 WSL 2 发行版，无需配置虚拟网络代理桥接，亦无需安装第三方 X11 服务。

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

## 配置文件与用户数据存储路径

Warpai 严格遵循本地优先设计，所有持久化数据、配置与缓存均保存在本机：

- **macOS / Linux**：`~/.config/warpai/`
  - 核心偏好配置：`~/.config/warpai/settings.toml`
  - Agent 通信总线管道：`~/.config/warpai/agent_bus/`
  - 本地 SQLite 数据库：`~/.config/warpai/warpai.sqlite`
- **Windows**：`%APPDATA%\Warpai\`
  - 核心偏好配置：`%APPDATA%\Warpai\settings.toml`
  - 本地 SQLite 数据库：`%APPDATA%\Warpai\warpai.sqlite`
