# Warpai Agent-to-Agent Handoff Brief 修改指导

> Original implementation request, preserved below. The implemented contract is
> [HANDOFF.md](HANDOFF.md); source-linked verification and pending gates are in
> [HANDOFF-ACCEPTANCE.md](HANDOFF-ACCEPTANCE.md).

日期：2026-11-03

状态：实现指导，尚未实施。本文中的新增字段及 capability 名称是建议目标契约，不代表当前已经存在。

本文可直接交给实现 Agent。目标是在现有协作架构上增加完整的任务交接能力，复用 Context Resources、任务状态机和传输通道。

## 1. 任务目标与实施边界

为 Warpai 增加 **Handoff Brief**：在 Agent 提交任务结果或明确保存检查点时，由 Broker 将已经存在的交付信息组织成固定版本的 Context Resource，供后续 Agent 按需读取。

交接包应回答：

- 当前任务要解决什么问题。
- 已经完成了什么。
- 涉及哪些文件、commit、产物和证据。
- 做过哪些验证，结果来自谁。
- 为什么采用当前方案。
- 还有哪些问题未解决。
- 接手后应该从哪里继续。
- 这些信息对应哪个任务版本、执行批次和工作区。

实施应覆盖 Local、SSH Companion 和直接 WSL，并保持 macOS、Windows、Linux 的契约一致。

本轮以**已跟踪任务**为交接锚点。普通聊天不自动转换为任务，也不自动生成摘要。已有消息可以携带经过授权的交接引用。

实现约束：

- Warpai 负责确定性整理和引用关联。
- Agent 负责提供决策理由、剩余问题等语义信息。
- 不引入额外模型、后台总结服务、向量数据库或第二套任务系统。
- 不扫描仓库，不读取 vendor transcript，不自动执行 Git、测试或 shell 命令。
- 不复制整个仓库、完整 diff、原始日志或完整聊天记录。
- 不恢复项目已经移除的云 AI、账户、计费或遥测产品面。

## 2. 开始前必须完成的检查

阅读目标工作树中的：

- `AGENTS.md`
- `README.md`
- `MEMORY.md`
- `specs/agent-communication-v2/CONTEXT-RESOURCES.md`
- `specs/agent-communication-v2/PROJECT-MEMORY.md`
- `specs/agent-communication-v2/API.md`
- `specs/agent-communication-v2/PLAN.md`
- 与 Worktree、Companion、SSH、WSL 有关的当前有效契约。

检查分支、工作区状态、已有未提交改动和进行中的任务。

交接功能应使用独立分支或工作树。已有网络修复属于独立工作，禁止把其未提交改动混入本功能。

编码前完成一份项目规格，例如：

```text
specs/agent-communication-v2/HANDOFF.md
```

至少固定：

- 产品范围。
- 数据结构。
- 字段来源与可信度。
- 生成时机。
- 权限与共享规则。
- API 变化。
- 生命周期。
- 版本兼容。
- 容量与清理策略。
- 验收标准。

实现中的契约变化应先更新该文档，不能让 UI、MCP 和 Store 各自形成不同解释。

## 3. 必须检查和修改的组件

以下路径是当前代码入口，实施前仍应核对最新源码。

| 组件 | 主要入口 | 修改职责 |
| --- | --- | --- |
| 共享操作与返回类型 | `crates/agent_bus/src/lib.rs` | 提交参数、交接引用、任务投影 |
| 上下文类型 | `crates/agent_bus/src/context.rs` | 新增交接内容类型、验证和序列化 |
| 上下文存储 | `crates/agent_bus/src/context_storage.rs` | 不可变发布、读取、可见性和引用检查 |
| 任务与事件存储 | `crates/agent_bus/src/storage.rs` | 原子生成、任务关联、幂等、历史和清理 |
| 执行线程与启动上下文 | `crates/agent_bus/src/project_memory.rs`、`crates/agent_bus/src/project_memory_storage.rs` | 暴露相关交接引用，保留执行身份 |
| MCP | `crates/agent_bus/src/mcp.rs` | 工具 schema、描述、发现和接手引导 |
| Companion | `crates/agent_bus/src/companion.rs`、`crates/agent_bus/src/companion_tasks.rs` 等 | 能力协商、共享操作转发与错误传播 |
| 远程协议 | `crates/remote_protocol/` | 仅在现有封装不能承载新增语义时调整 |
| 桌面远程客户端 | `app/src/remote_server/` | capability 检查、断连与迟到响应处理 |
| 通信面板 | `app/src/agent_communication/panel.rs`、`app/src/agent_communication/panel_controls.rs` | 任务详情、历史、选择和接手入口 |
| 历史导出与清理 | 当前 Store 与控制操作 | 保留被引用的数据，避免悬空引用 |
| 测试、打包与文档 | 现有 workflows、specs、manifest | 跨平台验证、Companion 配套交付 |

