---
layout: docs
lang: zh
title: "Project 模式：全向协同与泛行业实战手册"
description: "Warpai 重点扶持的非 Coding 泛行业协同指南：打破单一层级指挥链，所有 Agent 均可作为指挥官或工人全向互通消息与任务，涵盖科研论文、视觉设计、金融量化、专利合规等完整实战教程。"
section: 使用指南
doc_id: project-mode
prev_url: /docs/using.html
prev_title: 使用指南总览
next_url: /docs/worktree-mode.html
next_title: "Worktree 模式：Git 并行代码开发完全手册"
---

在当今的多智能体应用领域，大多数工具仅将 **Git Worktree** 作为唯一的任务划分单元。这种模式虽然适合传统软件工程，但对于日益繁重的**非 Coding 与跨学科场景**（如科学研究、工业设计、金融分析、知识产权法务、自媒体运营等），纯 Git 分支和提交历史显得极不协调且充满束缚。

**Warpai 重点扶持并设计了专有的 Project 模式（项目模式）**：
- **彻底打破单一主从指挥链**：网络拓扑内的**所有 Agent 既可以是指挥官（Commander），也可以是工人（Worker）**，角色可在不同任务阶段自由切换。
- **全向点对点通信与任务流转**：任何 Agent 之间均可互相收发消息、发起阻塞式提问、广播全局上下文契约并从公共任务池自由认领任务。
- **免 Git 仓库约束**：可在任意本地普通目录、项目工作区或网盘同步文件夹中即开即用，直接协同处理 Markdown、DOCX、PDF、CSV、PNG 或 SVG 文件。

---

## 核心机制：全向网状拓扑（Omnidirectional Mesh）

不同于 Worktree 模式下由上至下的集中式调度，Project 模式建立在平等的网状通信架构之上：

```
                    ┌──────────────────────────────┐
                    │    全局上下文契约 & 任务池    │
                    │   (warp_context / task_pool) │
                    └──────┬───────────────┬───────┘
                           │               │
            ┌──────────────┴───┐       ┌───┴──────────────┐
            │                  │       │                  │
      ┌─────▼──────┐     消息与工单全向互通    ┌─────▼──────┐
      │  Agent A   │ ◄──────────────────────► │  Agent B   │
      │(Commander) │                          │  (Worker)  │
      └─────▲──────┘                          └─────▲──────┘
            │                                       │
            │           双向提问与实时答疑          │
            └───────────────────────────────────────┘
```

### 1. 动态角色转换机制（Dynamic Role Reversal）
- **初期规划阶段**：例如由负责方案策划的 Agent A 担任指挥官，分解交付目标并向任务池派发需求。
- **专业审查阶段**：负责质量核验或事实核查的 Agent B 自动接管指挥权，对 Agent A 提交的草案发起审查质询，下发修回修改清单，双方无缝互换主导权。

### 2. 状态感知与人机安全屏障
Warpai 通过内存级 VT100 ANSI 观察器实时追踪所有 Agent 的状态：
- 🟢 **`Idle`（空闲就绪）**：已就绪在提示符，接收并执行队列任务。
- 🟡 **`Working`（执行中）**：正在思考推理、调用本地工具或生成素材。
- 🔴 **`WaitingApproval`（等待审批）**：触发提权操作（如写敏感目录或高危操作）时暂停，**对应的垂直 Tab 标签页与协作面板即时切换为高亮 WaitingApproval 标识，并同步触发系统桌面通知**，开发者无需在众多窗格中逐个盲猜翻找。

---

## 完整使用指南（How to Use）

### 第 1 步：激活 Project 模式
1. 打开 Warpai 桌面工作台。
2. 在左侧协同面板顶部，点击 **项目（Project）** 标签切换进入 Project 模式。
3. 协同面板将即时显示当前工作空间内注册的所有活跃 Agent 实例列表及其实时状态。

