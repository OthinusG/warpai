---
layout: docs
lang: zh
title: "Worktree 编码模式：Git 并行代码开发完全手册"
description: "专为 Git 代码仓库打造的大规模多 Agent 并行开发手册：物理隔离独立工作树、多分支并行开发各自独立功能、Coordinator 统筹调度、自动化对等评审与主干合并闭环。"
section: 使用指南
doc_id: worktree-mode
prev_url: /docs/project-mode.html
prev_title: "Project 模式：全向协同与泛行业实战手册"
next_url: /docs/using-warpai-agents.html
next_title: 多 Agent 协同工作流
---

在严肃的软件工程中，让多个 Agent 同时读写同一个 Git 仓库目录，必然引发严重的 Git 锁争用、Merge 冲突、脏文件覆盖与编译器缓存踩踏（如 `target/` 或 `node_modules/` 混乱）。

**Warpai 的 Worktree 编码模式将每个功能特性分配到独立的物理 Git Worktree 中**：
- **除协调者（Coordinator）统筹全局外，每个 Worktree 可以完全独立、并行地开发不同的功能特性**。
- **物理目录与编译缓存彻底隔离**：Worker A 在 `feature/api` 上的重构绝不会影响 Worker B 在 `feature/ui` 上的开发与单测。
- **工业级对等评审（Peer Review）闭环**：代码提交需经过同伴审阅（Accepted / Needs Changes），最后由 Coordinator 统一合并进主干。

---

## 架构模型：协调者与独立分支并行开发

```
                    ┌────────────────────────┐
                    │  Coordinator (主协调者) │
                    │   (统筹规划 & 代码集成)  │
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

1. **Coordinator（协调者）**：掌控全局需求，拆解任务，设定任务间的拓扑依赖，监督各分支进度，主持同行评审并执行最终的主干合并。
2. **Worker Agents（工作者）**：各自运行在独立的 Git 物理目录中，专注于独立特性的编码实现、本地测试验证与证据链收集。

---

## 完整使用指南（How to Use）

### 第 1 步：创建物理工作树（Worktree）
在项目根目录中，为不同功能模块创建独立的物理工作树分支：
```bash
# 为后端 API 创建独立工作树
git worktree add ../my-project-api feature/api

# 为前端 UI 组件创建独立工作树
git worktree add ../my-project-ui feature/ui

# 为端到端集成测试创建独立工作树
git worktree add ../my-project-tests feature/tests
```
每个工作树将在 Warpai 中以独立的垂直标签页展示，并锁定其专属工作目录。

### 第 2 步：分配并启动独立 Agent
在各自工作树标签页中，启动对应擅长的 CLI Agent：
- 标签页 1（`../my-project-api`）：运行 `claude` 或 `codex`
- 标签页 2（`../my-project-ui`）：运行 `cursor-agent` 或 `trae`
- 标签页 3（`../my-project-tests`）：运行 `agy` 或 `gemini`

### 第 3 步：选举 Coordinator（协调者）
1. 在左侧协同面板顶部，切换至 **工作树（Worktree）** 模式。
2. 从 **Coordinator** 下拉菜单中选择一名具备全局架构能力的 Agent。
3. Coordinator 会自动扫描所有活动分支与工作树中 Agent 的运行状态。

### 第 4 步：需求拆解与并行任务分发
向 Coordinator 键入顶层需求：
> *“我们需要为电商系统开发‘订单导出至 Excel’功能。请拆解任务：由 Worker A 负责后端数据导出 API，由 Worker B 负责前端导出弹窗与下载按钮，由 Worker C 编写端到端回归测试用例。要求分支物理隔离开发，完成后互相评审并集成。”*

Coordinator 将调用内置 MCP 工具（`warp_task_assign`）向各分支精准下发工单：
```json
{
  "task_id": "task-api-export",
  "target_branch": "feature/api",
  "title": "实现后端订单 Excel 导出接口",
  "dependencies": []
}
```

### 第 5 步：独立并发开发与本地证据收集
各 Worker 收到工单后，在其独立物理副本中编写代码与测试：
- Worker A 修改 Rust/Go 后端代码，`cargo check` 仅编译本地 `target/`，丝毫不影响 Worker B 的前端构建。
- 测试通过后，Worker 调用 `warp_evidence_add` 上报测试通过日志与截图。

### 第 6 步：自动化代码对等评审（Peer Review）
功能编写完毕后，触发严格的同行评审：
1. **提交成果**：Worker 调用 `warp_task_submit` 提交 Git Diff。
2. **分配 Reviewer**：Coordinator 调用 `warp_task_review` 指派另一名空闲的 Agent 担任审阅员。
3. **审阅判决**：
   - `needs_changes`：审阅员发现性能问题或代码异味，打回修回；Worker 自动接收具体意见并重构。
   - `accepted`：审阅员确认验证通过，签署准入许可。

### 第 7 步：Coordinator 主干整合与交付
所有并行子任务全部通过验收后：
1. Coordinator 调用 `warp_task_integrate`。
2. Warpai 自动化将已验证分支优雅合并至 `main`，执行全量自动化回归测试。
3. 生成整洁规范的 Git Commit 记录，完成本次多智能体并行功能交付。
