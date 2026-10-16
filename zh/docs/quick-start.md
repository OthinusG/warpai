---
layout: docs
lang: zh
title: 快速上手 (5分钟)
description: "5 分钟上手 Warpai：启动首个受协同管理的 CLI Agent、体验全自动终端状态感知与自动化任务排队投递。"
section: 快速入门
doc_id: quick-start
prev_url: /docs/install.html
prev_title: 安装指南
next_url: /docs/concepts.html
next_title: 核心概念与安全保障
---

在 5 分钟内快速熟悉 Warpai。学习如何启动受协同管理的 CLI Agent、观察全自动终端状态感知，并探索集成的三栏开发者工作台。

---

## 第 1 步：认识三栏式工作空间

启动 Warpai 时，你将看到一个高度整合的三栏桌面工作台：

```
┌──────────────────┬──────────────────────────────────────────┬──────────────────┐
│  左侧协同面板    │              中央 GPU 终端               │  右侧项目面板    │
│                  │                                          │                  │
│ • 模式切换       │ • GPU 硬件加速命令块渲染                 │ • 文件浏览器     │
│ • Coordinator    │ • 智能命令补全与执行历史                 │ • 轻量代码编辑器 │
│ • Agent 会话列表 │ • 垂直标签页树形排布                     │   (支持 Vim 模式)│
│ • 活跃任务池     │ • VT100 终端生命周期感知                 │ • 实时 Markdown  │
│ • 额度监控看板   │                                          │ • Git 差异审阅   │
└──────────────────┴──────────────────────────────────────────┴──────────────────┘
```

1. **左侧面板（Agent 协作与调度）**：管理多 Agent 协作会话、在**项目模式（Project：非 Coding 泛行业全向协同）**与**工作树模式（Worktree：Git 多分支并行代码开发）**间切换、分派任务、审阅 Peer Review 成果，并实时查看模型用量额度。
2. **中央面板（GPU 加速终端）**：基于 `wgpu` 构建的现代化高性能终端，支持命令块（Block）化执行、智能补全、垂直标签栏与分屏。
3. **右侧面板（项目工作台）**：内置项目文件树、支持 Vim 键位的轻量源码编辑器、Markdown 文档实时渲染预览器以及 Git Code Review 差异对比。

---

## 第 2 步：开启 Agent 协同支持

Warpai 与你本机已安装的各类命令行 Agent 工具实行原生桥接：

1. 打开 **设置（Settings）**（macOS 上按 `Cmd + ,`，Windows 上按 `Ctrl + ,`）。
2. 前往 **功能（Features） > Agent communication**。
3. 将 **Agent communication** 开关切换为 **开启（ON）**。
4. Warpai 会自动扫描系统 `PATH` 并列出识别到的 CLI Agent（如 `Claude Code`、`Codex`、`Gemini`、`Cursor`、`Antigravity`、`Trae` 等）。
5. 勾选你本机已安装并希望纳入协同管理的 Agent。

<div class="docs-callout tip">
  <div class="docs-callout-title">绝不污染全局系统</div>
  <p>勾选 Agent 仅激活 Warpai 的本地内存桥接层。它<strong>绝不会篡改</strong>你的全局 Shell 配置（如 <code>~/.zshrc</code>）、系统 PATH 或第三方 Agent 的配置文件。</p>
</div>

---

## 第 3 步：启动你的首个协同 Agent

1. 鼠标点击进入中央终端区域。
2. 直接运行你习惯的 CLI Agent 启动命令：
   ```bash
   # 启动 Claude Code
   claude

   # 或启动 Codex
   codex

   # 或启动 Google Antigravity
   agy
   ```
3. **观察左侧面板**：Warpai 会即刻识别出进程启动。协同面板中将实时出现该 Agent 的专属行，展示其品牌图标、进程 ID 与运行状态。

---

## 第 4 步：全自动屏幕状态感知与任务排队投递

Warpai 内置的高性能 VT100 屏幕观察器在内存中解析 ANSI 流，实时识别 Agent 状态：

- **`Working`（执行中）**：Agent 正在思考、生成代码、调用工具或执行测试。
- **`Idle`（空闲就绪）**：Agent 本轮交互已结束，停留在命令行提示符等待下一条指令。
- **`WaitingApproval`（等待审批）**：Agent 触发了敏感系统权限请求（如执行写入命令），已暂停等待你的手动确认。

### 体验全自动任务投递：
1. 当 Agent 处于忙碌状态（`Working`）时，分配给它的新任务与后续消息会安全暂存在专属收件箱中。
2. 一旦 Agent 任务完成并恢复至 `Idle`，Warpai 的**守护式 PTY 交付引擎**会自动将排队任务输入到终端提示符中。
3. **动态监督机制**：任务可随时向多个 Agent 动态派发；新发布任务自动加入监督列表全程追踪，任务完成后主动退出列表。
4. **无需手动频繁复制粘贴，更无需在多个终端窗口间来回切换排查！**

<div class="docs-callout note">
  <div class="docs-callout-title">未提交草稿防护</div>
  <p>若你在终端提示符中已输入但尚未回车提交的草稿文本，或 Agent 正处于等待授权确认（<code>WaitingApproval</code>）状态，Warpai 将自动暂停自动化投递，确保不冲撞你的输入与交互安全。</p>
</div>

---

## 第 5 步：实时监控 API 额度与 Token 消耗

在长流程复杂重构中，避免因意外触发供应商 Rate Limit 而导致中断：

1. 观察左侧协作面板底部的 **用量大盘（Data Usage）**。
2. 实时展示已连接供应商的 Quota 消耗百分比进度条、5小时/周重置倒计时以及实时余额。

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
