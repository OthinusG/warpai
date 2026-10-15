---
layout: docs
lang: zh
title: 全景功能特性与原版对比
description: "Warpai 功能特性全景矩阵、零遥测纯净架构说明以及与上游 Warp 和传统终端模拟器的多维度深度对比。"
section: 核心架构
doc_id: features
prev_url: /docs/concepts.html
prev_title: 核心概念与安全保障
next_url: /docs/using.html
next_title: 使用指南总览
---

Warpai 是一款专为自主 Coding Agent 协作打造的本地优先、零隐私侵犯现代终端。本文档提供详尽的功能全景清单、与上游 Warp 原版的架构差异对比，以及相比传统终端模拟器的核心优势。

---

## 1. 上游 Warp 原版 vs. Warpai 深度对比

<div class="table-wrapper">
<table>
  <thead>
    <tr>
      <th>对比维度</th>
      <th>Warp 原版</th>
      <th>Warpai</th>
      <th>用户收益与核心差异</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>云端依赖度</strong></td>
      <td>强依赖 Warp Cloud 服务端与持续网络连接。</td>
      <td><strong>100% 纯本地优先 (Local-First)</strong></td>
      <td>无单点故障风险，完全离线与内网隔离环境均可自如运行。</td>
    </tr>
    <tr>
      <td><strong>账号与登录限制</strong></td>
      <td>强制注册账号与登录墙验证。</td>
      <td><strong>零登录墙 (Zero Login Gate)</strong></td>
      <td>开箱即用，无需提供邮箱、密码或注册任何云端账户。</td>
    </tr>
    <tr>
      <td><strong>遥测埋点与隐私</strong></td>
      <td>内置 Sentry 崩溃采集、用量打点与分析上报。</td>
      <td><strong>零遥测埋点 (0 KB)</strong></td>
      <td>彻底剥离全部埋点与追踪代码。命令、输出与源码绝不离开本机。</td>
    </tr>
    <tr>
      <td><strong>开源协议与商业模式</strong></td>
      <td>专有商业软件，按月订阅计费与分级额度限制。</td>
      <td><strong>自由开源 (AGPL-3.0)</strong></td>
      <td>代码完全开源透明，无商业付费墙或人工功能节流。</td>
    </tr>
    <tr>
      <td><strong>AI 生态架构</strong></td>
      <td>仅绑定其自带的单一专有云端 AI 助手。</td>
      <td><strong>开放生态 (全面支持 20+ 款 CLI Agent)</strong></td>
      <td>自由使用 Claude Code、Codex、Gemini、Cursor、Antigravity 等，沿用自有 API Key。</td>
    </tr>
    <tr>
      <td><strong>多 Agent 编排协同</strong></td>
      <td>仅支持单聊对话侧边栏。</td>
      <td><strong>双协同模式 (项目模式 / 工作树模式)</strong></td>
      <td>支持多 Agent 跨物理分支并行开发，全自动任务分派与交叉审阅。</td>
    </tr>
    <tr>
      <td><strong>终端屏幕感知</strong></td>
      <td>对外部终端 CLI 进程无任何状态感知。</td>
      <td><strong>VT100 内存屏幕观察器</strong></td>
      <td>实时精确解析 <code>Working</code>、<code>Idle</code> 与 <code>WaitingApproval</code> 生命周期。</td>
    </tr>
    <tr>
      <td><strong>自动化任务投递</strong></td>
      <td>需在多窗口间频繁手动复制粘贴。</td>
      <td><strong>守护式 PTY 交付 (Guarded PTY Delivery)</strong></td>
      <td>排队任务在 Agent 空闲处于安全提示符时自动静默投递，无需人工看守。</td>
    </tr>
    <tr>
      <td><strong>协同协议与工具集</strong></td>
      <td>私有封闭通信协议。</td>
      <td><strong>38+ 项原生 MCP 工具集</strong></td>
      <td>基于通用 Model Context Protocol 协议规范，覆盖任务生命周期、文件并发锁定与上下文记忆。</td>
    </tr>
    <tr>
      <td><strong>开发者综合工作台</strong></td>
      <td>仅为单一终端界面。</td>
      <td><strong>终端 + 项目树 + 内置轻量编辑器 + Git 审阅</strong></td>
      <td>内置文件浏览器、支持 Vim 键位的高性能编辑器、Markdown 渲染预览与差异审阅。</td>
    </tr>
    <tr>
      <td><strong>远端服务器支持</strong></td>
      <td>仅提供标准基础 SSH 终端连接。</td>
      <td><strong>自包含轻量 Companion + WSL 2</strong></td>
      <td>远程伴侣内置私有 Git 运行时，远端服务器无需安装 Node.js、Python 或环境依赖。</td>
    </tr>
    <tr>
      <td><strong>额度与余额监控</strong></td>
      <td>仅能查询官方 Warp 账户额度。</td>
      <td><strong>多供应商实时用量大盘</strong></td>
      <td>多模型实时用量进度条，Claude、OpenAI、Gemini、Cursor 5小时/周重置窗口一目了然。</td>
    </tr>
  </tbody>