**Companion 和 MCP 主要做兼容性扩展，不预设重写。** 任务与历史视图继续使用现有数据流，增加交接投影和入口。

## 4. 先固定信息来源，避免虚构“自动理解”

交接内容分为三类。

| 类型 | 允许来源 | 处理方式 |
| --- | --- | --- |
| Broker 可确认的事实 | 任务、执行绑定、已保存证据、状态事件和审查记录 | 自动整理，禁止客户端覆盖 |
| Agent 提供的说明 | 结果摘要、决策、原因、剩余问题和建议下一步 | 结构化接收，保留作者与来源 |
| 当前没有的信息 | 未登记的文件变化、未提交的测试信息、未知工作区状态 | 明确标记缺失，不推断 |

例如：

- 存在测试命令和 Agent 报告的 `passed`，不等于 Warpai 执行过测试。
- 有 commit，不等于工作区没有未提交修改。
- 有文件路径，不等于文件内容仍与交付时一致。
- 任务被接受，不等于所有测试结果都得到了独立核验。
- 空数组不应默认表示“确认没有问题”，也可能是“没有提供信息”。

结构化字段必须能够区分：

```text
provided
not_provided
not_applicable
```

不得通过增加合理猜测来填满交接包。

## 5. 定义交接数据结构

建议增加：

```rust
ContextContent::Handoff { ... }
```

沿用现有资源身份：

```text
ContextRef = { id, revision, sha256 }
```

交接资源至少包含以下内容：

| 字段组 | 内容 | 写入方 |
| --- | --- | --- |
| Schema | `schema_version` | 系统 |
| 任务身份 | task ID、task revision、execution thread ID | 系统 |
| 执行身份 | attempt ID、生产 Agent、对应 run/binding | 系统 |
| 来源定位 | 已确认的 source workspace、已有 source revision 信息 | 系统或明确标注来源 |
| 生成定位 | 原操作 request ID、生成原因、已有记录的采集边界 | 系统 |
| 结果 | 结果摘要、结果记录引用 | Agent 内容与系统关联 |
| 交付证据 | file、commit、diff、test 等现有证据引用 | 系统整理 |
| 决策 | decision、rationale、相关证据引用 | Agent |
| 未解决问题 | issue、影响、状态、需要的决定 | Agent |
| 下一步 | 建议动作与顺序 | Agent |
| 完整性说明 | 哪些信息未提供、哪些列表被分页或省略展示 | 系统与 Agent |
| 上下文关联 | 已授权的固定版本 ContextRef | 系统校验 |

建议将 Agent 输入与系统生成的快照分开定义：

```text
HandoffSubmission
HandoffSnapshot
```

客户端不应提交或覆盖以下系统事实：

```text
project
author identity
attempt ownership
native run binding
source workspace authority
review outcome
system verification status
```

交接包里的“下一步”属于说明数据，不构成执行授权。

## 6. 防止伪造系统交接包

新增 `handoff` 类型后，不能让普通 `context_publish` 调用者直接构造一个看起来由 Broker 生成的交接包。

必须满足：

