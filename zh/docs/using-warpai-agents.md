---
layout: docs
lang: zh
title: 多 Agent 协同工作流
description: "深入剖析 Warpai 双协同架构：面向 Git 仓库的 Worktree 并行编码模式与面向泛领域的 Project 全向协同模式，涵盖 Commander/Worker 角色互换、跨 Agent 消息流转、人机状态感知与 6+ 大非 Coding 实战场景指南。"
section: 使用指南
doc_id: using-warpai-agents
prev_url: /docs/using-terminal.html
prev_title: 终端与命令块深入
next_url: /docs/cli-agents.html
next_title: 20+ 款 CLI Agent 配置指南
---

Warpai 将独立的 Agent CLI 会话提升为原生第一公民工作单元。无论你在多少个项目中启动多少个 Agent，每个 Agent 都独享原生的物理终端通道。

在协同架构上，业内大多数多 Agent 工具通常将 **Git Worktree** 作为唯一的并行单元，这在纯代码开发中表现优异，但面对日益普及的**非 Coding 泛行业场景**（如科学研究、视觉设计、金融量化、专利法务、内容运营等），纯 Git 分支模型往往显得过于死板。为此，Warpai 独创了**双协同架构**：

1. **Git Worktree 并发编码模式（Worktree Coding Mode）**：面向 Git 仓库的严谨软件研发。除协调者（Coordinator）负责顶层规划与主干合并外，每个 Worktree 可以完全独立、并行地开发不同的业务功能特性，物理分支与编译器缓存彻底解耦隔离。
2. **Project 全向协同模式（Project Omnidirectional Mode）**：面向非 Coding 及跨学科通用协作场景重点扶持！无需 Git 仓库即可在任意工作目录下启动。打破单一树状指挥链，**网络拓扑内的所有 Agent 均可作为指挥官（Commander）或工人（Worker），所有 Agent 均可双向互通消息、主动提问答疑、发布全局上下文契约并从公共任务池自由认领任务**。

---

## 第 1 部分：启用 Agent 通信总线

在开展多智能体协同前，需激活本地轻量级通信总线（Agent Communication Bus）：

1. 打开 **设置（Settings）**（macOS 上按 `Cmd + ,`，Windows 上按 `Ctrl + ,`）。
2. 进入 **功能（Features） > Agent communication**。
3. 将 **Agent communication** 切换为 **开启（ON）**。
4. 检查已识别到的 CLI Agent 清单（如 `Claude Code`、`Codex`、`Gemini CLI`、`Antigravity`、`Cursor Agent`、`Trae` 等），确保正在使用的 Agent 处于激活状态。
5. 关闭设置，新建终端标签页（`Cmd + T` / `Ctrl + T`），通信环境变量（`WARP_AGENT_ENDPOINT`）将自动注入新会话。

---

## 第 2 部分：Agent 进程运行、状态感知与通知系统

### 1. 启动原生 Agent 进程
在任意终端窗格中，直接运行对应的标准 CLI 命令：
```bash
claude   # 启动 Anthropic Claude Code
codex    # 启动 OpenAI Codex
agy      # 启动 Google Antigravity
gemini   # 启动 Google Gemini CLI
```

### 2. 内存级 VT100 状态机感知
Warpai 内置零损耗 VT100 ANSI 内存流解析器，无需任何外置 Hook 即可毫秒级感知每个 Agent 终端的真实运行生命周期：

- 🟢 **`Idle`（空闲就绪）**：Agent 当前轮次交互已完结，正在命令行提示符处稳定等待新指令。守护式 PTY 将在此状态下安全交付待办任务。
- 🟡 **`Working`（执行中）**：Agent 正在进行 LLM 深度思考、代码生成、调用本地工具或执行 Shell 脚本。
- 🔴 **`WaitingApproval`（等待审批）**：Agent 遇到了安全隔离防线（例如请求删除文件、写敏感配置、执行高危系统指令或网络访问）。

