# Warpai 全景功能特性清单与产品定位矩阵 (Feature Inventory & Value Proposition Matrix)

> 本文档全面梳理 Warpai 的所有功能特性、架构边界、精简项与独有创新，并按产品价值重要性进行层级排序。旨在为重构 `README.md`、`README.zh-CN.md` 及官网主页展示提供唯一事实来源（Single Source of Truth）。

---

## 1. 核心定位与价值主张 (Executive Summary)

**一句话定位**：
> **Warpai 是专为多 Agent 协同而生的本地优先（Local-First）现代终端工作台。**
> *The Terminal Built Around Autonomous Agents.*

- **传统终端（iTerm2 / Alacritty / Ghostty）**：仅负责执行命令与展示文本流，不具备智能感知与协同能力。
- **IDE 内嵌终端（VS Code / Cursor / Zed）**：以代码编辑器为绝对中心，Agent 通常被囚禁在侧边栏聊天框，无法胜任多分支并行重构、CLI 工具编排及长耗时后台任务。
- **原版 Warp**：强绑定官方闭源云端 AI、强制云账户登录、采集用户遥测数据，并设立了付费门槛。
- **Warpai**：
  1. **以 Agent 会话为工作单元（Unit of Work）**：把终端升级为多 Agent 协同的“作战指挥室”。
  2. **开放生态，不搞供应商锁定**：原生兼容并自动感知 20+ 款主流 CLI Agent（Codex, Claude Code, Gemini CLI, Cursor, Qoder, Trae, Antigravity 等）。
  3. **独创 Git Worktree 并行研发与 Coordinator 编排**：由主管 Agent 统筹、各分支 Agent 物理隔离并行施工、互审代码并最终自动集成，彻底解放开发吞吐量。
  4. **彻底的本地优先与绝对隐私**：零遥测、零云端账号、零付费墙，数据完全保留在本地，凭据存入系统安全密钥库。
  5. **一体化全栈工作流**：多 Agent 调度中控 + GPU 加速现代终端 + 项目文件树 + 轻量代码编辑 (Vim) + Markdown 实时预览 + Git Code Review + 本地模型额度大盘。

---

## 2. 差异化对比矩阵：精简性 vs 独有性

| 维度 | 原版 Warp (Upstream) | Warpai (本项目) | 核心用户收益 / 差异解释 |
| :--- | :--- | :--- | :--- |
| **云端依赖** | 强依赖 Warp Cloud 服务端 | **100% 本地优先 (Local-First)** | 无需连云，无单点故障，内网/离线可用 |
| **账户与登录** | 强制要求注册并登录 Warp 账号 | **零登录门槛 (No Cloud Account)** | 开箱即用，绝不收集或绑定用户身份 |
| **遥测与隐私** | 采集遥测、Sentry 上报与使用数据 | **零遥测 (Zero Telemetry)** | 彻底移除所有埋点与上报代码，代码与命令绝不上云 |
| **商业化付费** | 设有限额、订阅计费（Billing/Paywall） | **完全免费开源 (AGPL-3.0)** | 无隐形收费，无使用次数截断，社区自由维护 |
| **AI 架构** | 仅支持 Warp 官方闭源云端 AI 及云端中继 | **开放生态：自主调度 20+ 款独立 CLI Agent** | 自由选用各家最强模型/CLI，自带 key/账号即可 |
| **Agent 协作机制**| 无多 Agent 协作能力（仅单对话框） | **双重协同模式 (Project / Worktree)** | 支持多 Agent 跨分支并行施工、互相评审与集成 |
| **Agent 状态感知**| 无外部 CLI 感知能力 | **终端屏幕流状态机识别 (Screen Detection)** | 基于 VT100 流实时探测 Working / Idle / Approval |
| **任务派发** | 人工在每个窗口反复复制粘贴 | **智能守护自动入队 (Guarded PTY Delivery)** | Agent 空闲时自动把排队任务注入，流水线自运转 |
| **协同协议与工具**| 私有协议，不开放协同总线 | **38+ 原生 MCP 工具 (`warp-lite-communication`)** | 标准化任务管理、上下文共享、项目记忆与互审 |
| **工作区扩展** | 仅有终端 | **集成 Project Explorer + Editor + Preview** | 终端内直接看文件、编辑修改、预览文档，不依赖重型 IDE |
| **远程工作 (Remote)**| 基础 SSH，缺乏协同深度 | **自包含轻量 SSH Companion + WSL 2 深度集成** | 远程服务器免装 Node/Python/Git，全功能本地无差体验 |
| **Token / 账单大盘**| 仅看官方消耗，不支持外部模型 | **内置多平台 Quota 监控 (原生移植 Orca)** | 实时监控 Claude, Codex, Gemini, Cursor 等用量余额 |