<div class="docs-shot-placeholder" data-shot="SHOT-15">
  <div class="shot-header">
    <span class="shot-badge">SHOT-15 (P1)</span>
    <span class="shot-title">Project 模式多智能体平权协同全景</span>
  </div>
  <p class="shot-desc">Project 模式多智能体协同：非 Git 目录下多个 Agent（如 Claude + Codex）通过消息通信、互答疑问与发布共享上下文契约。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/project-mode-collaboration.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

### 第 2 步：跨 Agent 点对点通信（P2P Messaging）
当任意 Agent 需要向同伴下发任务、移交中间数据或同步结论时，调用原生通信工具：
```json
// Agent A 调用 warp_agent_send 向 Agent B 发送协作消息
{
  "recipient": "agent_data_processor",
  "message": "请读取 data/raw_q3_report.csv，执行异常值过滤与标准化处理，产出清洗后的归一化表格。"
}
```
- **投递保障**：若接收方处于 `Idle` 提示符状态，守护式 PTY 立即将指令安全送达；若其处于 `Working` 状态，消息将暂存入其私人收件箱（`warp_agent_inbox`），待其本轮结束后平滑提取。

### 第 3 步：跨 Agent 结构化问答通道（Ask & Answer）
在多 Agent 协同推理中，当某 Agent 遇到自身知识库盲区，需向特定专家求证时，可发起结构化问答：
```json
// Agent B 向专业合规 Agent 发起澄清提问
{
  "recipient": "agent_compliance_officer",
  "question": "本季度量化策略调整后，最大杠杆比例是否符合监管披露要求（上限 1.5 倍）？"
}
```
被提问的 Agent 会在其收件箱中收到专属 Question 工单，调用 `warp_agent_answer` 回填结论，从而解除提问方的等待阻塞。

<div class="docs-shot-placeholder" data-shot="SHOT-27">
  <div class="shot-header">
    <span class="shot-badge">SHOT-27 (P2)</span>
    <span class="shot-title">Project 模式跨智能体结构化问答与共享记忆</span>
  </div>
  <p class="shot-desc">Project 模式智能体问答与共享记忆：展示跨 Agent 呼出的结构化问答卡片（warp_agent_ask 阻断性咨询）及下方共享全局事实契约（Shared Context）面板。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1100x650</span>
    <span>目标路径: docs/images/project-mode-qa-context.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

### 第 4 步：公共任务池、任务排队与动态监督（Task Queuing & Dynamic Supervision）
对于复杂多阶段协作，任何具备指挥意图的 Agent 均可创建公共任务池并随时派发任务：
1. **任务持久化排队与 FIFO 串行消费**：所有派发的任务和公共任务池项均持久化进入队列（`state: "queued"`）。每个 Agent 在同一时间独占执行一项任务（`state: "running"`）。派发方下发任务仅负责入队而无需等待，执行方在当前任务提交后，通过 `warp_task_start` 依次出队执行。
2. **随时向多个 Agent 动态派发**：打破固定批次限制，Agent 可在任意时刻向一个或多个同伴发布新任务，无序等待前序任务执行完毕。
3. **自动进入动态监督任务列表**：新入队任务自动加入主 Agent 的监督任务列表（`warp_task_list`），全程保持跨轮次的可视化监督。
4. **工人自主认领与执行**：空闲 Worker Agent（当前无运行任务时）调用 `warp_task_claim` 认领与其领域匹配的任务，周期性调用 `warp_task_progress` 上报完成进度。
5. **提交与完成主动退出**：Worker 调用 `warp_task_submit` 提交交付物；任务一旦经审阅确认或终态处理完成，便主动并自动从监督列表中退出。

### 第 5 步：共享全局上下文与动态知识库（Context Publication）
为防止跨 Agent 协作中的“认知脱节”，各 Agent 可将关键契约持久化至全局上下文区：
- 调用 `warp_context_publish` 暴露事实清单、术语定义与中间配置。
- 其他 Agent 调用 `warp_context_get` 随时拉取最新上下文，保证全队高度对齐。

