---
layout: docs
title: "Deep Dive: Multi-Agent Workflows"
description: "Explore Warpai's dual coordination architecture: isolated Git Worktree mode for parallel feature development and omnidirectional Project mode for cross-domain teams, featuring dynamic Commander/Worker roles, inter-agent messaging, native status sensing, and 6+ real-world non-coding scenarios."
section: Using Warpai
doc_id: using-warpai-agents
prev_url: /docs/using-terminal.html
prev_title: Terminal & Command Blocks
next_url: /docs/cli-agents.html
next_title: 20+ CLI Agents Guide
---

Warpai elevates independent agent CLI sessions into first-class citizenship. However many agents you run across your projects, each gets its own native physical terminal instance with hardware-accelerated rendering.

Most multi-agent frameworks in the industry treat the **Git Worktree** as the sole unit of parallel execution. While worktrees excel in traditional software engineering, they prove rigid and constraining for **non-coding and cross-disciplinary scenarios** (such as scientific research, creative design, quantitative finance, patent drafting, and content strategy). To bridge this divide, Warpai delivers a groundbreaking **Dual-Architecture Coordination System**:

1. **Git Worktree Coding Mode**: Built specifically for rigorous software engineering in Git repositories. Beyond the Coordinator, each agent develops features concurrently within physically isolated Git worktrees—guaranteeing that branches, lock files, and compiler caches never collide.
2. **Project Omnidirectional Mode**: A heavily supported paradigm for non-coding and multi-agent team workflows! Activated in any project directory without Git constraints. It breaks the traditional rigid hierarchy: **every agent can act interchangeably as a Commander or a Worker, freely exchanging messages, asking questions, publishing shared context contracts, and claiming tasks from a public pool**.

---

## Part 1: Enabling the Agent Communication Bus

Before initiating multi-agent collaboration, activate Warpai's local communication bus:

1. Open **Settings** (`Cmd + ,` on macOS, `Ctrl + ,` on Windows).
2. Navigate to **Features > Agent communication**.
3. Toggle the **Agent communication** switch **ON**.
4. Review the detected CLI agent checklist (e.g., `Claude Code`, `Codex`, `Gemini CLI`, `Antigravity`, `Cursor Agent`, `Trae`, etc.) to ensure all active agents are enabled.
5. Close Settings. Open a fresh terminal tab (`Cmd + T` / `Ctrl + T`) to ensure bridge environment variables (`WARP_AGENT_ENDPOINT`) are active.

---

## Part 2: Running Agents, State Sensing & Notification System

### 1. Launching Native Agent Processes
In any Warpai terminal pane, launch your agent via its standard CLI command:
```bash
claude   # Anthropic Claude Code
codex    # OpenAI Codex
agy      # Google Antigravity
gemini   # Google Gemini CLI
```

### 2. In-Memory VT100 Screen State Detection
Warpai includes a native zero-overhead VT100 ANSI stream parser in memory, detecting each agent's execution phase in real time without external hooks:

- 🟢 **`Idle`**: The agent has concluded its turn and is safely waiting at the shell prompt for new input. Guarded PTY delivery uses this state to dispatch queued tasks.
- 🟡 **`Working`**: The agent is reasoning, generating code/text, invoking local tools, or running shell commands.
- 🔴 **`WaitingApproval`**: The agent triggered a safety boundary (such as requesting privileged file deletion, modifying system configs, or accessing external networks).

### 3. Status Tab Indicators & Desktop Notifications
In conventional multi-pane terminals, developers are forced to manually flip through tabs to find an agent paused at a confirmation prompt.

In Warpai:
- When any agent enters the `WaitingApproval` state, its **vertical tab indicator and collaboration panel badge immediately switch to WaitingApproval**.
- Simultaneously, Warpai triggers a native **system desktop notification**, identifying the blocked agent name, workspace, and requested operation.
- Clicking the notification immediately focuses the relevant pane so you can grant permission with `Enter` or reject it with `Esc`.