---

## 3. 全景功能清单（按重要性排序）

### Tier 1: 核心杀手级特性 (Core Differentiators)
*这些功能是 Warpai 的灵魂，是同类终端与 IDE 完全不具备的护城河能力。*

#### 1. 20+ 款主流 CLI Agent 原生适配与统一纳管
- **无侵入式自动感知与挂载**：
  - 自动识别并管理当前环境安装的命令行 Agent，包括：**Codex、Claude Code、Gemini CLI、Cursor CLI、Qoder、QoderCN、Trae、Antigravity (agy)、Mistral Vibe、Goose、Hermes、DeepSeek Harness (dsh-tui)、Grok Build、Copilot CLI、Pi、oh-my-pi (omp)、Auggie、Amp、Droid、OpenCode** 等。
  - 支持用户自定义任意符合标准协议的 CLI 客户端匹配规则。
- **运行时环境自适应（Zero Global Pollution）**：
  - 不篡改用户的全局配置文件（不修改 `~/.bashrc`、`~/.zshrc` 或 Agent 的自身配置）。
  - 在会话启动时，动态向终端子进程注入 `WARP_AGENT_ENDPOINT`、`WARP_AGENT_CAPABILITY` 与 `WARP_TERMINAL_SESSION_UUID`，启动轻量桥接服务 `warpai-agent`。
  - 针对 Codex 等特殊客户端，支持动态传递参数（如 `--no-daemon` 和按会话独立的 MCP 挂载参数），避免污染后台 Daemon。

#### 2. 双重协作架构：Project 模式与 Worktree 模式
- **Project 模式（轻量化通用协作网络）**：
  - **适用场景**：方案设计、文献综述、数据分析、文档撰写、跨领域讨论。
  - **工作机制**：无需依赖 Git 仓库，所有加入当前项目的 Agent 会话组成平等的协作节点。共享项目上下文资源、共享任务树，Agent 之间可以直接提问答疑、委派子任务。
- **Worktree 模式（突破性的并行代码工程流）**：
  - **适用场景**：中大型功能开发、多模块重构、并行测试与修复。
  - **物理级代码隔离**：深度整合 Git 原生 Worktree，在不同的物理目录下并行拉起不同的分支，各自开独立的终端标签页。
  - **Coordinator（协调主管）机制**：
    - 用户可在面板中将任意在线的 Agent 指定为 **Coordinator**。
    - **职责分工**：Coordinator 负责拆解需求、将任务派发给不同 Worktree 里的 Worker Agents；Worker Agents 之间代码隔离，仅与 Coordinator 或同检出区 Agent 通信。
    - **同行评审（Peer Review）闭环**：Worker 完成任务后提交变更报告；Coordinator 接收结果，可指派其他 Agent 进行代码审查、提出修改意见（Needs Changes）或批准通过（Accepted）。
    - **无冲突集成（Task Integration）**：审查通过后，由 Coordinator 统筹合并并验证，执行最终的 Git Commit，杜绝多 Agent 在同一目录修改冲突的问题。
    - **容灾与热切换**：切换 Coordinator 不会中断正在执行的进程；若 Coordinator 掉线，会平滑降级，支持无缝指定新主管接管未完成任务。

#### 3. 终端屏幕流状态机识别 (Screen Detection) 与智能守护唤醒 (Guarded Delivery)
- **底层 VT100 / ANSI 流精准感知**：
  - 内置基于各类 Agent 交互特征的 TOML 规则库（Manifests），直接在底层 PTY 捕获屏幕渲染流与提示符状态。
  - 精确推导当前 Agent 的生命周期三态：
    1. `Working`：Agent 正在思考、生成代码或执行子工具。
    2. `Idle`：Agent 已完成当前回合，回到交互提示符等待指令。
    3. `WaitingApproval / WaitingInput`：Agent 遇到高危操作或询问，正在阻塞等待人工授权（如运行 rm 命令或修改关键文件）。