### 第 6 步：多 Agent 并发文件锁定（File Reservations）
当多个 Agent 同时需要读写共享资料时，调用文件预留机制防止写覆盖冲突：
```json
{
  "file_path": "reports/draft_chapter3.md",
  "action": "reserve",
  "reason": "正在重构引言与实验证据链闭环，预计占用 3 分钟"
}
```
完成后调用 `warp_file_release` 释放，确保文件系统绝对安全。

---

## 六大泛行业非 Coding 真实场景实战指南

### 场景 1：科研论文写作与顶刊同行预审（Scientific Research & Peer Review）
- **行业痛点**：传统学术写作中，文献梳理容易遗漏重要对比、实验图表与文字脱节、缺少独立严苛的审稿人视角提前把关。
- **Agent 小队配置**：
  1. `Agent-Scholar`（文献检索与综述）：抓取 arXiv/CrossRef 关键文献，梳理核心贡献与基线数据。
  2. `Agent-DataPlot`（数据分析与绘图）：清洗实验数据，生成符合 300 DPI 顶刊规范的高清图表与专业图注。
  3. `Agent-Writer`（主文起草 Commander）：统揽整体故事线，整合各节草稿，确保逻辑严密、证据链闭环。
  4. `Agent-Polisher`（学术语言润色）：优化学术用语与逻辑转折，杜绝模糊陈述，核验引用格式。
  5. `Agent-MockReviewer`（模拟顶刊审稿人）：独立审阅全文，以苛刻视角提出 Major/Minor Concerns，命令主文团队有针对性地修回完善。
- **协同链路**：
  1. `Agent-Scholar` 检索前沿进展，提取 baseline 数据并通过 `warp_context_publish` 发布至全局上下文。
  2. `Agent-DataPlot` 认领绘图任务，产出高分辨率矢量图与数据统计表。
  3. `Agent-Writer` 起草 Introduction 与 Results 章节，保持论据与图表严密对应。
  4. `Agent-MockReviewer` 执行同行预审，开具审稿修回清单打回 `Agent-Writer`。
  5. `Agent-Writer` 与 `Agent-Polisher` 针对性精修，输出终稿并附带点对点 Response Letter。

### 场景 2：工业级视觉与多媒体创意设计（Industrial & Creative Multimedia Design）
- **行业痛点**：设计需求沟通成本高、概念方案到矢量图元产出割裂、图层与视觉规范难以自动化质检。
- **Agent 小队配置**：
  1. `Agent-ArtDirector`（设计总监 Commander）：制定设计规范（Design Tokens）、色彩搭配矩阵与网格排版系统。
  2. `Agent-PromptEngineer`（提示词工程专家）：将总监的设计构思精炼为可控的图像生成 Payload 与参数。
  3. `Agent-VectorArtist`（矢量与代码构图师）：编写原生 SVG 图标、Canvas 交互动效与响应式布局代码。
  4. `Agent-VisualQA`（视觉质检审查员）：针对对比度、排版溢出、跨平台 DPI 适配进行自动化像素级审查。
- **协同链路**：
  `ArtDirector` 发布视觉风格指南 → `VectorArtist` 与 `PromptEngineer` 并行产出矢量图与视觉素材 → `VisualQA` 审查并在发现溢出时调用 `warp_agent_send` 要求微调图元坐标。

### 场景 3：量化投研与金融市场分析（Quantitative Finance & Market Intelligence）
- **行业痛点**：金融数据海量碎片化、因子回测与风控审核往往分离、策略报告生成繁琐。
- **Agent 小队配置**：
  1. `Agent-MacroAnalyst`（宏观与研报解析员）：自动解析财报、研报 PDF 与宏观经济指标。
  2. `Agent-FactorMiner`（量化因子挖掘工人）：编写特征工程逻辑，挖掘量化阿尔法（Alpha）因子。
  3. `Agent-BacktestEngine`（回测验证执行者）：运行历史数据模拟回测，计算夏普比率、最大回撤与胜率。
  4. `Agent-RiskAuditor`（风险合规风控员）：基于极端行情压力测试，审查杠杆率与尾部风险。
  5. `Agent-PortfolioLead`（投研总监 Commander）：汇总各 Agent 研判结论，统筹生成每日投资组合决策案。
