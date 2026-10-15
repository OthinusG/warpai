---
layout: docs
lang: zh
title: 使用指南总览
description: "Warpai 核心工程工作流完全指南：Git Worktree 并行开发、Coordinator 任务委派、自动化对等评审及内置工作台。"
section: 使用指南
doc_id: using
prev_url: /docs/features.html
prev_title: 全景特性与原版对比
next_url: /docs/using-terminal.html
next_title: 终端与命令块深入
---

本指南全面介绍 Warpai 在真实工程场景中的核心工作流：从跨 Git Worktree 并行调度多 Agent 研发小队，到终端内置代码编辑、自动化对等评审（Peer Review）与远端 SSH 开发。

---

## 典型工作流 1：借助 Git Worktree 进行大规模并行开发

Warpai 最具生产力的杀手级工作流是 **工作树模式（Worktree Mode）**。通过将每个功能特性分配到独立的物理 Git Worktree 中，多个 CLI Agent 可完全并发编写代码，互不污染彼此的文件系统与编译器缓存。

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   1. 在协同面板顶部切换至“工作树模式”（Worktree Mode）                      │
│   2. 为不同模块创建独立的物理工作树（如 api、ui、tests）                     │
│   3. 在对应工作树标签页中分别启动你的 CLI Agent                             │
│   4. 在下拉列表中选择你的 Coordinator（协调者）Agent                        │
│   5. Coordinator 将分解后的子任务与上下文分派给各个 Worker                  │
│   6. 各 Worker 在物理隔离环境中并行编码，完成后提交差异并进行对等评审       │
│   7. Coordinator 汇总验收，验证通过后整合代码并完成最终 Git 提交            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 分步操作指南：

#### 1. 为功能模块创建工作树
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
1. 将模式切换开关从 **项目（Project）** 切换为 **工作树（Worktree）**。
2. 点击 **Coordinator** 下拉菜单，选择具备架构全局掌控力的核心 Agent。
3. 选定后，Coordinator 将获得所有活动分支与工作树中 Agent 的全局全景视图。

#### 4. 派发需求与监督执行
向 Coordinator 键入需求指令：
> *“我们需要为系统增加数据导出功能。请拆解任务：将后端 API 接口委派给 Worker A，前端下载组件委派给 Worker B，集成测试套件委派给 Worker C。组织他们互相进行代码对等评审，验收通过后合并交付。”*

Coordinator 将通过 Warpai 内置的 MCP 工具（`warp_task_assign`）向各个 Worker 派发结构化任务。各 Worker 在空闲时会自动收到任务并展开工作。

---

## 典型工作流 2：自动化代码对等评审与集成管道

Warpai 在各 Agent 之间构建了全闭环的对等代码评审（Peer Review）机制：

1. **成果提交**：当 Worker 完成功能编写后，通过 `warp_task_submit` 提交测试材料与 Diff 补丁。
2. **指派审阅**：Coordinator 通过 `warp_task_review` 指派另一名同伴 Agent 担任 Reviewer。
3. **审阅决议**：
   - 若发现逻辑缺陷或风格问题，Reviewer 将任务标记为 `needs_changes` 并附带具体修改建议。Worker 的收件箱会自动收到返修提醒并开始修正。
   - 验证无误后，Reviewer 将任务标记为 `accepted`。
4. **统一整合**：Coordinator 调用 `warp_task_integrate`，将特性分支合并入主干，运行全量验证测试并创建标准 Git Commit。

---

## 典型工作流 3：面向调研、推演与技术写作的平面协作

对于非代码工程任务（学术文献综述、竞品调研、商业策划书撰写、系统架构设计），请使用 **项目模式（Project Mode）**：

- **无需 Git 仓库**：在本地任意文件夹或网盘同步盘中均可即时开启。
- **共享全局上下文**：各 Agent 通过 `warp_context_publish` 共享研究笔记、参考文献与阶段性结论。
- **协同澄清疑难**：负责检索的 Agent 可通过 `warp_agent_ask` 向负责事实核查的 Agent 发起询问，并通过 `warp_agent_reply` 获取结构化答复。

---

## 典型工作流 4：无缝利用终端内置开发者工作台

双手无须离开键盘，告别繁重窗口来回跳转：
- **项目文件树（Project Explorer）**：在终端侧边浏览、新建、重命名和管理目录文件。
- **轻量代码编辑器与 Vim 模态**：在应用内直接编辑配置文件与脚本，支持 `:w`、`:q` 等标准 Vim 快捷键。
- **实时 Markdown 渲染预览**：即时渲染排版规整的技术文档、表格与架构图。
- **原生 Git 差异对比审阅**：在提交代码前直观查看增删差异，高亮对比无遗漏。
