---
layout: docs
lang: zh
title: 核心概念与安全保障
description: "深入理解 Warpai 的底层设计哲学：将 Agent 作为基本工作单元、项目与工作树双模式以及独家守护式 PTY 交付。"
section: 核心架构
doc_id: concepts
prev_url: /docs/quick-start.html
prev_title: 快速上手 (5分钟)
next_url: /docs/features.html
next_title: 全景特性与原版对比
---

深入理解 Warpai 的底层设计哲学与架构原理，将帮助你充分释放多 Agent 自主协同开发的巨大威力。

---

## 1. 将 Agent 会话作为基本工作单元

在传统 IDE 与以对话为核心的辅助工具中，AI 通常被局限在一个悬浮聊天窗或侧边栏里。每次请求都是孤立的单次问答，人类开发者依然充当着搬运代码、切换文件并人工跑测试的“人工路由器”。

**Warpai 确立了以 Agent 会话为第一公民的工程工作单元**：
- 每个 Agent 拥有独立的操作系统进程、终端上下文、工作目录与环境变量。
- Agent 可以接收结构化的任务委派，包含明确的交付标准与验收边界。
- Agent 可以主动上报进度、向其他 Agent 咨询澄清、发起代码交叉评审（Peer Review），并提交经过自测验证的代码补丁。
- Warpai 提供了底层终端底座与进程通信总线（Agent Bus），使多个 Agent 能够像一支配合默契的真实研发小组一样自主协作。

---

## 2. 双协作模式：项目模式 vs. 工作树模式

在左侧协同面板顶部，用户可随时在两种互为补充的协作模式间自由切换：

```
┌─────────────────────────────────┬─────────────────────────────────┐
│            项目模式             │           工作树模式            │
│         (PROJECT MODE)          │         (WORKTREE MODE)         │
│                                 │                                 │
│ • 平等对等通信网络              │ • 层次化 Coordinator 调度架构   │
│ • 无需 Git 代码仓库             │ • 基于 Git 仓库工作树           │
│ • 共享单项目工作目录            │ • 物理隔离的独立分支目录        │
│ • 适用场景：文献调研、设计方案、│ • 适用场景：功能并行开发、      │
│   技术写作、数据分析            │   代码重构、测试套件编写        │
└─────────────────────────────────┴─────────────────────────────────┘
```

### 项目模式（共享调研与方案推演）
在 **项目模式（Project Mode）** 中，当前目录下的所有活跃 Agent 处于同一个平等的协作网络中：
- Agent 之间可以直接发起对话、交换数据与委派子任务。
- 任务池、上下文资源与长效项目记忆对所有 Agent 公开可见。
- 适用于任意普通文件夹（无论是否为 Git 仓库），是撰写调研报告、架构技术规范与头脑风暴的理想载体。

### 工作树模式（大规模并行软件工程）
在实际项目开发中，让多个 Agent 同时读写同一份物理文件，必然引发文件冲突、编译撞车与上下文污染。

**工作树模式（Worktree Mode）借助 Git 原生 Worktree 彻底根治了这一难题**：
- 为每个功能模块、缺陷修复或测试任务创建独立的物理工作树（Worktree）目录。
- 各 Agent 在完全隔离的物理副本中并行编码，Worktree A 的实验性修改绝不会破坏 Worktree B 的编译与测试。
- 由一名被指定为 **Coordinator（协调者）** 的核心 Agent 统筹全局：拆解任务分配给各个 Worktree 中的 Agent，并在提交后组织对等评审，最终将验收通过的代码合并回主干。

---

## 3. 屏幕状态机与生命周期感知

命令行 Agent 是运行在伪终端（PTY）上的字符界面应用。为了在不入侵篡改 Agent 源码的前提下实现全自动感知与调度，Warpai 打造了专有的 **VT100 内存屏幕观察器**。

通过内置的 Agent 行为描述清单（`crates/agent_bus/src/screen_detection/manifests/*.toml`），Warpai 实时采样终端字符流，精确收窄出 Agent 的瞬时生命周期状态：

<div class="table-wrapper">
<table>
  <thead>
    <tr>
      <th>生命周期状态</th>
      <th>状态含义</th>
      <th>Warpai 处理动作</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong><code>Working</code>（执行中）</strong></td>
      <td>Agent 正在推理、运行 Shell 命令或调用模型接口。</td>
      <td>将外部发来的新消息与任务安全暂存入该 Agent 的专属收件箱中。</td>
    </tr>
    <tr>
      <td><strong><code>Idle</code>（空闲就绪）</strong></td>
      <td>Agent 本轮交互已完结，终端回到提示符等待新指令。</td>
      <td>激活 <strong>守护式 PTY 交付引擎</strong>，自动平滑投递队列中的待办任务。</td>
    </tr>
    <tr>
      <td><strong><code>WaitingApproval</code>（等待审批）</strong></td>
      <td>Agent 触发了敏感权限请求（如高危删除、修改文件、联网等）。</td>
      <td>暂停自动化投递，标签页亮起珊瑚红光，提醒人类开发者及时介入审批。</td>
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

## 4. 独家守护式 PTY 交互交付

与粗暴地向管道写入字符的简单脚本不同，Warpai 贯彻严格的 **守护式 PTY 交付（Guarded PTY Delivery）**：

1. **提示符精准同步**：仅当 VT100 解析器严格确认终端处于静默、完整且活跃的输入提示符状态时，才会注入字符。
2. **草稿防冲撞机制**：若开发者已在提示符中手动输入了草稿内容，自动化投递将立即暂缓，绝不会覆盖你的输入或提前发送残缺指令。
3. **权限确认优先**：当检测到 Agent 等待用户确认时，强制锁定为交互模式，直到人类完成批准或驳回。

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