### 3. 状态 Tab 标签页指示与系统通知（告别传统盲猜）
普通多窗格终端在多 Agent 并行时，用户必须逐个切屏检查哪一个 Agent 卡在了确认步骤。

在 Warpai 中：
- 当任意 Agent 进入 `WaitingApproval` 状态时，该终端对应的**垂直标签页（Tab）与侧边栏成员卡片会即时切换为高对比度的 WaitingApproval 状态标记**。
- 同时，Warpai 会即时向操作系统推送**系统桌面通知**（Native Desktop Notification），标明停滞的 Agent 名称、工作区与请求操作。
- 人类开发者点击通知即可直接跳转至对应窗格，按回车单次放行或按 `Esc` 拒绝，彻底释放注意力。

<div class="docs-shot-placeholder" data-shot="SHOT-13">
  <div class="shot-header">
    <span class="shot-badge">SHOT-13 (P1)</span>
    <span class="shot-title">Human-in-the-Loop Approval Prompt</span>
  </div>
  <p class="shot-desc">人机审批拦截交互：终端内 Agent 触发高危操作提示确认，状态 Tab 标签页更新为 WaitingApproval 指示，并同步触发系统桌面通知。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1000x500 (2:1)</span>
    <span>目标路径: docs/images/agent-waiting-approval-terminal.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 第 3 部分：Project 模式——全向网状协同与完整实战指南（核心重点）

针对非 Coding 与跨学科复合场景，Warpai 重点扶持并提供了完整的 **Project 模式**。

### 1. 核心工作机制：全向互通网状网络（Omnidirectional Mesh）

不同于以 Git 提交和分支为核心的约束模型，Project 模式直接运作于项目工作区（甚至普通的本地目录或网络共享盘）。

```
                    ┌──────────────────────────────┐
                    │    全局上下文契约 & 任务池    │
                    │   (warp_context / task_pool) │
                    └──────┬───────────────┬───────┘
                           │               │
            ┌──────────────┴───┐       ┌───┴──────────────┐
            │                  │       │                  │
      ┌─────▼──────┐     消息与任务互通    ┌─────▼──────┐
      │  Agent A   │ ◄────────────────► │  Agent B   │
      │(Commander) │                    │  (Worker)  │
      └─────▲──────┘                    └─────▲──────┘
            │                                 │
            │          双向提问与答疑         │
            └─────────────────────────────────┘
```

- **角色自由转换**：没有固定的主从等级。在初始策划阶段，Agent A 可以作为 Commander 制定战略并分解任务；进入专业审查阶段，Agent B 也可以作为 Commander 提审要求 Agent A 修正，双方既能指挥也能执行。
- **全向消息与工单互通**：任何 Agent 都可以通过内置通讯协议直接向特定同伴发送指令、发起交互式提问或广播共享认知。
- **非 Git 强依赖**：对于论文写作、设计素材、财务报表、法务卷宗等场景，文件可以是 Markdown、DOCX、PDF、CSV、PNG 或 SVG，无需强行维护复杂的 Git 分支历史。

### 2. 核心协同机制与 How to Use（完整使用方法）

#### ① 激活 Project 模式
1. 在左侧协同面板顶部，点击 **项目（Project）** 标签切换至 Project 模式。
2. 此时协同面板将展示当前工作区中所有注册的 Agent 成员列表与实时状态。

#### ② 跨 Agent 点对点通信与即时消息
当 Agent A 需要 Agent B 协同协助时，可调用原生通信工具直接发信：
```json
// Agent A 调用 warp_agent_send 发送任务与背景资料
{
  "recipient": "agent_data_analyst",
  "message": "请对 2024-2026 年行业财报 CSV 数据进行回归分析，重点输出夏普比率与波动率矩阵。"
}
```
接收方 Agent 处于 `Idle` 提示符时，守护式 PTY 会自动将消息投递到其终端输入流中；若其处于 `Working` 状态，消息将安全暂存入其私人收件箱（`warp_agent_inbox`），在工作完成后平滑取出。