- **智能守护与无感自动交付（Guarded PTY Delivery）**：
  - 当有其他 Agent 派发任务或系统有待处理工单时，若目标 Agent 正处于 `Working`，任务自动进入 `inbox` 队列保持按序排队。
  - 一旦底层屏幕流探测到 Agent 变为 `Idle`，且检测到终端无用户未发送的草稿输入时，后台守护协程**自动将排队指令注入 PTY 执行**，完成自动唤醒！
  - 避免了人类必须守在窗口前、等 Agent 跑完再手动切换标签页按回车的低效痛点。
  - **安全护栏**：若检测到终端处于 `WaitingApproval`，绝不擅自越权按确认；若用户在终端输入了未回车的文本，绝不强行冲刷覆盖。

#### 4. `warp-lite-communication` 原生 MCP 协同总线 (38+ 核心工具)
*为接入的 CLI Agent 提供一整套操作系统级的团队协作能力，工具集包括：*
- **任务全生命周期管理**：
  - `warp_task_assign`（创建并指派任务）
  - `warp_task_claim`（从公共任务池认领任务）
  - `warp_task_create_pool`（创建悬赏/公共任务池）
  - `warp_task_progress`（同步任务百分比与阶段说明）
  - `warp_task_submit`（提交任务交付物与运行日志）
  - `warp_task_review`（评审任务：通过、打回、修改建议）
  - `warp_task_integrate`（Coordinator 专用：集成验收任务）
  - `warp_task_set_dependencies`（定义任务的前置拓扑依赖）
  - `warp_task_cancel` / `warp_task_retry` / `warp_task_reassign`（熔断、重试与转派）
- **点对点通信与问答系统**：
  - `warp_agent_send`（发送结构化消息，可选择是否期待回复）
  - `warp_agent_reply`（对特定请求产生确定性结果回调：succeeded, failed, blocked）
  - `warp_agent_ask` / `warp_agent_answer` / `warp_agent_questions`（跨 Agent 阻断式/异步咨询）
  - `warp_agent_inbox` / `warp_agent_ack` / `warp_agent_wait`（消息收件箱与处理回执）
  - `warp_agent_ready`（显式上报本回合执行完毕，进入待命状态）
- **上下文共享与项目记忆**：
  - `warp_context_publish` / `warp_context_get` / `warp_context_list`（跨 Agent 共享关键上下文、架构决策、环境状态）
  - `warp_project_bootstrap` / `warp_project_memory_update`（项目知识库持久化与初始化）
- **并发文件锁定（File Reservations）**：
  - `warp_file_reserve` / `warp_file_renew` / `warp_file_release` / `warp_file_reservations`（多 Agent 并行开发时锁定关键文件，防止写冲突）

#### 5. 纯本地优先、零遥测、零云绑定的隐私架构 (Privacy & Local-First)
- **彻底的代码级剔除（Deep Cleaned）**：
  - 删除了上游 51+ 个禁用的云端 AI 搜索与 Embedding 实现。
  - 删除了 38MB+ 无用内嵌静态图片资源，体积大幅瘦身。
  - 删除了所有 Sentry 错误追踪、网络上报脚本、云端遥测（Telemetry）通道。
  - 删除了强制绑定登录界面的阻断逻辑，不收集邮箱、名字或机器指纹。
- **本地自治数据存储**：
  - 所有配置、项目状态、会话历史、任务追踪全部保存在本地 `~/.config/warpai`（Windows 为 `%USERPROFILE%\.config\warpai`）下的 SQLite 数据库与结构化文件中。
  - 数据完全归用户所有，不受任何远程服务器停机、封禁或策略变更影响。
- **R.A.I.L.G.U.A.R.D. 安全隔离**：
  - 绝不通过 IPC、日志、错误提示暴露用户的 API Key、Token 或凭据。
  - 彻底禁用并 Stub 了潜在高危的 `computer_use`（防止外部 Agent 越权操纵操作系统键鼠截屏）。

#### 6. 自包含远程协作引擎：SSH Companion 与 WSL 2 原生集成
- **独立的 `warpai-companion` 远程助手**：
  - 单一可执行二进制文件，原生支持 Linux (x64/aarch64)、macOS、Windows。
  - **零远程环境污染**：自带内嵌独立的私有 Git 运行时，远程主机**无需**安装任何特定的 Node.js、Python、tmux、socat 或额外工具链。
  - **极度轻量**：由终端连接按需唤醒拉起，60 秒无活动自动退出释放资源，无后台守护进程常驻。
