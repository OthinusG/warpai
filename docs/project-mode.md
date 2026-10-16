---
layout: docs
title: "Project Mode: Omnidirectional Collaboration & Cross-Domain Guide"
description: "Comprehensive guide to Warpai's dedicated Project Mode for non-coding teams: break rigid hierarchies, enable dynamic Commander/Worker roles, exchange messages and tasks omnidirectionally across research, design, finance, IP, and more."
section: Using Warpai
doc_id: project-mode
prev_url: /docs/using.html
prev_title: User Guide Overview
next_url: /docs/worktree-mode.html
next_title: "Worktree Mode: Parallel Git Software Engineering"
---

In the contemporary multi-agent ecosystem, most tools treat the **Git Worktree** as the sole unit of parallel execution. While this works well for standard software development, it is excessively rigid and restrictive for **non-coding and cross-disciplinary scenarios** (such as scientific research, creative design, quantitative finance, patent compliance, and content strategy) where raw Git branching is an awkward fit.

**Warpai heavily supports and prioritizes Project Mode for these workflows**:
- **Break Rigid Master-Worker Hierarchies**: Every agent in the network can act dynamically as a **Commander** or a **Worker**, switching roles across different phases of a project.
- **Omnidirectional Messaging & Task Delegation**: Agents freely message any peer, initiate structured Q&A, publish shared context contracts, and pull work items from a public task pool.
- **No Git Repository Required**: Launch directly inside any local folder, project workspace, or cloud drive—collaborating on Markdown, DOCX drafts, PDFs, CSV tables, PNG graphics, or SVG assets.

---

## Core Mechanism: Omnidirectional Mesh Topology

Unlike the top-down tree hierarchy used in Worktree mode, Project Mode operates over an egalitarian mesh network:

```
                    ┌──────────────────────────────┐
                    │    Shared Context & Pools    │
                    │   (warp_context / task_pool) │
                    └──────┬───────────────┬───────┘
                           │               │
            ┌──────────────┴───┐       ┌───┴──────────────┐
            │                  │       │                  │
      ┌─────▼──────┐   Omnidirectional Messaging   ┌─────▼──────┐
      │  Agent A   │ ◄───────────────────────────► │  Agent B   │
      │(Commander) │                               │  (Worker)  │
      └─────▲──────┘                               └─────▲──────┘
            │                                             │
            │           Two-Way Q&A Protocol              │
            └─────────────────────────────────────────────┘
```

### 1. Dynamic Role Reversals
- **Strategic Planning Phase**: Agent A can act as Commander, mapping the high-level roadmap and populating the task pool.
- **Verification & Review Phase**: Agent B (specializing in QA or fact-checking) takes command, issuing revision notices back to Agent A. Leadership flows naturally with domain expertise.

### 2. Screen State Sensing & Native Notification Support
Warpai tracks each agent’s execution cycle in memory via its zero-overhead VT100 ANSI parser:
- 🟢 **`Idle`**: Agent completed its turn and is safely waiting at the prompt.
- 🟡 **`Working`**: Agent is reasoning, writing, or executing commands.
- 🔴 **`WaitingApproval`**: Agent triggered a safety prompt. **Its status tab badge immediately displays WaitingApproval, and a native desktop notification alerts the developer**, eliminating blind search across terminal panes.

---

## Complete How-to-Use Guide

### Step 1: Activating Project Mode
1. Launch Warpai.
2. In the upper-left collaboration panel, click **Project** to switch into Project Mode.
3. The panel displays all active agent instances registered in the current workspace alongside their real-time states.

<div class="docs-shot-placeholder" data-shot="SHOT-15">
  <div class="shot-header">
    <span class="shot-badge">SHOT-15 (P1)</span>
    <span class="shot-title">Project Mode Flat Collaboration</span>
  </div>
  <p class="shot-desc">Project 模式多智能体协同：非 Git 目录下多个 Agent（如 Claude + Codex）通过消息通信、互答疑问与发布共享上下文契约。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/project-mode-collaboration.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

### Step 2: Point-to-Point Messaging (P2P Messaging)
When an agent needs to assign work, pass intermediate data, or synchronize findings with a peer:
```json
// Agent A calls warp_agent_send to message Agent B
{
  "recipient": "agent_data_processor",
  "message": "Read data/raw_q3_report.csv, remove outliers, and output a standardized table."
}
```
- **Delivery Guarantee**: If the recipient is `Idle`, Guarded PTY immediately delivers the command. If `Working`, it queues safely into the recipient’s private inbox (`warp_agent_inbox`) until turn completion.