#### ③ 结构化跨 Agent 提问与应答（Ask & Answer）
在复杂的推理链条中，如果某个 Agent 需要另一名专家的输入方可继续决策，可使用阻塞式提问通道：
```json
// Agent B 向 Agent C 发起专项提问
{
  "recipient": "agent_legal_compliance",
  "question": "当前技术交底书第 3.2 节所提方案，是否会落入已有对比文件 US9876543B2 的从属权利要求保护范围？"
}
```
被提问的 Agent 会收到明确的 Question 工单，调用 `warp_agent_answer` 回填审查结论，解除提问方的等待阻塞。

#### ④ 公共任务池流转与认领（Task Pool & Claiming）
对于流水线协作，Commander Agent 可创建任务池，Worker Agent 自主拉取：
- **创建任务池与派发任务**：调用 `warp_task_create_pool`，定义多个任务条目与依赖顺序（如：先文献清洗，后图表绘制，再初稿合成）。
- **任务认领与进展上报**：空闲 Worker Agent 调用 `warp_task_claim` 认领匹配自身专长的任务，并在执行过程中调用 `warp_task_progress` 汇报完成百分比。
- **提交交付物与证据**：完成后调用 `warp_task_submit` 附带文件路径或证据描述。

#### ⑤ 共享全局上下文与动态知识库（Context Publication）
为防止跨 Agent 协作中的“幻觉断层”，各 Agent 可将阶段性契约持久化到共享上下文区：
- 调用 `warp_context_publish` 发布接口规格、统一术语表、分析结论。
- 其余同伴调用 `warp_context_get` 随时拉取最新的上下文锚点，保持全团队认知高度对齐。

---

### 3. 六大真实非 Coding 场景落地手册

以下详述 6 个典型泛行业场景中，如何配置多 Agent 团队、定义角色职责与流转全向协同：

#### 场景 1：科研论文写作与顶刊同行预审（Scientific Research & Peer Review）
- **核心痛点**：传统学术写作中，文献梳理容易遗漏重要对比、实验图表与文字脱节、缺少独立严苛的审稿人视角提前把关。
- **Agent 角色配置**：
  1. `Agent-Scholar`（文献检索与综述）：抓取 arXiv/Crossref 关键论文，提取核心贡献与基线对比数据。
  2. `Agent-DataPlot`（数据分析与绘图）：清洗实验数据，生成符合 300 DPI 顶刊规范的高清图表与专业图注。
  3. `Agent-Writer`（主文起草 Commander）：统揽整体故事线，整合各节草稿，确保逻辑严密、证据链闭环。
  4. `Agent-Polisher`（学术语言润色）：优化学术用语与逻辑转折，杜绝模糊陈述，核验引用格式。
  5. `Agent-MockReviewer`（模拟顶刊审稿人）：独立审阅全文，以苛刻视角提出 Major/Minor Concerns，命令主文团队有针对性地修回完善。
- **协同链路**：`Scholar` 检索数据发布至 `warp_context` → `DataPlot` 绘图交付成果 → `Writer` 汇总成初稿 → `MockReviewer` 审查驳回并指出机制解释薄弱点 → `Writer` 与 `Polisher` 针对性重写闭环。

#### 场景 2：工业级视觉与多媒体创意设计（Design & Creative Multimedia）
- **核心痛点**：设计需求沟通成本高、概念方案到矢量图元产出割裂、图层与视觉规范难以自动化质检。
- **Agent 角色配置**：
  1. `Agent-ArtDirector`（设计总监 Commander）：拆解产品设计规范（Design Tokens）、色彩搭配矩阵与网格系统。
  2. `Agent-PromptEngineer`（提示词工程专家）：将总监的设计构思精炼为可控的图像生成 Payload 与参数。
  3. `Agent-VectorArtist`（矢量与代码构图师）：编写原生 SVG 图标、Canvas 交互动效与响应式布局代码。
  4. `Agent-VisualQA`（视觉质检审查员）：针对对比度、排版溢出、跨平台 DPI 适配进行自动化像素级审查。