<div class="docs-shot-placeholder" data-shot="SHOT-13">
  <div class="shot-header">
    <span class="shot-badge">SHOT-13 (P1)</span>
    <span class="shot-title">Human-in-the-Loop Approval Prompt</span>
  </div>
  <p class="shot-desc">Human-in-the-loop permission prompt: agent requests privileged action in terminal, status tab updates with WaitingApproval indicator, and native desktop notification alerts the developer.</p>
  <div class="shot-meta">
    <span>Recommended dimensions: 1000x500 (2:1)</span>
    <span>Target path: docs/images/agent-waiting-approval-terminal.png</span>
    <span class="shot-tag">Pending screenshot · See docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## Part 3: Project Mode — Omnidirectional Collaboration & Complete How-to-Use Guide (Core Focus)

For non-coding and multi-disciplinary teams, Warpai provides full-fledged native support for **Project Mode**.

### 1. Core Mechanism: Omnidirectional Mesh Topology

Rather than forcing every operation into a Git commit tree, Project Mode operates directly over a project folder (or standard directory/shared drive).

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

- **Dynamic Role Reversals**: There is no rigid master-worker hierarchy. During the planning stage, Agent A can act as Commander to set the strategic roadmap and assign deliverables. During peer review, Agent B can act as Commander to request revisions from Agent A. Every agent can lead or execute.
- **Omnidirectional Messaging & Tasks**: Any agent can communicate with specific peers directly, ask blocking questions, or broadcast updates to the public pool.
- **No Git Lock-In**: Whether manipulating Markdown, DOCX drafts, PDF papers, CSV datasets, PNG visuals, or SVG vectors, teams collaborate seamlessly without branch overhead.

### 2. Core Coordination Mechanisms & How to Use

#### ① Activating Project Mode
1. In the upper-left collaboration panel, click **Project** to switch into Project Mode.
2. The panel displays all registered agents active in the current project alongside their live states.

#### ② Point-to-Point Messaging
When Agent A requires assistance from Agent B, it invokes the native messaging tool:
```json
// Agent A sends a direct task with parameters to Agent B
{
  "recipient": "agent_data_analyst",
  "message": "Perform regression analysis on the 2024-2026 financial report CSV, calculating the Sharpe ratio and covariance matrix."
}
```
If the recipient is `Idle`, Guarded PTY safely injects the payload into its prompt. If the recipient is `Working`, the message queues into its private inbox (`warp_agent_inbox`) and arrives upon turn completion.

#### ③ Structured Cross-Agent Questions & Answers (Ask & Answer)
When an agent cannot proceed without a peer expert's input, it opens a structured question channel:
```json
// Agent B asks a blocking question to Agent C
{
  "recipient": "agent_legal_compliance",
  "question": "Does Section 3.2 of the technical disclosure risk infringement on patent US9876543B2 claim 4?"
}
```
The recipient receives a formal Question card and responds via `warp_agent_answer`, unblocking the inquiring agent immediately.

#### ④ Public Task Pools & Dynamic Task Supervision
For pipeline workflows, a Commander Agent can orchestrate a shared task pool with continuous dispatch:
- **Continuous Dispatch at Any Time**: Agents can publish tasks to multiple agents at any time, without waiting for fixed batch phases or prior tasks to conclude.
- **Dynamic Supervision Set**: Newly assigned tasks automatically enter the supervising agent's monitored task list, tracking progress and status across turn yields.
- **Claim & Report Progress**: Idle worker agents call `warp_task_claim` to pick tasks matching their specialty, reporting progress with `warp_task_progress`.
- **Submit & Active Retirement**: Once completed, workers invoke `warp_task_submit`. When the result is verified and accepted, the task actively exits the supervising agent's task list.

#### ⑤ Shared Global Context & Living Knowledge Bases
To prevent hallucinations across agents, teams maintain synchronized facts in shared context memory:
- Call `warp_context_publish` to expose specifications, design tokens, or verified metrics.
- Other agents call `warp_context_get` to retrieve verified facts, ensuring alignment across all participants.

---

### 3. Six Real-World Non-Coding Domain Scenarios

Here is how cross-disciplinary teams configure multi-agent swarms with distinct roles and communication pipelines:

#### Scenario 1: Scientific Research & Academic Peer Review
- **Pain Point**: Manual literature surveys miss key prior art; experimental figures frequently mismatch draft captions; missing independent peer review leads to harsh journal rejections.
- **Agent Roles**:
  1. `Agent-Scholar` (Literature Retrieval): Searches arXiv/CrossRef, extracts core baselines, and compiles evidence chains.
  2. `Agent-DataPlot` (Data & Visualization): Cleans raw experiment datasets and generates publication-grade 300 DPI figures with rigorous captions.
  3. `Agent-Writer` (Manuscript Drafting Commander): Synthesizes findings into manuscript sections, maintaining claim boundaries.
  4. `Agent-Polisher` (Academic Polishing): Refines phrasing, enforces active voice conventions, and audits citation fidelity.
  5. `Agent-MockReviewer` (Simulated Top-Tier Referee): Assesses novelty and technical soundess, issuing Major/Minor concerns that force iterative revisions.
- **Coordination Flow**: `Scholar` publishes sources → `DataPlot` outputs verified figures → `Writer` drafts the manuscript → `MockReviewer` challenges the core mechanism → `Writer` & `Polisher` revise to form a closed evidence loop.

#### Scenario 2: Creative & Industrial Design
- **Pain Point**: High communication costs between concept planning and asset generation; hand-crafted SVG code has alignment bugs; visual QA is tedious.
- **Agent Roles**:
  1. `Agent-ArtDirector` (Design Director Commander): Establishes Design Tokens, palette matrices, and spatial typography rules.
  2. `Agent-PromptEngineer` (Prompt Specialist): Translates design briefs into precise, reproducible generative payloads.
  3. `Agent-VectorArtist` (SVG & Code Vectorizer): Generates native SVG illustrations, canvas elements, and responsive layout code.
  4. `Agent-VisualQA` (Visual Quality Auditor): Inspects contrast ratios, layout overflow, and multi-resolution rendering fidelity.
- **Coordination Flow**: `ArtDirector` publishes token guidelines → `VectorArtist` produces vector code while `PromptEngineer` prepares textures → `VisualQA` verifies rendered assets and dispatches adjustments via `warp_agent_send`.

#### Scenario 3: Quantitative Finance & Market Intelligence
- **Pain Point**: Financial data is fragmented; alpha factor generation is siloed from risk management; manual synthesis slows execution.
- **Agent Roles**:
  1. `Agent-MacroAnalyst` (Macro & Earnings Analyst): Ingests SEC 10-K filings, earnings transcripts, and macroeconomic indicators.
  2. `Agent-FactorMiner` (Alpha Factor Worker): Formulates mathematical expressions and feature engineering pipelines.
  3. `Agent-BacktestEngine` (Backtest Verification Specialist): Simulates historical trades, calculating Sharpe, Sortino, and maximum drawdown.
  4. `Agent-RiskAuditor` (Risk & Compliance Officer): Stress-tests positions against tail risk and regulatory leverage limits.
  5. `Agent-PortfolioLead` (Portfolio Commander): Synthesizes agent signals to sign off on allocation adjustments.
- **Coordination Flow**: `MacroAnalyst` discovers catalysts → `FactorMiner` builds quantitative signals → `BacktestEngine` runs simulations → `RiskAuditor` performs stress tests → `PortfolioLead` generates the investment memo.

#### Scenario 4: Patent Mining & Legal Compliance
- **Pain Point**: R&D engineers lack patent claim vocabulary, while attorneys miss underlying technical nuances, creating vulnerable IP disclosures.
- **Agent Roles**:
  1. `Agent-TechMiner` (Technical Disclosure Extractor): Scans codebase commits, architectural RFCs, and lab notes to isolate inventive points.
  2. `Agent-ClaimDrafter` (Patent Claims Drafter): Formulates independent and dependent claims adhering to patent office guidelines.
  3. `Agent-PriorArtSearch` (Prior Art Searcher): Queries global patent registries to locate matching publications.
  4. `Agent-PatentExaminer` (Simulated Patent Examiner): Issues simulated Office Actions (OA) attacking novelty and obviousness to strengthen defensibility.
- **Coordination Flow**: `TechMiner` drafts disclosures → `PriorArtSearch` flags conflicting citations → `ClaimDrafter` sharpens claim boundaries → `PatentExaminer` tests arguments before formal filing.