- 公共 `ContextPublish` 拒绝发布或修改系统保留的 handoff 资源。
- Broker 通过内部受控入口生成交接资源。
- 系统字段来自已认证的调用身份、任务和执行绑定。
- 不允许通过客户端提供的 task ID、作者名或 workspace 字符串替代授权检查。
- 同一内容即使拥有合法 SHA-256，也不能因此被视为经过系统认证的交付。

**SHA-256 只证明内容一致性，不证明内容正确、来源可信或操作获得授权。**

## 7. 扩展已有 API，保持单一事实来源

优先扩展现有操作，不新增一套交接消息协议。

| 操作或投影 | 建议变化 |
| --- | --- |
| `TaskSubmit` | 增加可选结构化 `handoff` 输入 |
| `TaskProgress` | 支持明确保存交接检查点，默认不生成 |
| `TaskFail` | 可携带部分交付与未完成说明 |
| `TaskFinishCancel` | 可携带停止位置和保留产物说明 |
| `AgentReply` | 任务关联场景可携带已有、已授权的交接引用 |
| `TaskGet` | 返回当前相关交接引用及其来源状态 |
| `TaskHistory` | 展示创建、审查、替代和相关生命周期事件 |
| `ExecutionThreadGet` | 返回该执行线程相关的交接引用 |
| `ProjectBootstrap` | 返回紧凑的待接手交接索引 |
| `ContextGet/List` | 支持 handoff 类型及对应可见性 |

旧字段继续有效。不要要求所有旧 Agent 立即提供新的结构化内容。

旧客户端提交结果时：

- 系统可以从已有结果和证据生成基础交接快照。
- 决策、未解决问题等缺失字段标记为未提供。
- 不能因为缺少新增的可选说明字段而拒绝原本合法的提交。

注意：现有输入上下文 `context_refs` 与任务输出 `handoff_ref` 应分别保存。不能把新交接引用自动追加到原输入列表，造成原有固定引用被改变，或把已达到 16 个引用的任务推到上限之外。

## 8. 固定生成时机和生命周期

不要新增一套与任务状态机竞争的交接状态机。交接资源记录快照，任务状态仍由现有任务操作决定。

| 场景 | 应有行为 |
| --- | --- |
| 正常提交 | 生成提交快照，任务进入现有 submitted 状态 |
| 审查接受 | 保留原快照；当前审查投影显示 accepted |
| 审查退回 | 保留旧快照和反馈；后续合法重提交通道产生新版本 |
| 明确检查点 | 生成 checkpoint，任务执行状态按现有规则保留 |
| 阻塞 | 可保存检查点，但不能标记完成或移出监督列表 |
| 任务失败 | 保存部分工作和失败说明，不包装成成功交付 |
| 确认取消完成 | 保存停止位置，保留 cancelled 语义 |
| 断连或进程异常退出 | 保留已经落盘的快照；不推断执行已经停止 |
| 重试或重新分配 | 保留历史快照，严格关联原 task revision 和 attempt |
| 新任务承接前一任务 | 显式附加固定版本引用，保留来源关系 |

异常退出时不承诺一定能够生成新包。恢复逻辑应能展示最后一个已保存的检查点，并明确执行结果仍然不确定。

不能因为存在交接包而：

- 接受任务。
- 确认取消完成。
- 自动开始下一次执行。
- 自动重试不确定的写操作。
- 自动确认消息已处理。
- 绕过未回答的问题。

## 9. 保证提交、快照和通知的原子性

复用现有 Store 事务。

正常提交的一次成功事务至少应涵盖：

```text
Validate task/revision/attempt/run
→ Validate structured input and evidence references
→ Build bounded immutable snapshot
→ Persist resource and task association
→ Persist task transition
→ Record events
→ Queue existing task notifications
→ Persist idempotency receipt
```

外部唤醒仍在事务成功后通过现有机制执行。

要求：

- 使用现有事务上下文，避免嵌套开启事务。
- 同一个 `request_id`、同一请求内容重试，返回原始引用和回执。
- 不重复创建资源版本、任务事件或通知。
- 同一个 request ID 携带不同内容时拒绝。
- 相同任务旧版本或旧 attempt 的迟到提交必须拒绝。
- 事务失败不能留下孤立资源、半完成状态或已消费通知。
- 网络断开后的重试不能重复执行一次结果提交。

