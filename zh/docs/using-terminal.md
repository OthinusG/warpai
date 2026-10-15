---
layout: docs
lang: zh
title: 终端与命令块深入
description: "命令块原子操作、智能补全、垂直标签栏、项目文件树、Vim 编辑器、Markdown 实时渲染与 Git 差异审阅全流程实操指南。"
section: 使用指南
doc_id: using-terminal
prev_url: /docs/using.html
prev_title: 使用指南总览
next_url: /docs/using-warpai-agents.html
next_title: 多 Agent 协同工作流
---

本指南深入演示 Warpai 核心终端能力与集成开发者工具的日常实操技巧，聚焦**具体如何操作**：点击哪个按钮、按下哪些快捷键、以及如何最高效地流转。

---

## 第 1 部分：终端基础与命令块（Command Blocks）

不同于将输出视为无序滚屏纯文本流的传统终端，Warpai 将每一次命令及其执行结果封装为一个独立的原子化 **命令块（Block）**。

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ❯ git status                                                 [0.08s] [✓ 0]  │
│ On branch main                                                              │
│ Your branch is up to date with 'origin/main'.                               │
│ nothing to commit, working tree clean                                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1. 命令块导航与焦点切换
- **鼠标操作**：在终端历史中点击任意命令块即可将其选中，外围微光高亮边框即代表当前获得焦点。
- **键盘快捷键**：按 `Cmd + Up` / `Cmd + Down`（Windows 上为 `Ctrl + Up` / `Ctrl + Down`）在各个命令块之间快速跳转。

### 2. 复制命令输入 vs. 复制纯净输出
- **复制命令输入**：点击命令块右上角的剪贴板图标，或按下 `Cmd + Shift + C`（Windows 为 `Ctrl + Shift + C`）。仅复制原始命令文本（如 `git status`）。
- **复制纯净输出**：按下 `Cmd + Shift + O`（Windows 为 `Ctrl + Shift + O`）。只复制该命令产生的控制台输出内容——彻底剥离提示符、时间戳与 `❯` 符号，避免污染剪贴板。

### 3. 折叠与展开长输出
当执行类似 `npm install` 或 `cargo build` 等产生数百行日志的命令时：
- 点击命令块标题栏左侧的 **折叠箭头**（▼）。
- 或选中该块后按下 `Cmd + /`（Windows 为 `Ctrl + /`）。
- 输出即刻折叠为一行整洁的摘要，保持终端视口清爽。再次点击即可恢复展开。

<div class="docs-shot-placeholder" data-shot="SHOT-04">
  <div class="shot-header">
    <span class="shot-badge">SHOT-04 (P1)</span>
    <span class="shot-title">Atomic Command Blocks &amp; Actions</span>
  </div>
  <p class="shot-desc">Block 块化操作特写：包含执行成功的命令块 [✓ 0] 与耗时、包含超长输出被折叠的块、右上角悬停展示“复制输入 / 复制纯净输出”操作图标。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1200x600 (2:1)</span>
    <span>目标路径: docs/images/terminal-blocks-inspection.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 第 2 部分：智能参数补全与全局命令面板

Warpai 针对 Git、Docker、Cargo、NPM、Kubernetes 以及数百个常用 CLI 工具内置了智能参数签名库。

### 1. 使用智能补全
1. 开始键入命令（如 `git check`）。
2. 即时弹出交互式补全浮层，列出匹配的子命令（如 `checkout`、`check-ref-format` 等）及官方文档摘要。
3. 按 `Tab` 或 `方向键下` 选定目标项。
4. 按 `Enter` 或 `Tab` 完成自动填充。
5. 当键入参数开关时（如 `git commit -`），自动列出可用选项（`-m`、`--amend`、`-a` 等）及具体释义。

<div class="docs-shot-placeholder" data-shot="SHOT-05">
  <div class="shot-header">
    <span class="shot-badge">SHOT-05 (P1)</span>
    <span class="shot-title">Smart Autocompletion Popup</span>
  </div>
  <p class="shot-desc">智能参数补全浮层：输入 git commit - 时弹出的交互式补全列表，显示 -m, -a, --amend 等参数说明与官方文档高亮。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1000x500 (2:1)</span>
    <span>目标路径: docs/images/terminal-autocompletion-popup.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

### 2. 全局居中命令面板（`Cmd + P` / `Ctrl + P`）
1. 按下 `Cmd + P`（Windows 上为 `Ctrl + P`）。
2. 输入动作名称（如 `Split`、`Theme`、`Clear`、`New Tab` 等）。
3. 回车即刻执行该动作，全程双手不离键盘。

<div class="docs-shot-placeholder" data-shot="SHOT-06">
  <div class="shot-header">
    <span class="shot-badge">SHOT-06 (P1)</span>
    <span class="shot-title">Global Command Palette (Cmd+P)</span>
  </div>
  <p class="shot-desc">全局居中命令面板：按下 Cmd+P / Ctrl+P 弹出的动作搜索窗，展示切换主题、分屏、清屏等快捷操作的模糊匹配。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1000x550</span>
    <span>目标路径: docs/images/terminal-command-palette.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 第 3 部分：标签页与灵活分屏

针对宽屏显示器与多任务重度并行场景，Warpai 提供灵活的布局管理：