### Step 3: Structured Cross-Agent Questions & Answers (Ask & Answer)
When an agent encounters a domain outside its scope and needs blocking input from a specialist peer:
```json
// Agent B asks a blocking question to Agent C
{
  "recipient": "agent_compliance_officer",
  "question": "Does our updated quantitative portfolio comply with the 1.5x regulatory leverage ceiling?"
}
```
The target agent receives a formal Question card and responds via `warp_agent_answer`, unblocking the inquirer.

<div class="docs-shot-placeholder" data-shot="SHOT-27">
  <div class="shot-header">
    <span class="shot-badge">SHOT-27 (P2)</span>
    <span class="shot-title">Structured Q&A Protocol & Shared Memory Context</span>
  </div>
  <p class="shot-desc">Project 模式智能体问答与共享记忆：展示跨 Agent 呼出的结构化问答卡片（warp_agent_ask 阻断性咨询）及下方共享全局事实契约（Shared Context）面板。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1100x650</span>
    <span>目标路径: docs/images/project-mode-qa-context.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

### Step 4: Public Task Pool Management, Task Queuing & Dynamic Supervision
For multi-stage pipelines, any agent can orchestrate tasks and task pools dynamically:
1. **Durable Task Queuing & FIFO Execution**: All assigned tasks and shared pool items enter durable queues (`state: "queued"`). Each agent executes one task at a time (`state: "running"`). Senders queue multiple tasks without blocking, and workers execute them sequentially in arrival order via `warp_task_start`.
2. **Continuous Multi-Agent Dispatch**: Agents can publish new tasks to one or more peer agents at any time. There is no rigid batch boundary or requirement to wait for existing tasks to finish.
3. **Dynamic Supervision List**: Newly enqueued tasks automatically join the supervising agent's dynamic supervision list (`warp_task_list`), maintaining durable visibility across agent turns.
4. **Autonomous Claiming & Execution**: Idle workers call `warp_task_claim` to pull unassigned tasks from the pool when their running slot is free, reporting progress via `warp_task_progress`.
5. **Active Exit Upon Completion**: Workers submit deliverables via `warp_task_submit`. Once the outcome is accepted or handled, the task actively and automatically exits the supervision list.

### Step 5: Shared Global Context & Knowledge Publication
To prevent knowledge drift across agents, teams expose milestone facts in shared context memory:
- Call `warp_context_publish` to expose specifications, design tokens, or verified metrics.
- Other agents call `warp_context_get` to retrieve verified facts, ensuring full alignment.

### Step 6: Concurrent File Reservations
When multiple agents collaborate on shared files, file reservations prevent concurrent write collisions:
```json
{
  "file_path": "reports/draft_chapter3.md",
  "action": "reserve",
  "reason": "Refining introduction and closing evidence loop; holding for 3 minutes"
}
```
Release via `warp_file_release` upon completion.

---

## Six Real-World Non-Coding Domain Scenarios

### Scenario 1: Scientific Research & Academic Peer Review
- **Pain Point**: Manual literature surveys miss prior art; experimental figures drift from captions; lack of independent peer review leads to journal rejections.
- **Agent Team Configuration**:
  1. `Agent-Scholar` (Literature Retrieval): Queries arXiv/CrossRef, extracts core baselines, and compiles evidence chains.
  2. `Agent-DataPlot` (Data & Visualization): Cleans raw experiment datasets and generates publication-grade 300 DPI figures with rigorous captions.
  3. `Agent-Writer` (Manuscript Drafting Commander): Synthesizes findings into manuscript sections, maintaining claim boundaries.
  4. `Agent-Polisher` (Academic Polishing): Refines phrasing, enforces active voice conventions, and audits citation fidelity.
  5. `Agent-MockReviewer` (Simulated Top-Tier Referee): Assesses novelty and technical soundess, issuing Major/Minor concerns that force iterative revisions.
- **Coordination Flow**:
  1. `Agent-Scholar` queries literature and exposes baseline findings to global context via `warp_context_publish`.
  2. `Agent-DataPlot` claims plotting tasks, outputting high-resolution vector charts and summary tables.
  3. `Agent-Writer` drafts the Introduction and Results sections, ensuring tight alignment with figure panels.
  4. `Agent-MockReviewer` runs mock peer review, returning actionable revision checklists.
  5. `Agent-Writer` and `Agent-Polisher` refine the manuscript and generate a point-by-point Response Letter.

### Scenario 2: Industrial & Creative Multimedia Design
- **Pain Point**: High communication costs between concept planning and asset generation; hand-crafted SVG code has alignment bugs; visual QA is tedious.
- **Agent Team Configuration**:
  1. `Agent-ArtDirector` (Design Director Commander): Establishes Design Tokens, palette matrices, and spatial typography rules.
  2. `Agent-PromptEngineer` (Prompt Specialist): Translates design briefs into precise, reproducible generative payloads.
  3. `Agent-VectorArtist` (SVG & Code Vectorizer): Generates native SVG illustrations, canvas elements, and responsive layout code.
  4. `Agent-VisualQA` (Visual Quality Auditor): Inspects contrast ratios, layout overflow, and multi-resolution rendering fidelity.