- **协同链路**：`ArtDirector` 发布视觉风格指南 → `VectorArtist` 与 `PromptEngineer` 并行产出矢量图与视觉素材 → `VisualQA` 审查并在发现溢出时调用 `warp_agent_send` 要求微调图元坐标。

#### 场景 3：量化投研与金融市场分析（Quantitative Finance & Market Intelligence）
- **核心痛点**：金融数据海量碎片化、因子回测与风控审核往往分离、策略报告生成繁琐。
- **Agent 角色配置**：
  1. `Agent-MacroAnalyst`（宏观与研报解析员）：自动解析财报、研报 PDF 与宏观经济指标。
  2. `Agent-FactorMiner`（量化因子挖掘工人）：编写特征工程逻辑，挖掘量化阿尔法（Alpha）因子。
  3. `Agent-BacktestEngine`（回测验证执行者）：运行历史数据模拟回测，计算夏普比率、最大回撤与胜率。
  4. `Agent-RiskAuditor`（风险合规风控员）：基于极端行情压力测试，审查杠杆率与尾部风险。
  5. `Agent-PortfolioLead`（投研总监 Commander）：汇总各 Agent 研判结论，统筹生成每日投资组合决策案。
- **协同链路**：`MacroAnalyst` 提取异动因子发布共享上下文 → `FactorMiner` 构建因子并交由 `BacktestEngine` 回测 → `RiskAuditor` 独立合规校验 → `PortfolioLead` 汇总签署调仓报告。

#### 场景 4：专利交底挖掘与法律合规审查（Patent Mining & Legal Compliance）
- **核心痛点**：研发人员写不好法律语言，专利代理人难以理解深层技术细节，容易导致权利要求保护范围过窄或过宽。
- **Agent 角色配置**：
  1. `Agent-TechMiner`（技术交底挖掘员）：深度阅读工程架构设计、源码与技术备忘录，提炼核心创新点。
  2. `Agent-ClaimDrafter`（权利要求书撰写员）：按照专利局规范撰写独立权利要求与层层从属权利要求。
  3. `Agent-PriorArtSearch`（现有技术查新检索员）：比对公开专利库，寻找高相似度对比文件。
  4. `Agent-PatentExaminer`（模拟专利审查员）：模拟官方审查意见通知书（OA），针对新颖性与创造性发起质询，倒逼撰写团队打磨争辩点。
- **协同链路**：`TechMiner` 提取交底书草案 → `PriorArtSearch` 检索对比文献 → `ClaimDrafter` 定稿权利要求书 → `PatentExaminer` 模拟驳回测试 → 全流程交付坚不可摧的申报文档。

#### 场景 5：新媒体内容生产与矩阵运营（Content Strategy & Media Operations）
- **核心痛点**：全网热点转瞬即逝、长内容向各平台（短视频、小红书、推文、播客）手工二次分发效率低下。
- **Agent 角色配置**：
  1. `Agent-TrendRadar`（热点监控雷达）：追踪各平台趋势热榜与竞品动态，挖掘爆款选题。
  2. `Agent-Scriptwriter`（脚本与分镜策划）：产出深度长视频文案、播客脚本与分镜头时间轴。
  3. `Agent-MultiPlatformAdapt`（多平台矩阵改写）：将主干内容重构为图文笔记、短视频口播文案与技术推文。
  4. `Agent-OpsAnalyst`（数据复盘与排程调度）：监控历史发帖数据，拟定最佳发布时间与互动话术。
- **协同链路**：`TrendRadar` 捕获热点派单给 `Scriptwriter` → 产出母版脚本后，多个 `MultiPlatformAdapt` 并行转写不同渠道文案 → 审核通过后归档排期。

