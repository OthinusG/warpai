---
layout: docs
lang: zh
title: 多 Agent 协同工作流
description: "通过 Worktree 隔离开发、Coordinator 任务池调度、人机交互审批、对等评审（Peer Review）与原生 MCP 同步编排多智能体团队。"
section: 使用指南
doc_id: using-warpai-agents
prev_url: /docs/using-terminal.html
prev_title: 终端与命令块深入
next_url: /docs/cli-agents.html
next_title: 20+ 款 CLI Agent 配置指南
---

本指南是使用 Warpai 进行多智能体编排协同的实战手册，涵盖启动 Agent 会话、管理自主任务队列、在**项目模式（Project Mode）**下组织调研研讨小队、在**工作树模式（Worktree Mode）**下主导并行软件工程，以及跨 SSH 和 WSL 2 远程协同。

---

## 第 1 部分：启用 Agent 协同总线

在开始多智能体协同前，需激活本地通信桥接层：

1. 打开 **设置（Settings）**（macOS 上按 `Cmd + ,`，Windows 上按 `Ctrl + ,`）。
2. 前往 **功能（Features） > Agent communication**。
3. 将 **Agent communication** 切换为 **开启（ON）**。
4. 检查已识别到的 CLI Agent 清单（如 `Claude Code`、`Codex`、`Gemini`、`Cursor`、`Antigravity`、`Trae` 等），确保你在使用的 Agent 均已勾选。
5. 关闭设置，新建一个终端标签页（`Cmd + T` / `Ctrl + T`），以确保通信环境变量（`WARP_AGENT_ENDPOINT`）在会话中生效。

---

## 第 2 部分：运行与实时监管单个 Agent

### 1. 启动 Agent 进程
在任意终端窗格中，输入对应的原生 CLI 命令回车运行：
```bash
claude   # 启动 Claude Code
codex    # 启动 OpenAI Codex
agy      # 启动 Google Antigravity
```

### 2. 观察 Agent 实时状态
在 **左侧协同面板** 中，该 Agent 将作为一个活跃成员展示：
- **Agent 图标与名称**：（如 Claude、Codex、Gemini）。
- **进程 ID 与物理目录**：明确标识当前正在哪一个物理文件夹中作业。
- **屏幕状态指示灯**：
  - 🟢 **`Idle`（就绪）**：已完成上一轮交互，在提示符等待新任务。
  - 🟡 **`Working`（执行中）**：正在推理、生成代码、调用工具或运行 Shell 命令。
  - 🔴 **`WaitingApproval`（等待审批）**：在敏感安全节点暂停（如请求写入文件或运行高危命令）。

<div class="docs-shot-placeholder" data-shot="SHOT-13">
  <div class="shot-header">
    <span class="shot-badge">SHOT-13 (P1)</span>
    <span class="shot-title">Human-in-the-Loop Approval Prompt</span>
  </div>
  <p class="shot-desc">人机审批拦截交互：终端内 Agent 触发高危操作提示确认（Do you want to run...? [y/n]），左侧面板锁定 WaitingApproval，展示安全防护。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1000x500 (2:1)</span>
    <span>目标路径: docs/images/agent-waiting-approval-terminal.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 第 3 部分：项目模式（扁平化多 Agent 研讨）

**项目模式（Project Mode）** 将单个项目文件夹内的多个 Agent 组织为一个平等的研讨协作小组，无需任何 Git 仓库即可工作：

- **激活模式**：在左侧面板顶部点击 **项目（Project）** 按钮。
- **跨 Agent 点对点通信**：Agent 之间通过 `warp_agent_list` 互相感知，通过 `warp_agent_ask` 发起提问，通过 `warp_agent_reply` 获取回复。
- **共享全局上下文**：各 Agent 通过 `warp_context_publish` 发布技术方案与数据结构，其他 Agent 通过 `warp_context_get` 随时提取同步。

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

---

## 第 4 部分：工作树模式与 Coordinator 任务分发

在实际软件开发中，多个 Agent 编辑同一个分支必然引发 Merge 冲突和编译混乱。**工作树模式（Worktree Mode）** 让每个 Agent 分别运作在独立的 Git 物理目录中。

### 1. 快速创建独立工作树
```bash
git worktree add ../project-api feature/api
git worktree add ../project-ui feature/ui
git worktree add ../project-tests feature/tests
```

### 2. 选举 Coordinator（协调者）
1. 在协同面板顶部切换为 **工作树（Worktree）** 模式。
2. 从 **Coordinator** 下拉列表中选择一名具备技术把控力的核心 Agent。
3. Coordinator 会自动统揽所有分支状态、任务分配与交叉审阅流程。

<div class="docs-shot-placeholder" data-shot="SHOT-02">
  <div class="shot-header">
    <span class="shot-badge">SHOT-02 (P0)</span>
    <span class="shot-title">Worktree Mode with Coordinator</span>
  </div>
  <p class="shot-desc">Worktree 并行研发核心流：顶部选定 Coordinator，展示 feature/api、feature/ui、feature/test 三个物理分支 Worktree 各自独立绑定不同 Agent 并行施工。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/warpai-worktree-coordinator.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

### 3. 分派任务与拓扑依赖
开发者或 Coordinator 可直接创建携带输入上下文、产出定义与前置拓扑依赖的结构化任务。

<div class="docs-shot-placeholder" data-shot="SHOT-16">
  <div class="shot-header">
    <span class="shot-badge">SHOT-16 (P1)</span>
    <span class="shot-title">Task Assignment &amp; Pool Dialog</span>
  </div>
  <p class="shot-desc">Coordinator 任务分发对话框：展示任务标题、指派目标分支、详细说明与前置拓扑依赖设置界面。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1100x700</span>
    <span>目标路径: docs/images/worktree-task-assignment.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

---

## 第 5 部分：对等评审（Peer Review）与主干集成流水线

1. **提交交付**：Worker 编码完毕后，调用 `warp_task_submit` 提交附带 Git Diff 的成果。
2. **指派审阅**：Coordinator 调度另一名同伴 Agent 担任 Reviewer（`warp_task_review`）。
3. **审阅裁决**：
   - `needs_changes`：Reviewer 发现问题驳回，附带具体修回清单；Worker 自动接收并开始修复。
   - `accepted`：Reviewer 验证通过，给予好评。
4. **主干集成**：Coordinator 调用 `warp_task_integrate`，将分支合并入主干并执行自动化测试，生成整洁规范的 Git Commit。

<div class="docs-shot-placeholder" data-shot="SHOT-03">
  <div class="shot-header">
    <span class="shot-badge">SHOT-03 (P0)</span>
    <span class="shot-title">Peer Review &amp; Integration Verdict</span>
  </div>
  <p class="shot-desc">同行审查闭环：展示 Worker 提交任务后，Reviewer Agent 给出审查意见（Accepted / Needs Changes），右侧展现 Git 差异审查面板高亮对比。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1440x810 (16:9)</span>
    <span>目标路径: docs/images/warpai-peer-review-diff.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>