- **终端驱动的无缝目录感知**：
  - 无论本地还是远程 SSH，在终端中执行 `cd /path/to/project`，工作区自动感知并切换上下文，无需手动配置绑定。
- **Windows 原生 WSL 2 深度支持**：
  - 支持直接穿透管理 WSL 2 内部的 Linux 环境，通过安全的本地 stdio 通道与 WSL Companion 通信，无缝实现 WSL 内部 CLI Agent 的协同与文件管理。

---

### Tier 2: 一体化工程工作台特性 (Integrated Engineering Workbench)
*为开发者提供免跳出的一体化微型 IDE 体验，减少在终端、VS Code、Finder 之间的频繁切换。*

#### 7. 终端内嵌式项目文件管理器 (Project Explorer)
- 在终端右侧面板提供极速的文件树浏览。
- 支持直接在面板中新建文件、新建文件夹、重命名、批量删除。
- 自动随当前终端会话的当前工作目录（cwd）智能聚焦与高亮。
- 无论本地工程还是 SSH 远程服务器工程，通过安全的 SFTP / stdio 实现毫秒级响应的文件操作。

#### 8. 内置轻量代码编辑器与 Vim 键位支持
- 单击文件树即可就地展开内置轻量代码编辑器，毫秒级冷启动。
- 支持主流编程语言的代码语法高亮与行号显示。
- **原生 Vim 模式支持**：支持常用的 Normal / Insert / Visual 模式、移动快捷键与常用编辑指令。
- **远程无感保存**：支持直接编辑远程 SSH 或 WSL 主机上的源码，保存时自动触发远程安全写回，并具备冲突保护（Hash 校验，防止覆盖未知变更）。

#### 9. Markdown 原生富文本与图片实时渲染预览 (Live Preview)
- 专为 AI 时代的文档、报告、方案而优化。
- 支持在右侧面板直接预览 Markdown 文档，支持完整的 GitHub 风格 Markdown（GFM）、表格、代码高亮。
- 原生支持本地与远程图片的就地渲染加载，方便查阅 Agent 生成的图表、报告或技术方案。

#### 10. 内置 Git Code Review 差异审查面板
- 原生内置的 Git 审查面板，随时查看当前工程的 Git 变更状态。
- 清晰列出已暂存（Staged）、未暂存（Unstaged）、未追踪（Untracked）文件。
- 提供直观的双栏/单栏 Diff 差异比对、代码折叠与隐藏行展开。
- 支持在 SSH 远程仓库上直接进行只读式的 Code Review，无需在远程部署可视化工具。

#### 11. 多模型 Token / 额度 / 账单本地安全大盘 (Agent Usage Dashboard)
- **原生移植自开源项目 Orca (纯 Rust 重构)**：
  - 不引入额外的 Electron、Node.js 运行时，纯粹通过 Rust 原生实现各大模型供应商的用量与账单查询。
- **广泛的供应商覆盖**：
  - 实时查询并展示各平台的用量百分比、配额重置倒计时（5小时/每周窗口）及美元余额：
    - **Anthropic Claude**（通过当前 CLI 登录凭据或 Token 直查 `/api/oauth/usage`）
    - **OpenAI Codex / WHAM**（通过 CLI 凭据获取实时额度）
    - **Google Gemini**（通过 Code Assist 项目配额直查）
    - **Cursor**（通过安全 Cookie 查询月度额度与并发池用量）
    - **Google Antigravity (agy)**（针对 >=1.1.11 安全解析 `/usage` 免费元数据）
    - **xAI Grok**（通过 CLI 查询信用额度与月度账单）
    - 支持 Kimi、MiniMax、Z.ai、GLM 等主流编码计划。
- **系统级硬件安全凭据存储**：
  - 凭据仅存储在系统原生钥匙串（macOS Keychain / Windows DPAPI），绝不落盘明文，绝不进入任何 Agent 通信总线或日志。

---

### Tier 3: 顶级现代终端基建 (Modern Terminal Capabilities)
*继承并提纯了现代终端的顶级底层工程架构。*

#### 12. GPU 硬件加速渲染引擎 (wgpu / WarpUI)
- 基于 Rust 生态先进的 `wgpu` 图形抽象层与专属定制的 `warpui` UI 工具包。
- 纯 GPU 驱动的文本排版、着色器渲染与图层合成。
- 极低的输入延迟，面对数十万行的大规模终端输出依然保持 60FPS+ 丝滑滚动，告别传统终端的卡顿与撕裂。