对支持交接的新路径，生成失败不能被静默隐藏。返回明确错误，并保证重试能够安全恢复。

同时必须通过容量设计保证：新增自动整理不会让原本合法的旧提交仅因重复复制过多内容而失败。

## 10. 区分任务版本、执行批次和资源版本

至少存在三种不同版本：

```text
task revision
attempt identity
context resource revision
```

三者不能互相替代。

建议每个任务维护稳定的交接资源身份，每次合法检查点或结果提交追加资源版本，并保存明确的来源绑定。

必须保证：

- 旧任务版本的交接不能显示为当前结果。
- 新 attempt 不能继承旧 attempt 的执行权限。
- 重新分配后仍能读取允许查看的旧交付，但不能把旧 run 当作当前生产者。
- 审查变化不能改写已发布资源内容。
- 没有当前执行对应快照时，应显示“暂无当前快照”，可以另列历史快照。

对已经提交的交接进行纠正，应走明确的现有补交、退回或重提交流程。不能直接修改旧资源，也不能创建隐藏的纠错旁路。

## 11. 交接快照和当前状态必须分别表达

快照里的事实是“当时记录了什么”。

当前状态需要另外查询：

- 当前任务是否已验收。
- 验收是否被撤回。
- 该结果是否已经被新结果替代。
- 对应执行是否仍不确定。
- 引用证据是否仍然可用。

不得把这些动态信息写回已经计算摘要的资源内容。

例如，一个提交时生成的快照可以永久保留当时的 `submitted` 来源事实；后来验收通过，由任务当前投影表达。验收撤回后，旧快照仍可读取，但不能继续显示成当前已通过交付。

## 12. 保留原有权限范围，防止自动发布扩大可见性

这是必须单独设计和测试的部分。

现有普通 Context Resource 在项目内共享，而 Worktree 任务读取具有更严格的参与者与 Coordinator 边界。自动交接不能把原本受限的任务内容直接变成项目公开资源。

目标规则：

- handoff 资源默认继承来源任务的读取范围。
- `context_get`、`context_list`、MCP resource browsing、bootstrap、面板、历史导出都执行一致检查。
- 不允许从列表标题、预览、计数或错误响应中绕过权限读取正文。
- 资源作者已经离线，不应导致合法历史读取失效。
- 新读取者仍必须使用其当前有效身份通过授权。

跨任务交给 B 时，需要明确的、限定范围的资源共享：

1. 发起者有权读取来源交接。
2. 发起者有权管理目标任务或向目标 Agent 发出对应指令。
3. 只授权指定固定版本交接资源给目标任务的合法参与者。
4. 不因此授予整个来源任务、来源工作区或全部历史的访问权限。
5. 不自动递归开放交接包引用的其他受限资源。

可复用现有 Store metadata 保存这类引用授权，不新增独立权限服务。

普通 Project 和 Worktree 的协调权限必须保持现有差异。不能借此让普通参与者取得 Coordinator 权限。

## 13. 明确两种接手方式

### 同一任务继续执行

- 保留稳定 task/thread 身份。
- 展示此前相关检查点与执行结果。
- 继续执行仍须经过现有停止确认、重试、重新分配和 `TaskStart` 检查。
- 执行结果不确定时，交接包不能替代恢复决策。

### 新任务消费前一任务交付

- 在创建或分配时显式选择固定版本交接引用。
- 保留来源任务与目标任务的关联。
- 只有业务上需要等待前置任务验收时，才使用现有依赖关系。
- 不把所有引用关系自动变成执行依赖。
- 不把“消费前一任务交付”错误写成 delegation 的 parent thread。

已有任务固定的输入引用不能跟随资源最新版本自动变化。更新上下文应通过明确的重新分配或其他已经定义的授权操作完成。

来源任务后来被退回时，应提示受影响的后续任务重新核验，不能自动回滚文件、取消正在执行的 Agent 或重复执行任务。

## 14. MCP 修改与 Agent 接手流程