</table>
</div>

---

## 2. 功能特性分级矩阵

### 第一梯队：核心杀手级差异化特性

#### 1. 原生支持并管理 20+ 款命令行 AI Agent
- 深度适配并自动感知生命周期状态：**Claude Code、Codex、Gemini CLI、Cursor CLI、Antigravity (agy)、Qoder、QoderCN、Trae、Mistral Vibe、Goose、Hermes、DeepSeek Harness (dsh-tui)、Grok、Copilot CLI、Pi、oh-my-pi (omp)、OpenCode、Amp、Auggie、Droid**。
- 支持通过 TOML 清单灵活扩展任意第三方命令行终端 Agent。
- **绝不污染全局系统**：零侵入，不改动 `~/.zshrc`、`PATH` 或 Agent 个人全局配置文件。

#### 2. 双协作模式：项目模式 vs. 工作树模式
- **项目模式**：面向设计推演、学术调研、规范编写与架构评审的平等网络通信模式，不依赖 Git 仓库。
- **工作树模式**：利用 Git 原生 Worktree 将任务物理拆分至独立目录，各 Agent 在独立物理分支并行写代码，彻底杜绝文件锁冲突与依赖脏化。
- **Coordinator 架构模式**：指派领队 Agent 统一拆解需求、派发子任务给各个 Agent、组织对等评审（Peer Review），并验收整合后提交干净的 Git Commit。

#### 3. 屏幕状态机与独家守护式 PTY 交付
- 专有 VT100 解析器实时监听终端 ANSI 字符流，准确识别 `Working`（执行中）、`Idle`（就绪）与 `WaitingApproval`（等待审批）三态。
- **守护式 PTY 交付**：排队任务在 Agent 恢复就绪且提示符安全时自动投递。检测到用户未提交草稿或敏感等待审批时自动暂停，杜绝命令冲撞。

#### 4. 38+ 原生 MCP 协同工具（`warp-lite-communication`）
- 完整任务生命周期控制（`warp_task_assign`、`warp_task_claim`、`warp_task_progress`、`warp_task_submit`、`warp_task_review`、`warp_task_integrate`）。
- 智能体点对点通信（`warp_agent_send`、`warp_agent_ask`、`warp_agent_reply`、`warp_agent_inbox`、`warp_agent_ready`）。
- 项目全局上下文与长期记忆（`warp_context_publish`、`warp_project_bootstrap`）。
- 协同开发文件独占锁预定（`warp_file_reserve`、`warp_file_release`）。

---

### 第二梯队：现代化 GPU 硬件加速终端核心

- **高性能 GPU 渲染**：基于 Rust `wgpu` 打造，在 macOS 上调用 Apple Metal，在 Windows 上调用 DirectX 12 与 Vulkan。
- **原子级命令块（Command Blocks）**：每条命令及其输出均封装为独立的交互卡片，支持独立复制输入、导出纯净输出、块内过滤搜索以及折叠长输出。
- **智能交互式补全**：内置常用 CLI 命令签名库，键入时即时弹出参数描述与语法帮助。
- **全局命令面板（Command Palette）**：`Cmd + P`（macOS）/ `Ctrl + P`（Windows）一键模糊搜索操作、配置与标签页。
- **垂直标签栏与分屏**：左右/上下分屏排布，支持垂直标签树，适应超宽屏与多任务并行。

---

### 第三梯队：开箱即用的终端内置开发者工作台

- **项目文件浏览器（Project Explorer）**：树状文件导航与当前终端目录实时联动，支持新建、重命名、删除与相对路径复制。
- **轻量级源码编辑器**：语法高亮、自动折行，原生支持标准 Vim 模态编辑快捷键（`:w`、`:q`、普通模式与插入模式）。
- **实时 Markdown 与图片渲染预览**：直接在终端侧边渲染 Markdown 格式表格、公式、任务清单以及本地/远端图片。
- **终端原生 Git 差异审阅（Git Code Review）**：提供双栏并排与统一 Diff 视图，语法高亮显示增删代码段落。
