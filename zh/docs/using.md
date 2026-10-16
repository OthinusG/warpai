---
layout: docs
lang: zh
title: 使用指南总览
description: "Warpai 核心工程工作流完全指南：Git Worktree 并行代码开发、Project 全向网状协同、Coordinator 任务委派、自动化对等评审及内置工作台。"
section: 使用指南
doc_id: using
prev_url: /docs/features.html
prev_title: 全景特性与原版对比
next_url: /docs/using-terminal.html
next_title: 终端与命令块深入
---

本指南全面介绍 Warpai 在真实工程与跨学科场景中的核心工作流：从跨 Git Worktree 并行调度多 Agent 研发小队、在 Project 模式下组织泛行业网状协作，到终端内置代码编辑、自动化对等评审（Peer Review）与远端 SSH 开发。

---

## 典型工作流 1：面向 Git 仓库的 Worktree 并行开发模式

针对严肃的代码工程项目，Warpai 提供工业级的 **工作树模式（Worktree Mode）**。通过将每个功能特性分配到独立的物理 Git Worktree 中，多个 CLI Agent 可完全并发编写代码，互不污染彼此的分支、文件锁与编译器缓存。

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   1. 在协同面板顶部切换至“工作树模式”（Worktree Mode）                      │
│   2. 为不同模块创建独立的物理工作树（如 api、ui、tests）                     │
│   3. 在对应工作树标签页中分别启动你的 CLI Agent                             │
│   4. 在下拉列表中选择你的 Coordinator（协调者）Agent                        │
│   5. Coordinator 统筹需求，将子任务与上下文分派给各个独立分支               │
│   6. 各 Worker 在物理隔离环境中并行编码，完成后提交差异并进行对等评审       │
│   7. Coordinator 汇总验收，验证通过后整合代码并完成最终 Git 提交            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 分步操作指南：

#### 1. 为功能特性创建物理工作树
运行标准 Git 命令或使用工作树快捷向导：
```bash
git worktree add ../my-project-api feature/rest-api
git worktree add ../my-project-ui feature/frontend-components
git worktree add ../my-project-tests feature/integration-tests
```
每个工作树将自动在 Warpai 的独立垂直标签页中打开，并锁定其物理工作目录。

#### 2. 在各自工作树中启动 Agent
切换至对应标签页并启动你选定的 Agent：
- 在 `../my-project-api` 标签页：启动 `claude` 或 `codex`
- 在 `../my-project-ui` 标签页：启动 `cursor-agent` 或 `trae`
- 在 `../my-project-tests` 标签页：启动 `gemini` 或 `agy`

#### 3. 指定 Coordinator（协调者）
在左侧协同面板中：
1. 将模式切换开关切换为 **工作树（Worktree）**。
2. 点击 **Coordinator** 下拉菜单，选择具备架构全局掌控力的核心 Agent。
3. 选定后，Coordinator 将获得所有活动分支与工作树中 Agent 的全局全景视图。

#### 4. 派发需求与监督执行
向 Coordinator 键入需求指令：
> *“我们需要为系统增加数据导出功能。请拆解任务：将后端 API 接口委派给 Worker A，前端下载组件委派给 Worker B，集成测试套件委派给 Worker C。组织他们互相进行代码对等评审，验收通过后合并交付。”*

Coordinator 将通过 Warpai 内置的 MCP 工具（`warp_task_assign`）向各个 Worker 派发结构化任务。各 Worker 在空闲时会自动收到任务并展开工作。

---

## 典型工作流 2：面向泛行业的 Project 全向网状协同模式（重点扶持）

对于非 Coding 场景（科研论文写作、视觉与多媒体设计、量化与金融分析、专利挖掘与法律合规、新媒体矩阵运营、商业尽职调查等），纯 Git 分支模型往往显得过于沉重和局限。Warpai 重点扶持并提供了 **项目模式（Project Mode）**：

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   1. 在协同面板顶部切换至“项目模式”（Project Mode）                          │
│   2. 无需 Git 仓库，在本地任意工作空间或同步盘中直接启动多个 Agent          │
│   3. 打破单一指挥链：所有 Agent 既可以作为指挥官也可以作为工人              │
│   4. 任意 Agent 之间全向双向互发消息（warp_agent_send / reply）             │
│   5. 遇到专业盲区时发起结构化提问与应答（warp_agent_ask / answer）          │
│   6. 通过公共任务池自由认领任务并流转（warp_task_create_pool / claim）     │
│   7. 阶段性产出发布为全局共享上下文契约（warp_context_publish / get）        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

- **全向消息与任务互通**：所有 Agent 都是平等节点。在科研场景中，文献检索 Agent 既可以向论文撰写 Agent 提供引用，也可以向数据绘图 Agent 提议图表样式；模拟审稿人 Agent 更可直接向主笔 Agent 下发修回指令。
- **动态角色随时切换**：在初始策划阶段 Agent A 是指挥官（Commander），在专业验收阶段 Agent B 则切换为指挥官，实现真正意义上的多智能体自由闭环。
- **广泛赋能跨学科领域**：详见 [多 Agent 协同工作流]({{ '/zh/docs/using-warpai-agents.html' | relative_url }}) 中的六大实战落地手册。

---

## 典型工作流 3：自动化代码对等评审与集成管道

在工作树模式下，Warpai 在各 Agent 之间构建了全闭环的对等代码评审（Peer Review）机制：

1. **成果提交**：当 Worker 完成功能编写后，通过 `warp_task_submit` 提交测试材料与 Diff 补丁。
2. **指派审阅**：Coordinator 通过 `warp_task_review` 指派另一名同伴 Agent 担任 Reviewer。
3. **审阅决议**：
   - 若发现逻辑缺陷或风格问题，Reviewer 将任务标记为 `needs_changes` 并附带具体修改建议。Worker 的收件箱会自动收到返修提醒并开始修正。
   - 验证无误后，Reviewer 将任务标记为 `accepted`。
4. **统一整合**：Coordinator 调用 `warp_task_integrate`，将特性分支合并入主干，运行全量验证测试并创建标准 Git Commit。

---

## 典型工作流 4：无缝利用终端内置开发者工作台

双手无须离开键盘，告别繁重窗口来回跳转：
- **项目文件树（Project Explorer）**：在终端侧边浏览、新建、重命名和管理目录文件。
- **轻量代码编辑器与 Vim 模态**：在应用内直接编辑配置文件与脚本，支持 `:w`、`:q` 等标准 Vim 快捷键。
- **实时 Markdown 渲染预览**：即时渲染排版规整的技术文档、表格与架构图。
- **原生 Git 差异对比审阅**：在提交代码前直观查看增删差异，高亮对比无遗漏。