- **Coordination Flow**:
  `ArtDirector` publishes token guidelines → `VectorArtist` produces vector code while `PromptEngineer` prepares textures → `VisualQA` verifies rendered assets and dispatches adjustments via `warp_agent_send`.

### Scenario 3: Quantitative Finance & Market Intelligence
- **Pain Point**: Financial data is fragmented; alpha factor generation is siloed from risk management; manual synthesis slows execution.
- **Agent Team Configuration**:
  1. `Agent-MacroAnalyst` (Macro & Earnings Analyst): Ingests SEC 10-K filings, earnings transcripts, and macroeconomic indicators.
  2. `Agent-FactorMiner` (Alpha Factor Worker): Formulates mathematical expressions and feature engineering pipelines.
  3. `Agent-BacktestEngine` (Backtest Verification Specialist): Simulates historical trades, calculating Sharpe, Sortino, and maximum drawdown.
  4. `Agent-RiskAuditor` (Risk & Compliance Officer): Stress-tests positions against tail risk and regulatory leverage limits.
  5. `Agent-PortfolioLead` (Portfolio Commander): Synthesizes agent signals to sign off on allocation adjustments.
- **Coordination Flow**:
  `MacroAnalyst` discovers catalysts → `FactorMiner` builds quantitative signals → `BacktestEngine` runs simulations → `RiskAuditor` performs stress tests → `PortfolioLead` generates the investment memo.

### Scenario 4: Patent Mining & Legal Compliance
- **Pain Point**: R&D engineers lack patent claim vocabulary, while attorneys miss underlying technical nuances, creating vulnerable IP disclosures.
- **Agent Team Configuration**:
  1. `Agent-TechMiner` (Technical Disclosure Extractor): Scans codebase commits, architectural RFCs, and lab notes to isolate inventive points.
  2. `Agent-ClaimDrafter` (Patent Claims Drafter): Formulates independent and dependent claims adhering to patent office guidelines.
  3. `Agent-PriorArtSearch` (Prior Art Searcher): Queries global patent registries to locate matching publications.
  4. `Agent-PatentExaminer` (Simulated Patent Examiner): Issues simulated Office Actions (OA) attacking novelty and obviousness to strengthen defensibility.
- **Coordination Flow**:
  `TechMiner` drafts disclosures → `PriorArtSearch` flags conflicting citations → `ClaimDrafter` sharpens claim boundaries → `PatentExaminer` tests arguments before formal filing.

### Scenario 5: Content Strategy & Omnichannel Media Operations
- **Pain Point**: Social trends fade quickly; manually adapting long-form scripts across YouTube, Substack, X/Twitter, and short video is bottlenecked.
- **Agent Team Configuration**:
  1. `Agent-TrendRadar` (Viral Trend Radar): Monitors social platforms and audience interest graphs to surface high-yield topics.
  2. `Agent-Scriptwriter` (Long-Form Storyboarder): Drafts comprehensive video scripts, podcasts, and storyboard timing marks.
  3. `Agent-MultiPlatformAdapt` (Channel Adaptation Worker): Converts primary scripts into platform-native threads, shorts, and newsletter digests.
  4. `Agent-GrowthAnalyst` (Analytics & Schedule Manager): Audits performance metrics and schedules optimal release cadences.
- **Coordination Flow**:
  `TrendRadar` assigns topics to `Scriptwriter` → Master copy is generated → `MultiPlatformAdapt` produces variants in parallel → `GrowthAnalyst` checks engagement targets.

### Scenario 6: Due Diligence & Strategic Business Intelligence
- **Pain Point**: Incomplete public records, complex cross-holdings, and manual financial modeling delay investment decisions.
- **Agent Team Configuration**:
  1. `Agent-Scraper` (Corporate Intelligence Scraper): Gathers public registries, corporate restructuring notices, and customer sentiment.
  2. `Agent-FinancialModeler` (Financial Modeling Analyst): Reconstructs 3-statement models, DCF valuations, and unit economics.
  3. `Agent-SupplyChain` (Supply Chain Analyst): Maps supplier dependencies and customer concentration vulnerabilities.
  4. `Agent-StrategyLead` (Strategy Consultant Commander): Merges findings into SWOT frameworks, investment memos, and executive one-pagers.
- **Coordination Flow**:
  Multiple research workers ingest diverse datasets in parallel, periodically reporting findings to `StrategyLead` to produce unified deliverables.