复用现有工具体系，统一修改 schema、工具描述、bootstrap 和返回类型。

应覆盖：

```text
warp_task_submit
warp_task_progress
warp_task_fail
warp_task_finish_cancel
warp_task_get
warp_task_history
warp_execution_thread_get
warp_project_bootstrap
warp_context_get
warp_context_list
```

以及相关分配、发送和回复操作。

接手引导应明确要求 Agent：

```text
Refresh bootstrap and task
→ Read the exact pinned handoff resource
→ Check source task/attempt and current review status
→ Identify missing or stale information
→ Inspect only the necessary source and evidence
→ Enter execution through existing task operations
```

必须保留以下语义：

- 发现工具不代表开始执行。
- 读取交接不代表接受任务。
- 读取资源不确认原消息已处理。
- 收到通知不代表 Agent 已读取。
- Agent 的“已读取”自述不等于正确理解。
- 数据中的建议不能覆盖用户规则。

无任务关联的普通结果回复不自动创建新任务或新交接系统。

## 15. Companion、SSH、WSL 和协议兼容

新增明确的 capability，例如：

```text
handoff_briefs_v1
```

只有实现了该版本完整语义的服务才可声明，包括：

- 数据结构。
- 生成规则。
- 原子性。
- 权限。
- 读取与投影。
- 错误行为。

复用现有 Agent Bus `Operation` 和 Companion 通道。

特别注意：当前操作反序列化使用严格字段规则。**不能因为新字段是 optional，就假定旧 Companion 会忽略它。**

兼容要求：

| 组合 | 预期行为 |
| --- | --- |
| 新桌面、新 Companion | 使用完整功能 |
| 新桌面、旧 Companion | 显示不支持；不发送其无法解析的新字段或类型 |
| 旧 Agent、新 Broker | 原提交继续可用，缺失说明明确标记 |
| 历史记录 | 能正常读取，显示无交接包 |
| 不支持新资源类型的参与者 | 不收到无法消费的结构化内容，不静默改成大段内联正文 |

不要仅为新增字段就默认升级协议 major。先按现有 capability 与版本规则判断。

继续保留：

- 主机、账户、项目、连接和 generation fence。
- 迟到响应拒绝。
- 当前受确认的远程项目身份。
- 断连后的不确定性。
- SSH 与 WSL 的既有权限边界。

相同 Git remote URL 不能成为跨主机、跨账户或跨项目交接的授权依据。本轮不新增跨主机联邦。

## 16. 消息队列和通知传递

继续使用现有队列和唤醒机制。

通知只携带必要的任务定位与固定版本引用，不携带整个交接正文，也不把正文塞进 PTY。

必须区分：

```text
queued
notification submitted
resource read
task started
result submitted
review accepted
```

这些是不同事实，不应自动互相推进。

失败恢复要求：

- 唤醒失败时保留已有队列记录。
- 资源读取失败时保留待处理任务。
- 相同通知重放不生成第二个交接包。
- 结果消息不引发互相回复的确认循环。
- 取消、审批、输入 epoch 和 native run 的保护继续生效。

如果记录资源读取事件，它只能表示“资源被读取”，不能标为“已接手完成”。没有产品必要时，不新增专门的已读操作。

## 17. 通信面板、任务详情与历史

复用现有面板，不另建一套任务管理页面。

任务详情至少显示：

- 交接摘要。
- 来源任务版本、执行批次和生产者。
- 交接资源版本。
- 提交状态与当前验收状态。
- 关键决策。
- 未解决问题。
- 下一步建议。
- 证据入口与缺失信息提示。
- 历史版本入口。

列表和 bootstrap 只读取紧凑索引。用户展开详情后再按需读取正文。

UI 要求：

- 任务切换后，迟到响应不能覆盖当前任务。
- 加载、空状态、无权限、旧 Companion、读取失败应分别表达。
- 历史记录不能显示成当前已验收结果。
- 路径只是来源信息，不能绕过已有文件访问检查。
- 保留焦点、草稿、选择、滚动和取消行为。
- 英文、简体中文、繁体中文覆盖一致。

