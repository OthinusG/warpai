---
layout: docs
lang: zh
title: 原生 MCP 协同工具体系
description: "Warpai 内置的 38+ 项本地 MCP 协议工具（warpai-local-com）技术参考：覆盖任务生命周期、对等代码审阅、并发文件锁定与智能体通信。"
section: Agent 生态
doc_id: mcp-api
prev_url: /docs/cli-agents.html
prev_title: 20+ 款 CLI Agent 配置指南
next_url: /docs/config.html
next_title: 配置项与外观主题
---

**Model Context Protocol (MCP)** 是连接不同独立 CLI Agent、将其熔铸为高效研发团队的开放协同通信规范。

在 Warpai 中开启 Agent Communication 功能后，内置的 `warpai-local-com` 本地 MCP 服务将通过标准 JSON-RPC 2.0 协议暴露 **38+ 项原生工具**，用于任务委派分发、对等代码审阅、发布技术规范契约与协同文件加锁。

## MCP 服务名称

每个 Agent 运行环境使用独立的 MCP 服务名称：

| Agent 运行环境 | MCP 服务名称 |
| --- | --- |
| 本地桌面端 | `warpai-local-com` |
| SSH 远端账号 | `warpai-ssh-com` |
| 直连 WSL 发行版及用户 | `warpai-wsl-com` |

---

## 1. 任务生命周期、排队机制与动态监督工具集

### `warp_task_assign`
将一项结构化任务委派指派给指定的 Agent 或特定的 Git Worktree 物理分支。
- **输入参数**：
  - `to` (字符串，必填)：目标接收 Agent 的唯一标识名。
  - `description` (字符串，必填)：完整技术规范、约束条件与预期交付物。
  - `acceptance` (字符串，必填)：明确的验收准则与自动化验证指令。
  - `context_refs` (ContextRef 数组，选填)：绑定的上下文快照引用（上限 16 项）。
  - `reviewer` (字符串，选填)：指定代码审阅同伴 ID（缺省为指派者本人）。
  - `dependencies` (字符串数组，选填)：前置依赖的任务 ID 列表。
  - `start_deadline` (字符串，选填)：ISO-8601 启动截止时间。
  - `execution_timeout_seconds` / `review_timeout_seconds` (整数，选填)：执行与审阅超时设定。
  - `request_id` (字符串，必填)：幂等性防重 UUID。
- **返回数据**：`{ "id": "uuid", "state": "queued" | "blocked", "delivery": "queued", ... }`
- **任务排队语义**：调用 `warp_task_assign` **仅负责任务入队**，不会阻塞发起方，亦不等待执行完成。任务将持久化写入接收方 Agent 的专属任务队列（`state: "queued"`）。发起方可立即继续向该 Agent 或其他同伴追加派发更多任务，所有新派发任务自动加入发起方的动态监督列表。

### `warp_task_start`
从任务队列中提取并正式启动执行指定的任务修订版。
- **输入参数**：
  - `task_id` (字符串，必填)：要启动的任务唯一 ID。
  - `revision` (整数，必填)：任务修订版本号（初始为 1）。
  - `expected_version` (整数，选填)：乐观锁并发版本校验。
  - `request_id` (字符串，必填)：幂等性防重 UUID。
- **返回数据**：`{ "id": "uuid", "state": "running", "attempt_id": "uuid", ... }`
- **单任务独占保护**：每个 Agent 在同一时刻**只能处于执行一项任务的状态**（`state == "running"`）。若当前已有正在运行的任务，调用启动将被严正拦截（`"An assigned task is already running"`）。队列中的其他后续任务将按照到达顺序安全排队，等待当前任务完成或提交后依次取出执行。

### `warp_task_create_pool`
在项目公共任务池中创建一项未指派的具体任务（`state: "queued"`），面向符合条件的同伴开放认领。
- **输入参数**：`description`、`acceptance`、`context_refs`、`eligible`（有资格认领的 Agent ID 列表）、`reviewer`、`dependencies`、`request_id`。
- **返回数据**：`{ "id": "uuid", "state": "queued", ... }`

### `warp_task_claim`
Agent 主动从公共任务池中认领任务，并将其直接从 `queued` 转换为 `running` 状态。
- **输入参数**：`task_id` (字符串，必填)、`expected_version` (整数，选填)、`request_id` (字符串，必填)。
- **独占约束**：仅当认领方当前没有任何其他正在运行的任务时，才允许认领入场。

### `warp_task_list`
查询当前 Agent 可见的所有任务列表，支持按状态或负责人筛选（`state`、`assignee`、`cursor`、`limit`）。
- **动态刷新**：刷新跨全部执行者（包含离线同伴）的动态监督集，保留未决任务，已处理完成的终态任务自动退场。

### `warp_task_get` / `warp_task_history`
读取任务的完整监督详情（最新进度、阻塞提问、截止时间、执行不确定性）与结构化事件时序历史。

### `warp_task_progress`
在不变更所有权的前提下，向正在运行的任务追加里程碑进展、说明或阻塞原因。
- **输入参数**：`task_id`、`revision`、`note`、`wait_reason`、`request_id`。

