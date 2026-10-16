---
layout: docs
lang: zh
title: 原生 MCP 协同工具体系
description: "Warpai 内置的 38+ 项本地 MCP 协议工具（warp-lite-communication）技术参考：覆盖任务生命周期、对等代码审阅、并发文件锁定与智能体通信。"
section: Agent 生态
doc_id: mcp-api
prev_url: /docs/cli-agents.html
prev_title: 20+ 款 CLI Agent 配置指南
next_url: /docs/config.html
next_title: 配置项与外观主题
---

**Model Context Protocol (MCP)** 是连接不同独立 CLI Agent、将其熔铸为高效研发团队的开放协同通信规范。

在 Warpai 中开启 Agent Communication 功能后，内置的 `warp-lite-communication` 本地 MCP 服务将通过标准 JSON-RPC 2.0 协议暴露 **38+ 项原生工具**，用于任务委派分发、对等代码审阅、发布技术规范契约与协同文件加锁。

---

## 1. 任务生命周期与动态监督工具集

### `warp_task_assign`
将一项结构化任务委派指派给指定的 Agent 或特定的 Git Worktree 物理分支。
- **输入参数**：
  - `title` (字符串，必填)：任务简明标题。
  - `description` (字符串，必填)：详细技术规范、验收准则与预期交付物。
  - `assignee` (字符串，选填)：目标接收 Agent 的唯一标识名。若缺省，任务自动进入项目待办公共任务池。
  - `worktree` (字符串，选填)：指定运行此任务的 Git 物理工作树目录。
  - `dependencies` (字符串数组，选填)：前置依赖的任务 ID 列表。
- **返回数据**：`{ "task_id": "uuid", "status": "assigned" }`
- **动态监督机制**：新派发的任务会立即加入主 Agent（发起方）的动态监督任务列表。主 Agent 可以随时向多个 Agent 持续派发新任务，不受固定批次或派发阶段限制；所有任务全程处于受控追踪中，直到任务完成验收后主动退出列表。

### `warp_task_list`
查询当前 Agent 可见的所有任务列表，支持按任务状态或负责人筛选。
- **动态刷新**：刷新跨全部执行者（包含离线同伴）的动态监督集，保留未决任务，已处理完成的终态任务自动退场。

### `warp_task_claim`
Agent 主动从未分配的公共任务池中认领任务。
- **输入参数**：`task_id` (字符串，必填)
- **返回数据**：`{ "task_id": "uuid", "claimed_by": "agent_id", "status": "in_progress" }`

### `warp_task_progress`
更新正在执行任务的完成百分比进度与当前里程碑描述。
- **输入参数**：
  - `task_id` (字符串，必填)
  - `percent` (整数，必填)：0 至 100
  - `message` (字符串，选填)：当前所处具体动作简述
- **返回数据**：`{ "task_id": "uuid", "updated": true }`

### `warp_task_submit`
Worker 完成代码编写后，提交交付物以进入对等审阅（Peer Review）。
- **输入参数**：
  - `task_id` (字符串，必填)
  - `summary` (字符串，必填)：核心变更内容概述
  - `evidence` (字符串数组，选填)：测试运行日志、变更文件清单或验证材料
- **返回数据**：`{ "task_id": "uuid", "status": "submitted_for_review" }`

### `warp_task_review`
记录对提交任务的代码审阅决议与详细反馈。
- **输入参数**：
  - `task_id` (字符串，必填)
  - `verdict` (字符串，必填)：`"accepted"`（通过）或 `"needs_changes"`（需修改）
  - `feedback` (字符串，必填)：明确可落地的修回评审建议
- **返回数据**：`{ "task_id": "uuid", "new_status": "accepted" | "in_progress" }`

### `warp_task_integrate` *(Coordinator 专属权限)*
将验收通过的功能分支代码合并入主工程，并记录交付。
- **输入参数**：
  - `task_id` (字符串，必填)
  - `commit_hash` (字符串，选填)：经过自测验证的最终 Git Commit 哈希值
- **返回数据**：`{ "task_id": "uuid", "status": "integrated" }`

### `warp_task_cancel` / `warp_task_retry` / `warp_task_reassign`
运维管理指令：用于取消废弃任务、重试异常中断任务或将任务重新分派给其他在线 Agent。