视觉事实来源使用现有原生面板、控件和已接受的布局规范。先完成固定数据状态，再做截图验收，覆盖长文本、缺失字段、失败状态和多语言。

## 18. 证据、commit 和工作区一致性

严格复用现有 Evidence 的语义。

当前系统允许记录测试 outcome，但不会把 Agent 报告的测试结果直接核验成独立执行事实。因此 UI 和 API 至少要区分：

- Agent 报告的测试结果。
- 系统对文件内容或 commit 等进行的核验。
- 任务审查结论。

不能把单个 `verified` 标志扩展成“全部测试已经由系统执行”。

测试信息应尽可能关联已有记录中的：

```text
command
outcome
exit_code
environment or platform description
evidence ID
source task/attempt
```

缺少运行环境、工具版本或 CI 证明时，保留缺失状态，不自动查询外部平台。

工作区信息要求：

- 使用已有确认绑定，不能从当前终端 cwd 反推原交付位置。
- 有 commit 也可能存在未提交修改。
- 多个 commit 证据不能随意压缩成一个“最终 commit”。
- 非 Git 项目应正常工作。
- 文档、研究和非编程任务可以没有 commit、测试或文件变化。
- 资源校验通过，不代表当前磁盘文件仍匹配快照。

需要检查文件或 Git 时，由获得授权的接手 Agent 使用现有工具按需核验；交接生成器本身不新增扫描行为。

## 19. 容量、持久化、清理与长期记忆

遵守现有容量约束：

- Context 内容当前为 32 KiB。
- 管理命令还需满足整体传输限制。
- 固定上下文引用当前最多 16 个。
- 列表和历史沿用分页与 Store 查询预算。

所有容量按实际编码字节计算，覆盖中文等 UTF-8 输入。

实现时应：

- 引用已有结果和证据，避免重复复制长文本。
- 对预览截断明确标记。
- 不静默截断关键决策和未解决问题。
- 对新增的 Agent 说明字段设置明确预算。
- 超限时返回可操作错误，或使用规格中已经固定的分块、分页方式。
- 不用递归展开所有历史交接来构造一个越来越大的包。

持久化与清理要求：

- 重启后可读取原固定版本。
- 归档不等于删除。
- 被有效任务、交接或记忆引用的资源不能被清理成悬空引用。
- 历史清理必须保留解释这些引用所需的最小来源记录。
- 清理预览体现引用保护。
- 取消共享不能通过修改旧快照实现。
- 不新增脱离现有策略的无限保留机制。

交接包属于任务快照。项目长期记忆继续走现有 proposal、review、promotion 和 retirement 流程，不能自动把每个交接包提升为项目记忆。

## 20. 安全与内容信任

必须保留：

- 项目、账户、任务、执行批次和 run 授权。
- workspace-relative 路径约束。
- 引用 digest 验证。
- 外部 URI 与任意文件读取限制。
- 消息和资源中的权限边界。

交接数据不能触发：

- shell 执行。
- 外部 URL 抓取。
- 任意文件读取。
- 切换工作区。
- 自动应用 patch。
- 自动接受任务。
- 更改用户权限。

交接内容是 Agent 输入。即使来自另一个 Agent，也可能包含错误或恶意指令，不能覆盖用户和项目规则。

不得收集或写入凭据、cookie、私钥、环境变量值、原始 transcript 或未筛选的命令输出。可以增加针对明确敏感内容的检查，但不能宣称过滤器能够保证发现所有 secret。

日志优先记录 ID、版本、状态和错误类别，不打印完整交接正文。

## 21. 实施顺序与阶段验收

| 阶段 | 工作 | 进入下一阶段的条件 |
| --- | --- | --- |
| A | 完成规格、字段来源、权限和兼容契约 | 关键语义没有悬空问题 |
| B | DTO、验证、序列化和系统保留类型 | 合法、非法、超限输入测试通过 |
| C | Store 原子生成、关联和幂等 | 故障注入与重放测试通过 |
| D | 生命周期、权限和跨任务引用 | 不确定执行与越权测试通过 |
| E | MCP、bootstrap 和接手引导 | 原有工具和新字段兼容通过 |
| F | Companion、SSH、WSL | 新旧版本组合及真实通道通过 |
| G | 面板任务详情与历史 | 固定数据和运行时截图验收通过 |
| H | 完整构建、产物和交付记录 | 所有完成标准有对应证据 |

