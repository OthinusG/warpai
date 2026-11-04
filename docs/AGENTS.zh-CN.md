# Agent 使用与兼容性

[English](AGENTS.md) | **中文使用指南** · [返回产品介绍](../README.zh-CN.md)

Warpai 可以运行你独立安装的 CLI Agent。参与消息与任务协作还需要该 CLI 版本支持原生 MCP 客户端，并加载 Warpai 的本地桥接配置。

本页依据仓库的 [原生 MCP 兼容性记录（英文）](../specs/agent-communication/COVERAGE.md) 整理。
“具备接入条件”表示已有原生 MCP 文档或已安装 CLI 的帮助信息作为依据，不表示每个服务商、版本的真实模型协作都已通过验收。

## 开启协作

1. 先安装并认证所需 CLI Agent；安装 [Warpai](../README.zh-CN.md#get-warpai)。
2. 打开 **Settings > Features > Agent communication**。该功能默认关闭。
3. 勾选可用的 Agent，Warpai 会在后台配置捆绑的原生 stdio MCP 桥接器。
4. 在项目目录中新开终端和 Agent 会话；不支持配置热加载的 CLI 需要重启。
5. 打开 **Agent collaboration** 查看参与者、消息与任务。

无需手动输入注册提示或复制动态凭据。同一规范化项目中的参与 Agent 可以互相发现；仓库子目录共享范围，不同仓库和独立 worktree 默认隔离。

## 接入条件与识别命令

下表是仓库记录的接入条件，不是所有已认证模型的通过名单。组织策略、已安装版本或原生 MCP 加载失败仍可能阻止参与。

| Agent | 识别命令 | 接入条件 |
| --- | --- | --- |
| Claude Code | `claude` | 原生 MCP；已有已安装 CLI 帮助信息。 |
| Gemini | `gemini` | 原生 MCP；已有官方文档。 |
| Codex | `codex` | 已安装版本需支持 `-c` 与 `--no-daemon`；普通交互启动自动添加会话级 MCP。 |
| Amp | `amp` | 仅本地 CLI；云端 orbs 不在范围内。 |
| Droid | `droid` | 官方连接器支持。 |
| OpenCode | `opencode` | 官方原生 MCP 配置。 |
| Copilot | `copilot` | 官方 CLI MCP 配置。 |
| Pi | `pi` | 需使用具备原生 MCP 的版本；旧版本外部扩展不作为回退方案。 |
| oh-my-pi | `omp` | 维护者提供原生 MCP 配置。 |
| Auggie | `auggie` | 官方 CLI 集成。 |
| Cursor | `agent`、`cursor-agent` | CLI 原生 MCP 能力。 |
| Goose | `goose` | 原生扩展配置。 |
| Hermes | `hermes`、`hermes-agent` | 原生 MCP 集成。 |
| Mistral Vibe | `vibe`、`vibe-acp` | 原生 stdio 配置；启用后在目标目录中新开终端。 |
| Antigravity | `agy` | 已安装 CLI 帮助确认原生 stdio。 |
| Grok Build | `grok` | 官方 MCP 配置。 |
| DeepSeek Harness | `dsh-tui`；使用 TUI profile 的 `dsh` | profile 必须挂载官方 MCP 客户端。 |
| Qoder | `qoder`、`qodercli`、`qoder-cli` | 原生 MCP；与 QoderCN 独立发现和授权。 |
| QoderCN | `qodercn`、`qoderclicn`、`qoder-cn` | 已安装 CLI 帮助确认原生 stdio；独立设置项。 |
| Trae | `trae`、`traecn`、`trae-cli`、`traecn-cli`、`traecli` | 需文档所述原生 MCP 版本；启动名称兼容性仍需逐版本确认。 |
| 自定义 CLI | 用户配置的识别规则 | 逐个确认原生 MCP 能力；识别到命令本身不足以证明可以协作。 |

各服务商的官方依据与适配器细节见 [完整技术记录（英文）](../specs/agent-communication/COVERAGE.md)。

## Codex：保留原来的命令

在 Warpai 本地终端开启协作并勾选 Codex 后，直接输入 `codex`、
`codex --yolo` 或 `codex resume --last`。Warpai 提交命令时添加原生会话级
MCP 参数与 `--no-daemon`，使不同终端的后台与项目绑定独立。
原有参数、工作目录、服务商选择和权限选项继续生效。

Warpai 每次启动都会重新解析并探测当前安装，因此仍可用原包管理器更新 Codex。
不替换用户命令，不修改 PATH、CODEX_HOME 或 Codex 配置，不复制临时可执行文件。
版本不具备所需能力时按原样启动，并显示 MCP 提示。
帮助、登录、批处理、显式远程连接、用户别名和 WSL 保持原生行为。
SSH 项目使用明确的 companion 启动入口。

## 配置和权限由谁管理

Warpai 仅管理自己拥有的 `warpai-local-com` 配置，不接管其他 MCP 服务或用户设置。
格式错误、所有权冲突、用户修改过的受管项或符号链接，会使配置操作停止并显示原因。

取消勾选 Agent 或关闭功能，会立即撤销桥接服务中的权限。
配置清理失败仍保持无权限，并可通过 **Refresh agents / retry cleanup** 重试。
已有 CLI 的配置移除可能需要重载或重启才能体现在其界面中。

桥接器必须从受管理的 Warpai 终端继承当前会话绑定。在终端外启动的进程不能借此自动参与。
动态凭据不写入持久配置，也不需要用户复制。Vibe 使用内存中的原生环境配置，因此切换 profile 或项目后应新开终端。

## 查看状态与排队任务

- 忙碌的 Agent 可以积累待办；只有满足原生就绪条件后才提交下一条 inbox 指令。
- 不会覆盖输入草稿，也不会代答权限请求。
- 排队、投递、确认收到、提交结果和验收通过是不同状态。
- 无法确认生命周期或等待人工批准时，不应仅凭“桥接器已连接”判断任务能够自动执行。

## 任务交接简报（handoff）

受跟踪任务保留交接简报，便于其他 Agent 从指定结果或检查点继续工作。提交任务时
保存基础简报；任务失败和确认取消时保留部分成果或停止点。显式进度检查点还可记录
决策、未解决问题和有序后续步骤。未提交的部分明确标为未提供，不从终端输出中推断。

在 **Agent collaboration** 中打开任务，浏览交接版本并按需加载。每条引用固定到
不可变版本；使用前应分别检查源任务当前状态、评审和执行不确定性。可将引用附到
后续任务，或向接收者显式共享指定版本。简报不授予源任务、文件系统或执行权限，
接收者仍须遵守正常的任务启动与恢复流程。
使用交接简报需要配套的 Companion 6.0.0。简报不是新的用户授权，也不代表提交结果已验收。
源任务被退回、重试或撤回后，旧引用不会自动跟随新版本。确认当前状态后再决定继续、
请求新简报或执行正常恢复。仍被活跃任务或消息引用的源记录不会被历史清理删除。

### MCP 中的交接流程

1. `warp_task_submit` 总会保存简报；`warp_task_fail` 和 `warp_task_finish_cancel`
   保留失败或停止时的简报。`warp_task_progress` 仅在显式提供 `handoff` 时保存检查点。
2. `handoff` 的决策、未解决问题和有序后续步骤使用结构化字段，总编码上限为 8 KiB。
   各部分区分已提供、未提供和不适用；决策引用的 evidence ID 必须属于源任务。
3. 在任务详情按需加载版本；`warp_task_history(handoff_only=true)` 可分页读取版本索引。
   固定引用包含资源 ID、版本和摘要，不会自动跟随最新版本。
4. 向后续任务的 `context_refs` 附加引用，或向普通消息接收者显式共享指定版本。
   接收者刷新 bootstrap/任务后，用 `warp_context_get` 读取该固定引用。
5. 可用 `handoff_part` 按需读取 status、task、result、binding 或指定 evidence ID。
   普通部分上限为 48 KiB，原生绑定上限为 256 KiB。缺失的来源信息保持明确缺失，
   Agent 报告的测试结果与证据验证是不同事实。

固定引用只授权读取指定版本，不授予源任务历史、递归上下文、文件系统、Coordinator
或执行权限。任务仍须正常调用 `warp_task_start`，遇到执行不确定性时走受保护的恢复流程。
源内容中的文字不能替代用户授权。读取时会提示源状态变化；不会因此自动重启下游工作。

历史导出包括交接资源及其源事件和证据；简报不会自动提升为项目记忆。创建由 Broker
管理，普通 context 发布不能伪造 handoff。本地、SSH 和直接 WSL 使用同一流程；远端
需要配套 Companion 6.0.0 和 `handoff_briefs_v1` 能力，不支持时拒绝请求，不退化为普通消息。

### 命令识别与任务历史

已知 Agent 可执行命令识别忽略 ASCII 大小写，例如 `CODEX` 和 `qoderCN`，参数保持原样。
别名和函数按原 Shell 规则查找：PowerShell 忽略 ASCII 大小写，POSIX Shell 精确匹配。
命令被识别后仍需受支持的原生 MCP、启用的 Agent 和有效会话绑定。

归档任务保留原来的完成或失败状态，并显示归档标记；重复 Archive 不改变版本。
Delete 需要确认当前任务版本，保护仍在执行、执行不确定、含未读消息或被引用的任务。
审计事件与操作回执保留。删除单条任务与清理整个历史是不同操作。

## 已验证到哪一步

仓库在 2026-10-01 记录了 Codex、Claude Code、QoderCN、Cursor、Antigravity 和 DeepSeek Harness 的真实 CLI 握手与工具发现测试。
这些探测没有服务商模型调用，不等于完整的跨 Agent 任务执行或自动唤醒验收；Qoder 与 QoderCN 的结果不能混用。

后续工程验收覆盖本地任务状态机、原生进程、桌面操作和受控 OpenSSH 场景。
原版 macOS Codex 0.160.0 已完成两个并发会话、合计四轮真实模型与 MCP 调用；
Windows 覆盖原生 Shell、参数传递和进程测试。其他服务商版本的覆盖范围按兼容性记录查看。
具体源码与 CI 证据见 [当前验收记录（英文）](../specs/agent-communication-v2/QUALITY.md)。

## SSH 项目

远程账号需要匹配版本的 companion 和在该账号中已安装、认证的 CLI。
桌面凭据不会复制到远端；远程参与权限固定在选定的账号和项目。
按照 [中文 SSH 入门步骤](../README.zh-CN.md#work-on-remote-projects) 配置和启动。