### 1. 垂直标签栏 vs. 水平标签栏
- **切换显示模式**：按 `Cmd + Shift + V`（Windows 为 `Ctrl + Shift + V`）在**垂直标签栏**（居于终端左侧）与传统顶部水平标签栏之间切换。
- **为何推崇垂直标签栏？**：在多 Git Worktree 与服务器会话并行时，水平标签很快会被挤压截断。垂直标签栏能完整显示分支名与工作目录，绝不折叠截断。
- **新建标签**：`Cmd + T`（Windows 为 `Ctrl + T`）。
- **关闭标签**：`Cmd + W`（Windows 为 `Ctrl + W`）。

### 2. 窗格分屏（Split Panes）
可在同一个标签页内自由划分为多个独立终端窗格：
- **垂直分屏（左右并排）**：按 `Cmd + D`（Windows 为 `Ctrl + Shift + D`）。
- **水平分屏（上下排布）**：按 `Cmd + Shift + D`（Windows 为 `Ctrl + Shift + E`）。
- **窗格间切换焦点**：按 `Cmd + Option + 方向键`（Windows 为 `Alt + 方向键`）。

<div class="docs-shot-placeholder" data-shot="SHOT-07">
  <div class="shot-header">
    <span class="shot-badge">SHOT-07 (P1)</span>
    <span class="shot-title">Vertical Tabs &amp; Split Panes</span>
  </div>
  <p class="shot-desc">垂直标签栏与分屏：左侧垂直展示多个展开分支名与标签页，中间切分为左右双窗格（一个窗格跑编译，另一个跑命令）。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/terminal-vertical-tabs-splitpanes.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 第 4 部分：项目文件浏览器（Project Explorer）

右侧面板提供与终端目录深度联动的完整工程文件树：

- **显隐项目文件树**：点击右上角工具栏的文件图标，或按快捷键 `Cmd + B`（Windows 为 `Ctrl + B`）。
- **右键操作**：支持新建文件/文件夹、重命名、复制相对路径与删除。
- **终端目录自联动**：终端中执行 `cd <路径>` 时，文件树会自动展开并高亮对应文件夹。

<div class="docs-shot-placeholder" data-shot="SHOT-08">
  <div class="shot-header">
    <span class="shot-badge">SHOT-08 (P1)</span>
    <span class="shot-title">Project Explorer &amp; Terminal Sync</span>
  </div>
  <p class="shot-desc">右侧项目文件树：展开清晰的文件目录树，展示右键菜单（新建文件/文件夹、重命名、删除），以及随终端 cd 自动联动高亮。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1200x800</span>
    <span>目标路径: docs/images/workbench-project-explorer.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 第 5 部分：内置轻量编辑器与 Vim 模态支持

在项目树中单击任意代码或文本文件，即可直接在终端内置编辑器中查看和编辑：
- **Vim 模式**：在设置中开启 Vim 键位支持后，按 `Esc` 进入 Normal 模式，通过 `h/j/k/l` 导航光标，输入 `:w` 保存，输入 `:q` 关闭。
- **语法高亮**：原生支持 Rust、Python、TypeScript、Go、C++、HTML、JSON、YAML、TOML 与 Markdown。

<div class="docs-shot-placeholder" data-shot="SHOT-09">
  <div class="shot-header">
    <span class="shot-badge">SHOT-09 (P1)</span>
    <span class="shot-title">Built-in Editor with Vim Mode</span>
  </div>
  <p class="shot-desc">轻量编辑器与 Vim 模式：在内置编辑器中打开代码文件，展现语法高亮、底栏显示 :w 保存状态及 Normal/Insert 模式光标。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1200x700</span>
    <span>目标路径: docs/images/workbench-editor-vim-mode.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 第 6 部分：Markdown 与图片实时渲染预览

在项目树中打开任意 `.md` 文件，切换至 **Preview（预览）** 标签：
- 原生渲染 GFM 格式表格、任务勾选框、高亮呼出框（Callouts）以及本地/远程图片。
- 当你的 Coding Agent 正在生成调研报告或技术文档时，右侧实时自动刷新渲染。

<div class="docs-shot-placeholder" data-shot="SHOT-10">
  <div class="shot-header">
    <span class="shot-badge">SHOT-10 (P1)</span>
    <span class="shot-title">Live Markdown &amp; Image Preview</span>
  </div>
  <p class="shot-desc">Markdown 实时富文本渲染：右侧 Preview 标签展示技术文档、GFM 排版、表格、代码高亮框及本地/远程图片就地渲染。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1200x750</span>
    <span>目标路径: docs/images/workbench-markdown-live-preview.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 第 7 部分：终端原生 Git 差异对比审阅

在执行提交前，就地查看工作区变更差异：
- 分组展示 **Staged Changes（已暂存）**、**Unstaged Changes（未暂存）** 与 **Untracked Files（未追踪）**。
- 双栏并排或统一 Diff 视图展示，绿色高亮新增代码，红色高亮删除内容。

<div class="docs-shot-placeholder" data-shot="SHOT-11">
  <div class="shot-header">
    <span class="shot-badge">SHOT-11 (P1)</span>
    <span class="shot-title">Terminal-Native Git Code Review</span>
  </div>
  <p class="shot-desc">内置 Git Code Review 面板：展示已暂存、未暂存、未追踪文件分类，并列展示双栏 Diff 对比（红绿行高亮与折叠行展开）。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/workbench-git-code-review.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>