实现路径由 Agent 负责选择。不要为常规实现选择反复请求确认，但遇到范围冲突、必要权限不足或真实破坏性风险时必须指出。

## 22. 必须覆盖的测试

### 数据与版本

- 新旧提交参数都可读取。
- 缺失、空值、不适用三种情况不混淆。
- 非 Git、无文件、无测试的任务可交接。
- task revision、attempt、resource revision 分别校验。
- 篡改内容或 digest 被拒绝。
- 非法路径、外部 URI、超大内容被拒绝。
- 中文字节预算正确。
- 普通发布不能伪造或覆盖系统 handoff。

### 事务与恢复

- 提交成功时资源、状态、历史、通知和回执一致。
- 在各写入阶段注入失败后没有部分成功。
- 相同请求重试不重复生成。
- 相同 request ID 携带不同内容被拒绝。
- 提交后的连接断开可以安全重试。
- Store 重启后旧引用仍然有效。

### 权限

- 外项目、外账户、无关 Worktree 读取被拒绝。
- list、get、bootstrap、历史和导出使用一致权限。
- 合法目标任务可读取明确共享的交接。
- 该共享不授予来源任务全部权限。
- 不自动递归扩大被引用资源权限。
- 来源 Agent 离线后，合法历史仍可读取。

### 生命周期

- 提交、接受、退回、失败、阻塞、取消分别正确。
- 异常退出不伪造完成。
- 执行不确定时不能直接产生新所有者。
- 旧 attempt 的迟到结果被拒绝。
- 旧快照不会冒充当前结果。
- 验收撤回不会改写旧资源。
- 读取、通知和接手不自动确认任务或消息。

### 证据

- Agent 报告的测试通过仍显示为报告结果。
- 文件或 commit 核验不能被解释成测试执行。
- 后续文件变化不改变旧交接内容。
- 证据缺失、不可用或来源不一致能够明确表达。

### 传输与 UI

- Local、真实 SSH、直接 WSL。
- 新旧 Companion 组合。
- macOS、Windows、Linux。
- 清除旧能力后的安全降级。
- 断连、重复通知、重启和迟到响应。
- 面板加载失败、长文本、多语言、历史版本和焦点保持。

Rust 构建与原生验收按项目规定在 GitHub 使用缓存执行，本地完成语法和静态检查。至少复用现有检查：

```bash
cargo test -p warp-agent-bus --locked
cargo test -p remote_protocol --locked

cargo check -p warpai --bin warpai --locked
cargo check -p warpai --bin warpai --features warp_platform --locked
```

再加入本功能的聚焦测试，以及现有协调、SSH、WSL 和原生 UI 验收路径。不要以纯 parser 测试替代真实 MCP 和 Companion 通道验证。

## 23. 完成交付标准

交付必须包含：

- 更新后的规格与 API 契约。
- 共享 DTO、Store、MCP、Companion 和面板的完整改动。
- 权限、事务、版本和故障恢复测试。
- 三平台构建结果。
- Local、SSH、WSL 验收记录。
- 与源码 commit 对应的测试产物。
- 配套 Companion、manifest、capability 和版本兼容说明。
- 原生 UI 截图及多语言检查结果。
- 更新后的 `MEMORY.md`，遵守其大小限制。
- 可审查的 PR，列明验证范围和未通过的交付门槛。

发布继续使用现有流程，不覆盖不可变版本，不擅自替换个人安装或修改个人配置。

**不能仅因交接包能够生成和显示，就宣布功能完成。** 必须证明接手方能够在正确权限下读取正确版本，理解其验证程度，并按既有任务规则继续工作。

主要风险集中在权限扩大、虚假验证状态和重试后的版本混淆，必须作为交付门槛验证。