#### Scenario 5: Content Strategy & Omnichannel Media Operations
- **Pain Point**: Social trends fade quickly; manually adapting long-form scripts across YouTube, Substack, X/Twitter, and short video is bottlenecked.
- **Agent Roles**:
  1. `Agent-TrendRadar` (Viral Trend Radar): Monitors social platforms and audience interest graphs to surface high-yield topics.
  2. `Agent-Scriptwriter` (Long-Form Storyboarder): Drafts comprehensive video scripts, podcasts, and storyboard timing marks.
  3. `Agent-MultiPlatformAdapt` (Channel Adaptation Worker): Converts primary scripts into platform-native threads, shorts, and newsletter digests.
  4. `Agent-GrowthAnalyst` (Analytics & Schedule Manager): Audits performance metrics and schedules optimal release cadences.
- **Coordination Flow**: `TrendRadar` assigns topics to `Scriptwriter` → Master copy is generated → `MultiPlatformAdapt` produces variants in parallel → `GrowthAnalyst` checks engagement targets.

#### Scenario 6: Due Diligence & Strategic Business Intelligence
- **Pain Point**: Incomplete public records, complex cross-holdings, and manual financial modeling delay investment decisions.
- **Agent Roles**:
  1. `Agent-Scraper` (Corporate Intelligence Scraper): Gathers public registries, corporate restructuring notices, and customer sentiment.
  2. `Agent-FinancialModeler` (Financial Modeling Analyst): Reconstructs 3-statement models, DCF valuations, and unit economics.
  3. `Agent-SupplyChain` (Supply Chain Analyst): Maps supplier dependencies and customer concentration vulnerabilities.
  4. `Agent-StrategyLead` (Strategy Consultant Commander): Merges findings into SWOT frameworks, investment memos, and executive one-pagers.
- **Coordination Flow**: Multiple research workers ingest diverse datasets in parallel, periodically reporting findings to `StrategyLead` to produce unified deliverables.

---

## Part 4: Git Worktree Mode (For Parallel Software Development)

When developing code inside a Git repository, having multiple agents edit the same local branch causes merge conflicts and corrupted build caches. Warpai's **Worktree Mode** assigns each agent to an isolated physical Git worktree.

```
                    ┌────────────────────────┐
                    │  Coordinator Agent     │
                    │   (Oversight & Merge)  │
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

### 1. Creating Isolated Physical Worktrees
Create separate worktrees for each feature directly from the terminal:
```bash
git worktree add ../project-api feature/api
git worktree add ../project-ui feature/ui
git worktree add ../project-tests feature/tests
```
Each worktree possesses its own physical directory, branch head, and compiler cache (`target/`, `node_modules/`), preventing concurrent write conflicts.

### 2. Selecting the Coordinator
1. Switch to **Worktree** mode at the top of the collaboration panel.
2. Select an architecturally capable agent from the **Coordinator** dropdown.
3. The Coordinator automatically oversees all worktrees, tracking task progress and dependency graphs.

### 3. Assigning Concurrent Feature Tasks & Dynamic Supervision
The developer or Coordinator dispatches structured tasks to specific branches at any time:
```json
{
  "target_branch": "feature/api",
  "task_title": "Implement RESTful export endpoints",
  "dependencies": []
}
```
- **Continuous Multi-Agent Dispatch**: The Coordinator is never blocked waiting for a single worker to finish. It can continuously publish new tasks to multiple agents at any time.
- **Dynamic Supervision List**: Every newly assigned task is automatically enrolled into the Coordinator's active supervision list.
- **Active Exit Upon Completion**: Worker A codes and runs unit tests in its isolated worktree while Worker B develops UI features concurrently. Once a task completes peer review and is accepted, it automatically exits the supervision list.

---

## Part 5: Peer Review & Mainline Integration Pipeline

In Worktree Mode, completed work is never blindly merged into `main`. Instead, it undergoes a disciplined peer review cycle:

1. **Submission & Evidence**: Worker finishes implementation and calls `warp_task_submit` accompanied by test logs (`warp_evidence_add`) and git diffs.
2. **Assigning Peer Review**: Coordinator assigns another available agent as reviewer (`warp_task_review`).
3. **Review Verdict**:
   - `needs_changes`: Reviewer flags bugs or architectural deviations; worker automatically receives actionable items and revises.
   - `accepted`: Reviewer validates tests and grants approval.
4. **Integration**: Coordinator invokes `warp_task_integrate` to merge the reviewed branch into `main`, execute test suites, and produce a clean Git commit.