#### 13. Block 块化终端架构 (The Block Paradigm)
- 彻底告别传统终端无序滚动的“电传打字机（TTY）”黑屏模式。
- **命令与输出原子聚合**：将每一条敲入的命令及其输出封装为一个独立的“Block（块）”。
- **强大的块级操作**：
  - 独立复制命令输入、独立复制输出结果（纯净无乱码）。
  - 针对超长输出一键折叠/展开。
  - 清晰展示每条命令的执行耗时与 Exit Code 退出状态码。
  - 历史命令快速定位与内容过滤。

#### 14. 智能命令自动补全与命令面板 (Autocompletion & Command Palette)
- **智能命令补全（Smart Completions）**：
  - 内置针对常用 CLI（Git, Docker, NPM, Cargo, K8s 等）的丰富命令签名与参数补全。
  - 支持历史输入预测、路径智能联想与参数说明悬浮窗。
- **全局命令面板（Command Palette - Cmd/Ctrl + P）**：
  - 键盘优先设计，快速搜索并执行所有终端动作、导航标签页、切换分屏、打开项目。

#### 15. 灵活的窗口与工作区布局系统
- **垂直标签页（Vertical Tabs）与水平标签页自由切换**：
  - 针对宽屏显示器特别优化的垂直标签页模式，容纳更多标签页而不压缩标题，清晰展示各工作分支。
- **无限制分屏切分（Split Panes）**：
  - 支持横向、纵向任意切分终端面板，快捷键顺滑跨窗格切换焦点。
- **跨窗口标签拖拽**：支持在不同的操作系统窗口之间自由拖拽和拆分标签页。

#### 16. 长耗时任务防休眠守护 (Keep-Awake Activity Guard)
- 内置平台原生的防睡眠机制（macOS `IOPMAssertion` / Windows `SetThreadExecutionState`）。
- 当有编译、训练、下载或后台 Agent 正在活跃工作时，智能阻止系统进入空闲睡眠，确保长达数小时的任务不被意外中断。

#### 17. 国际化多语言与个性化主题
- **原生多语言界面**：完整支持 **English、简体中文、繁體中文** 三种语言，UI 界面与设置选项深度本地化。
- **精选主题生态**：
  - 默认提供经过色彩学调优的 **Claude Warm Light（暖光浅色）**、Latte 以及沉浸深色主题。
  - 支持导入与自定义配色方案，图标与字体渲染均经过专业级次像素微调。

---

## 4. 横向竞品全维度对比表 (Competitive Analysis)

为了帮助用户直观理解 Warpai 在开发工具谱系中的精准生态位，以下为横向能力对比：

| 功能 / 场景 | Warpai | 传统现代终端<br>(iTerm2/Ghostty) | 原版 Warp | AI 代码编辑器<br>(Cursor/Windsurf) |
| :--- | :---: | :---: | :---: | :---: |
| **开源协议与本地性** | **AGPL-3.0 纯开源<br>100% 本地优先** | 开源/免费<br>本地 | 核心闭源<br>强依赖云端 | 专有商业软件<br>强依赖云端中继 |
| **强制云端登录** | **无 (零账户要求)** | 无 | **强制要求** | **强制要求** |
| **隐私与遥测** | **绝对零遥测** | 零或可选 | 收集遥测数据 | 收集代码与交互数据 |
| **第三方 CLI Agent 生态** | **原生支持 20+ 款<br>(Claude/Codex/Cursor等)** | 仅当普通终端运行<br>无法协同 | 不支持<br>(仅限自家闭源AI) | 仅运行自家内嵌 Agent |
| **多 Agent 跨分支并行研发**| **独家支持<br>(Git Worktree + Coordinator)**| 无法协同 | 不支持 | 不支持 (单线程对话) |
| **终端屏幕流状态机感知** | **原生支持 (VT100流推导)** | 不支持 | 不支持 | 不支持 |
| **任务队列与空闲自动投递**| **支持 (Guarded PTY)** | 不支持 | 不支持 | 不支持 |
| **团队协同协议 (MCP)** | **38+ 原生协同工具** | 无 | 无 | 部分支持标准单机 MCP |
| **GPU 硬件加速与 Block** | **具备 (wgpu 架构)** | 部分具备 GPU<br>无原生 Block | 具备 | 不适用 (为编辑器结构) |
| **内嵌文件管理与轻量编辑** | **内置 Explorer + Vim Editor** | 不具备 (需外部工具) | 基础文件支持 | 具备完整 IDE 能力 |
| **远程无侵入 Companion** | **单二进制免依赖<br>(自带私有 Git)** | 依赖远程环境配置 | 基础 SSH 连接 | 依赖远程 VS Code Server |
| **多平台模型额度监控** | **内置 (Rust 移植 Orca)** | 不具备 | 仅限官方点数 | 仅看自家消耗 |

