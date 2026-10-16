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
next_title: 全景功能矩阵与市面体系对比
---

深入理解 Warpai 的底层设计哲学与架构原理，将帮助你充分释放多 Agent 自主协同开发的巨大威力。

---

## 1. 将 Agent 会话作为基本工作单元

在传统 IDE 与以对话为核心的辅助工具中，AI 通常被局限在一个悬浮聊天窗或侧边栏里。每次请求都是孤立的单次问答，人类开发者依然充当着搬运代码、切换文件并人工跑测试的“人工路由器”。

**Warpai 确立了以 Agent 会话为第一公民的工程工作单元**：
- 每个 Agent 拥有独立的操作系统进程、终端上下文、工作目录与环境变量。
- Agent 可以随时接收动态派发的结构化任务工单，包含明确的交付标准与验收边界。
- 新派发任务自动加入主 Agent 的动态监督任务列表进行全程追踪，并在任务完成并确认结果后主动退出列表。
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
│ • 全向网状互通架构              │ • 协调者统筹与分支审查架构      │
│ • 所有 Agent 兼具指挥官/工人身份│ • 专为 Git 代码仓库深度定制     │
│ • 任意 Agent 之间互发消息与工单 │ • 物理隔离的独立分支工作树      │
│ • 无需 Git 仓库，普通目录即开即用│ • 杜绝编译缓存污染与锁冲突      │
│ • 重点扶持泛行业与非 Coding 领域│ • 适用场景：功能特性并发开发、  │
│   (科研论文、设计、金融、专利等)│   大规模重构、回归测试集成      │
└─────────────────────────────────┴─────────────────────────────────┘
```

### 项目模式（全向协同与泛行业赋能）
一般的多 Agent 工具往往将 Git Worktree 作为唯一的并行开发单元，这在纯代码开发中表现优秀，但无法直接套用到科研写作、视觉设计、量化金融、专利法务等非 Coding 场景。

为此，Warpai 倾力扶持泛行业应用，重磅打造 **项目模式（Project Mode）**：
- **打破固定主从链条**：所有 Agent 既可以作为指挥官（Commander）统筹下发任务，也可以作为工人（Worker）认领攻坚，角色随需转换。
- **全向点对点通信与任务流转**：任何 Agent 之间均可直接发送消息、发起结构化问答、从公共任务池认领工单。
- **无需 Git 代码仓库**：在普通文件夹或网盘目录中即开即用，直接协同处理 Markdown、DOCX、PDF、CSV、PNG 或 SVG 素材。
- **广泛覆盖六大泛行业场景**：深度赋能科研论文撰写与顶刊同行预审、工业视觉与多媒体设计、量化投研与金融分析、专利交底挖掘与法律合规、自媒体脚本与矩阵运营、商业尽调与战略情报分析等。

### 工作树模式（大规模并行软件工程）
在严肃的 Git 软件开发中，多个 Agent 同时读写同一分支必然引发文件冲突、编译器缓存踩踏与依赖混乱。

**工作树模式（Worktree Mode）借助 Git 原生 Worktree 彻底根治了这一难题**：
- 为每个功能特性分配独立的物理 Git Worktree 目录，每个 Worktree 可以完全独立、并行地开发不同的业务功能特性。
- 各 Agent 在完全隔离的物理副本中并行编码，Worktree A 的实验性修改绝不会破坏 Worktree B 的编译与测试缓存。
- 由一名被指定为 **Coordinator（协调者）** 的核心 Agent 统筹全局：拆解任务分配给各个 Worktree 中的 Agent，并在提交后组织自动化对等评审（Peer Review），最终将验收通过的代码合并回主干。
- **详尽操作与案例手册**：请参阅 [Project 模式：全向协同与泛行业实战手册]({{ '/zh/docs/project-mode.html' | relative_url }}) 以及 [Worktree 编码模式：Git 并行代码开发完全手册]({{ '/zh/docs/worktree-mode.html' | relative_url }})。

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
      <td>暂停自动化投递，状态标签页切换为 WaitingApproval 标识并触发系统桌面通知，提醒人类开发者及时介入审批。</td>
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