- **协同链路**：
  `MacroAnalyst` 提取异动因子发布共享上下文 → `FactorMiner` 构建因子并交由 `BacktestEngine` 回测 → `RiskAuditor` 独立合规校验 → `PortfolioLead` 汇总签署调仓报告。

### 场景 4：专利交底挖掘与法律合规审查（Patent Mining & Legal Compliance）
- **行业痛点**：研发人员写不好法律语言，专利代理人难以理解深层技术细节，容易导致权利要求保护范围过窄或过宽。
- **Agent 小队配置**：
  1. `Agent-TechMiner`（技术交底挖掘员）：深度阅读工程架构设计、源码与技术备忘录，提炼核心创新点。
  2. `Agent-ClaimDrafter`（权利要求书撰写员）：按照专利局规范撰写独立权利要求与层层从属权利要求。
  3. `Agent-PriorArtSearch`（现有技术查新检索员）：比对公开专利库，寻找高相似度对比文件。
  4. `Agent-PatentExaminer`（模拟专利审查员）：模拟官方审查意见通知书（OA），针对新颖性与创造性发起质询，倒逼撰写团队打磨争辩点。
- **协同链路**：
  `TechMiner` 提取交底书草案 → `PriorArtSearch` 检索对比文献 → `ClaimDrafter` 定稿权利要求书 → `PatentExaminer` 模拟驳回测试 → 全流程交付坚不可摧的申报文档。

### 场景 5：新媒体内容生产与矩阵全渠道运营（Content Strategy & Omnichannel Media Operations）
- **行业痛点**：全网热点转瞬即逝、长内容向各平台（短视频、小红书、推文、播客）手工二次分发效率低下。
- **Agent 小队配置**：
  1. `Agent-TrendRadar`（热点监控雷达）：追踪各平台趋势热榜与竞品动态，挖掘爆款选题。
  2. `Agent-Scriptwriter`（脚本与分镜策划）：产出深度长视频文案、播客脚本与分镜头时间轴。
  3. `Agent-MultiPlatformAdapt`（多平台矩阵改写）：将主干内容重构为图文笔记、短视频口播文案与技术推文。
  4. `Agent-OpsAnalyst`（数据复盘与排程调度）：监控历史发帖数据，拟定最佳发布时间与互动话术。
- **协同链路**：
  `TrendRadar` 捕获热点派单给 `Scriptwriter` → 产出母版脚本后，多个 `MultiPlatformAdapt` 并行转写不同渠道文案 → 审核通过后归档排期。

### 场景 6：商业尽职调查与战略情报分析（Business Intelligence & Due Diligence）
- **行业痛点**：多源情报（工商数据、舆情评价、供应链、高管履历）割裂，难以短时间内聚合成结构化商业研报。
- **Agent 小队配置**：
  1. `Agent-Scraper`（多源情报采集员）：清洗抓取公开财报、工商变更记录与新闻舆情。
  2. `Agent-FinancialModeler`（财务建模分析师）：重构三张财务报表，测算估值区间与盈利可持续性。
  3. `Agent-SupplyChain`（产业链图谱分析师）：梳理上下游客户集中度与核心供应商替代风险。
  4. `Agent-StrategyLead`（战略顾问 Commander）：汇总分析产出 SWOT 模型、商业画布与一页纸投资备忘录（One-Pager Memo）。
- **协同链路**：
  多名采集与建模 Worker 异步攻坚，实时向 `StrategyLead` 汇报进展，最终一键导出全套尽调报告。