---

## 5. 重构建议：README 与官网展示策略 (Refactoring Blueprint)

### 5.1 诊断：现有展示的致命软肋
1. **产品定位混淆**：把精力花在叙述与原版 Warp 的历史渊源上，让新用户误以为这只是一个“去掉登录的 Warp 破解版/精简版”，严重掩盖了核心价值！
2. **杀手级优势被埋没**：`Worktree 模式 + Coordinator 编排 + 20+ 款 CLI Agent 自动入队` 是整个业界目前唯一解决“多 Agent 并行大工程施工”的终极方案，在现有文档中却被当成普通功能轻描淡写。
3. **缺乏直观的“Aha Moment”视觉与动图**：文字过于冗长晦涩，缺乏一图胜千言的工作流架构图与终端实际运转动图。
4. **技术人员关心的痛点未前置**：开发者最关心的“安全隐私（零数据上报）”、“自带 key 自由使用任意大模型”、“无痛远程 SSH 开发”没有作为高亮特性在首屏展示。

### 5.2 推荐的全新展示结构（四步成案）

#### Step 1: 首屏冲击 (Hero Section)
- **主标题**：`Warpai: The Terminal Built Around Autonomous Agents`
- **副标题**：**为多 Agent 协同而生的本地优先现代终端。**
  *拒绝供应商锁定，在由你完全掌控的本地终端中，编排 20+ 款顶尖 CLI Agent 并行研发。*
- **三大核心标签**：
  - 🛡️ **100% Local-First & Zero Telemetry**（纯本地运行，绝对零遥测，不设付费墙）
  - 🤖 **Universal Agent Bus (20+ CLIs)**（Claude Code, Codex, Cursor, Gemini 等原生协同）
  - 🌳 **Parallel Worktrees with Coordinator**（多分支物理隔离并行研发，主管统一评审与集成）
- **核心操作区**：快速下载按钮（macOS Apple Silicon / Windows x64）+ 一行安装命令。

#### Step 2: 核心工作流震撼演示 (The Killer Workflow Showcase)
- 用一张精美的动图或架构图展现：
  > **需求进来 ➔ 用户在 Warpai 指定 Coordinator ➔ Coordinator 自动拉起 3 个 Worktree 并在后台指挥 Agent A 写后端、Agent B 写前端、Agent C 写测试 ➔ 屏幕状态检测发现 Agent 空闲自动注入下一阶段任务 ➔ Agent 互相发起 Peer Review ➔ Coordinator 校验通过一键合并提交。**
- 旁边配对比文案：*告别单窗口对话框的慢速轮询，告别 Git 冲突与上下文污染，这是属于 AI 时代的软件工程流水线。*

#### Step 3: 六大核心支柱 (Six Pillars of Value)
1. **开放的 Agent 协同总线**：讲透 20+ 款 CLI 支持与 38+ MCP 工具。
2. **突破性的 Git Worktree 并行工程**：讲透物理隔离与 Coordinator 评审机制。
3. **精准的状态机与自动守护唤醒**：讲透 VT100 流感知与空闲自动交付。
4. **全功能一体化工程工作台**：GPU 加速 Block 终端 + 文件树 + Vim 编辑器 + Markdown 预览 + Code Review。
5. **轻量无侵入的远程与 WSL 体验**：单二进制免依赖 Companion，随 `cd` 自动同步。
6. **坚定的隐私承诺与本地额度大盘**：零上报、系统钥匙串保护，多模型实时余额查询。

#### Step 4: 快速开始与无缝迁移 (Getting Started)
- 简单的安装指南。
- 如何在 `Settings -> Features -> Agent communication` 中一键勾选已安装的 CLI。
- 完善的文档索引链接。

---

*文档生成时间: 2026-10-15*
*维护者: Warpai 架构团队*