#### 场景 6：商业尽职调查与战略情报分析（Business Intelligence & Strategic Research）
- **核心痛点**：多源情报（工商数据、舆情评价、供应链、高管履历）割裂，难以短时间内聚合成结构化商业研报。
- **Agent 角色配置**：
  1. `Agent-Scraper`（多源情报采集员）：清洗抓取公开财报、工商变更记录与新闻舆情。
  2. `Agent-FinancialModeler`（财务建模分析师）：重构三张财务报表，测算估值区间与盈利可持续性。
  3. `Agent-SupplyChain`（产业链图谱分析师）：梳理上下游客户集中度与核心供应商替代风险。
  4. `Agent-StrategyLead`（战略顾问 Commander）：汇总分析产出 SWOT 模型、商业画布与一页纸投资备忘录（One-Pager Memo）。
- **协同链路**：多名采集与建模 Worker 异步攻坚，实时向 `StrategyLead` 汇报进展，最终一键导出全套尽调报告。

---

## 第 4 部分：Git Worktree 并发编码模式（面向代码开发）

对于代码工程项目，多个 Agent 在同一个 Git 工作目录修改代码极易产生合并冲突与编译器缓存互踩。Warpai 的 **Worktree 编码模式** 将每个 Agent 分配到物理隔离的独立 Git Worktree 中。

```
                    ┌────────────────────────┐
                    │  Coordinator (主协调者) │
                    │   (统筹调度 & 代码集成)  │
                    └───────────┬────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
 ┌──────▼──────┐         ┌──────▼──────┐         ┌──────▼──────┐
 │ Worktree A  │         │ Worktree B  │         │ Worktree C  │
 │ feature/api │         │ feature/ui  │         │feature/tests│
 │ (Agent A)   │         │ (Agent B)   │         │ (Agent C)   │
 └─────────────┘         └─────────────┘         └─────────────┘
```

### 1. 创建物理隔离的工作树
在终端中为各个功能模块创建物理工作树：
```bash
git worktree add ../project-api feature/api
git worktree add ../project-ui feature/ui
git worktree add ../project-tests feature/tests
```
每个工作树均具备完全独立的磁盘目录、独立的分支指针与独立的依赖编译缓存（如 `target/` 或 `node_modules/`），并发读写绝对物理隔离。

### 2. 指定 Coordinator（协调者）
1. 在协同面板顶部切换至 **工作树（Worktree）** 模式。
2. 从 **Coordinator** 下拉菜单中选择一名具备全局架构认知能力的 Agent（如 Claude 或 Codex）。
3. 协调者将自动掌控全部工作树分支、当前代码状态与任务依赖图谱。

### 3. 并行任务下发与依赖设定
人类开发者或 Coordinator 可直接通过界面或 API 向各分支委派结构化任务：
```json
{
  "target_branch": "feature/api",
  "task_title": "实现 RESTful 数据导出接口",
  "dependencies": []
}
```
Worker A 在其专属物理目录中开展编码与单测，与负责 UI 的 Worker B 互不干扰，彻底杜绝 Git 脏写。

---

## 第 5 部分：代码对等评审（Peer Review）与集成流水线

在 Worktree 模式下，任务完成并非粗暴地直接合并到主干，而是遵循工业级对等评审流程：

1. **提交成果与证据链**：Worker 完成功能后，调用 `warp_task_submit` 提交 Git Diff 与本地测试通过的证据（`warp_evidence_add`）。
2. **指派对等审阅（Peer Review）**：Coordinator 调度另一名空闲的 Agent 担任 Reviewer，调用 `warp_task_review` 进行代码审查。
3. **审阅判决反馈**：
   - `needs_changes`：审查者指出潜在 Bug 或规范违背，打回重构；Worker 收到反馈自动进行二次修复。
   - `accepted`：审查者验证通过，签发准入许可。
4. **主干统一集成**：由 Coordinator 调用 `warp_task_integrate`，将已验证分支优雅合并至主干，运行全量测试并生成标准 Git Commit。