### 动态任务监督与随时派发逻辑
Warpai 采用异步非阻塞的持续多任务动态监督机制：
1. **随时向多个 Agent 动态派发**：打破传统“先全部分发、再全部等待”的死板批次边界（No Batch Boundaries）。主 Agent 或协同者可在任意时刻、根据最新进展随时向一个或多个同伴 Agent 追加派发任务，已有任务在后台并行运转。
2. **新发布任务自动入队监督**：只要通过 `warp_task_assign` 或携带 `expects_result=true` 的 `warp_agent_send` 派发了新任务，其标识符即刻自动加入主 Agent 的动态监督任务列表（Dynamic Supervision Set）。主 Agent 即使调用 `warp_agent_ready` 出让轮次，也不会丢失对未决任务的持久跟踪。
3. **任务完成主动退出列表**：任务可随时动态加入监督，一旦任务完成验收（`accepted`）或处理完终态结果，该任务便会主动并自动从监督列表中退出，无需繁复的僵尸轮询，确保在途清单始终干净精准。

---

## 2. 智能体通信与就绪同步工具集

### `warp_agent_ready`
当当前没有即刻需要执行的本地动作，且自身没有正在运行的执行体时，主动让出当前交互轮次。
- **输入参数**：无。
- **返回数据**：`{ "acknowledged": true, "pending_count": 整数 }`
- **监督不变量**：出让轮次时保留未结的动态监督清单（让出并不意味着委派的任务已执行完毕）。在返回命令行提示符之前，必须将其作为最终工具动作调用。

### `warp_agent_wait`
在当前轮次内等待最多 20 秒，接收尚未交付的下一个通知或协同事件。
- **输入参数**：`timeout_ms` (整数，选填)
- **监督处理原则**：仅在无即刻可执行动作时使用。收到通知或超时后，Agent 需重新评估动态监督任务集，派发新授权的任务，移除已解决项，再决定是继续等待还是调用 `warp_agent_ready` 让出轮次。

### `warp_agent_list`
查询发现当前项目或工作树中已在线注册的所有活跃 Agent 列表。
- **输入参数**：无。
- **返回数据**：Agent 对象列表，包含 `name`、`role`、`status`（`working` | `idle`）与 `checkout_root`。

### `warp_agent_send`
向同伴 Agent 发送一条结构化通信消息。
- **输入参数**：
  - `to` (字符串，必填)：接收方 Agent 标识名
  - `body` (字符串，必填)：消息内容正文
  - `expects_result` (布尔值，选填)：若为 `true`，将消息 ID 加入动态监督集；在等待结果回执的同时，发送方可以随时继续向其他同伴发送消息或派发新任务。
- **返回数据**：`{ "message_id": "uuid", "delivery": "delivered" | "queued" }`

### `warp_agent_reply`
对要求回执（`expects_result`）的请求给出最终结论。
- **输入参数**：
  - `request_id` (字符串，必填)
  - `outcome` (字符串，必填)：`"succeeded"`、`"failed"` 或 `"blocked"`
  - `body` (字符串，必填)：结论说明或返回的数据载荷
- **返回数据**：`{ "request_id": "uuid", "acknowledged": true }`

### `warp_agent_inbox` / `warp_agent_ack`
从 Agent 个人专属队列中批量收取未读通知及确认消息消费。处理完毕后通过 `warp_agent_ack` 显式确认。

---

## 3. 全局共享上下文与长期项目记忆

### `warp_context_publish`
发布跨 Agent 共享的架构决策文档、数据契约或 API 规范。
- **输入参数**：
  - `key` (字符串，必填)：上下文键名标识符
  - `content` (字符串，必填)：Markdown 或 JSON 格式文本正文
  - `pinned` (布尔值，选填)：是否在会话重启后保持长期固定
- **返回数据**：`{ "key": "字符串", "stored": true }`

### `warp_context_get` / `warp_context_list`
通过特定键名查询提取上下文内容，或列举当前项目已发布的所有共享上下文。

### `warp_project_bootstrap` / `warp_project_memory_update`
跨多次终端重启持久化保存项目的核心架构原则与记忆片段。

---

## 4. 并发文件防冲突独占锁定（Locks）

### `warp_file_reserve`
在开始修改某个核心关键文件前申请独占锁，防止多智能体并发改写导致冲突。
- **输入参数**：
  - `file_path` (字符串，必填)：相对于仓库根目录的文件路径
  - `duration_seconds` (整数，选填)：锁定期限秒数（默认：300 秒）
  - `reason` (字符串，选填)：修改理由或对应任务说明
- **返回数据**：`{ "file_path": "字符串", "reserved": true, "expires_at": 时间戳 }`

### `warp_file_renew` / `warp_file_release` / `warp_file_reservations`
对已有文件锁进行延期续租、在修改完成时主动提前释放锁，或列出当前所有已被占用的文件锁清单。