### `warp_task_submit`
Worker 完成功能实现与自测验证后，提交交付物以进入对等审阅（Peer Review）。
- **输入参数**：
  - `task_id` (字符串，必填)
  - `revision` (整数，必填)
  - `result` (字符串，必填)：核心变更内容与实现逻辑摘要
  - `evidence` (字符串，必填)：单元测试运行日志或验证输出
  - `evidence_ids` (字符串数组，选填)：结构化证据描述符列表
  - `request_id` (字符串，必填)
- **返回数据**：`{ "id": "uuid", "state": "submitted" }`
- **执行槽释放**：提交后当前任务转为 `submitted`，Worker 的执行槽位即刻释放，允许通过 `warp_task_start` 提取并执行队列中的下一项任务。

### `warp_task_review`
记录对提交任务的代码审阅决议与详细反馈。
- **输入参数**：
  - `task_id` (字符串，必填)
  - `revision` (整数，必填)
  - `accepted` (布尔值，必填)：`true` 为验收通过；`false` 为需要修回修改
  - `feedback` (字符串，必填)：明确可落地的修回评审建议
  - `request_id` (字符串，必填)
- **返回数据**：`{ "id": "uuid", "state": "accepted" | "queued", ... }`
- **返工重排队机制**：若 `accepted == false`，系统自动将任务版本号递增（revision + 1），并将该任务重新送入执行者的任务队列（`state: "queued"`），等待 Worker 排期返工。

### `warp_task_integrate` *(Coordinator 专属权限)*
将验收通过的功能分支代码合并入主工程，并记录交付。
- **输入参数**：`task_id` (字符串，必填)、`commit` (字符串，选填)、`request_id` (字符串，必填)。
- **返回数据**：`{ "id": "uuid", "state": "integrated" }`

### `warp_task_cancel` / `warp_task_finish_cancel` / `warp_task_fail` / `warp_task_retry` / `warp_task_reassign` / `warp_task_set_dependencies`
全生命周期管理工具：
- `warp_task_set_dependencies`：定义前置依赖。未满足前置条件的任务自动进入 `state: "blocked"`，前置任务全体验收后自动解封转为 `state: "queued"`。
- `warp_task_cancel`：发起取消申请；运行中任务收到终止请求后由 Worker 通过 `warp_task_finish_cancel` 确认停止。
- `warp_task_fail`：报告不可逆执行失败并附带现场诊断材料。
- `warp_task_retry` / `warp_task_reassign`：对失败或超时的任务进行重试或换人，重新进入 `state: "queued"`。

---

### 任务排队机制与动态监督全景架构

Warpai 依托底层轻量 SQLite 状态总线，构建了高并发、强秩序的多 Agent 任务排队机制：

1. **持久化任务队列与 FIFO 串行消费**：所有派发的任务均持久化进入接收方队列（`state: "queued"`）。当 Worker A 正在执行任务 1 时，追加派发的任务 2、任务 3 将在队列中按到达顺序稳定候补。Worker A 完成前序任务并提交后，通过 `warp_task_start` 依次出队执行，杜绝了多任务并行修改引发的 Git 索引竞争、工作区脏写与大模型上下文溢出。
2. **随时动态派发与无阻塞入队**：Coordinator 或主 Agent 不必在启动前预设固定批次，可在项目推进的任意时刻随时向一个或多个同伴派发任务。派发只负责入队（`delivery: "queued"`），派发方无需等待即可持续下发后续工作，保持生产流水线高饱和运转。
3. **全队列动态生命周期追踪**：所有新入队任务自动进入主 Agent 的动态监督列表（Dynamic Supervision Set）。主 Agent 即使通过 `warp_agent_ready` 出让控制轮次，也能通过 `warp_task_list` 和 `warp_task_get` 实时掌控队列深度、阻塞提问与运行进展。
4. **依赖自动解封与返工重排队**：带前置依赖的任务在依赖未完备时处于 `blocked`，一旦前置通过立即自动解封为 `queued`；审阅打回（`needs_changes`）的任务自动版本升级并重回 `queued`，平滑衔接下一轮执行。
5. **终态主动出队归档**：任务一旦通过验收（`accepted`）或终态归档，便主动从在途监督列表中退出，无需繁复的僵尸轮询，确保监控面板始终精准聚焦核心活跃事项。

<div class="docs-shot-placeholder" data-shot="SHOT-26">
  <div class="shot-header">
    <span class="shot-badge">SHOT-26 (P1)</span>
    <span class="shot-title">任务排队与动态监督列表实景</span>
  </div>
  <p class="shot-desc">任务排队与动态监督：展示主 Agent 随时派发多任务进入队列（queued 状态），Worker 串行执行（running 状态），以及完成审阅后主动出队归档的全生命周期。</p>
  <div class="shot-meta">
    <span>建议尺寸: 1200x650</span>
    <span>目标路径: docs/images/mcp-dynamic-task-supervision.png</span>
    <span class="shot-tag">截图待补充 · 对应表格 docs/SCREENSHOT-INVENTORY.xlsx</span>
  </div>
</div>

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
