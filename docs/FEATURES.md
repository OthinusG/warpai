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

## 2. 行业主流 Agent 管理体系全景速览与七大体系分条横向评测

为清晰展现 Warpai 在现代开发者工具链中的定位，下表将 Warpai 与业内七类主流系统架构的定位与差异化破局点进行全景速览：

| 系统体系分类 | 代表性工具 | Warpai 核心差异化破局点 |
| :--- | :--- | :--- |
| **1. 传统高性能终端** | Ghostty、Kitty、Windows Terminal、Alacritty、iTerm2、WezTerm | 在完整保留 GPU 极致渲染速度的基础上，独家首创内存级 VT100 状态追踪、守护式 PTY 交付与多 Agent 协同面板。 |
| **2. 带 Agent 文本编辑器** | Zed、Cursor、Windsurf、Trae | 打破单工作区文件写踩踏，以物理 Git Worktree 物理隔离多 Agent 并发开发，开箱拥抱 20+ 款独立 CLI Agent，零厂商云端绑定。 |
| **3. 传统综合 IDE** | VS Code、JetBrains (IntelliJ IDEA)、Eclipse | <100ms 极速启动、极低内存开销；作为敏捷的多 Agent 协同中枢，与传统 IDE 互补联动，告别 IDE 终端卡顿发热。 |
| **4. 自主 ADE 与沙盒** | Orca、Conductor、OpenDevin (All-Hands)、Devin、SWE-agent | 100% 纯本地优先，彻底摆脱多 GB 的 Docker 镜像开销与昂贵云端月费，源码与密钥不出本地硬件，零隐私泄露风险。 |
| **5. 多 Agent 框架与复用器** | Herdr、AutoGen、CrewAI、ChatDev、tmux / zellij 脚本 | 将现代化 GUI 终端工作台（标签、分屏、Diff、编辑器）与工业级 SQLite 原生 MCP 协同总线（38+ 工具、持久化队列、评审）完美融合。 |
| **6. 桌面端 AI 客户端** | ChatGPT Desktop、Claude Desktop、TypingMind | 彻底消除跨窗口复制粘贴内耗，直接在真实终端会话中由 Agent 自主读写代码、运行构建、跑通测试并在关键节点授权。 |
| **7. 上游原版 Warp 终端** | 原版 Warp | 彻底剥离全部 51+ 个云端 AI 文件、Sentry 遥测与强制邮箱登录墙（AGPL-3.0 开源），新增 20+ 款 CLI 智能体感知与双协作模式。 |

---

### 七大体系分条横向深度评测

#### 1. Warpai vs. 传统终端模拟器
**代表系统**：Ghostty、Kitty、Windows Terminal、Alacritty、iTerm2、WezTerm。
- **核心优势**：极致的 GPU 硬件加速文本渲染、极高吞吐率、极低延迟与极小基础内存开销。
- **面对自主 Agent 协作时的局限**：缺乏 Agent 生命周期感知，Agent 阻塞在确认时容易遗漏；输入无门禁防护，流式输出时打字容易吞键；无协同协议，窗格间数据彼此孤立。
- **差异对比表**：

| 评估维度 | Warpai 解决方案 | 传统终端基线表现 |
| :--- | :--- | :--- |
| **Agent 状态感知** | **内存级 VT100 屏幕观察器**：解析 ANSI 流，实时识别 Working/WaitingApproval/Idle，阻塞时毫秒级推送桌面通知。 | **被动字符光栅化**：对底层运行状态无感知，Agent 阻塞时只能被动等待人工逐屏排查。 |
| **输入安全与可靠性** | **独家守护式 PTY 交付**：仅在 CLI 处于稳定可交互提示符时投递指令，有未提交草稿时自动暂停注入。 | **无缓冲直接写入**：在 Agent 流式输出或全屏刷新时输入，极易造成丢键、指令乱码与非预期执行。 |
| **协作调度总线** | **内置原生 MCP 服务（`warpai-local-com`）**：38+ 项工具，支持持久化任务队列、FIFO 排队激活、文件锁与对等评审。 | **无任何协同协议**：各窗格为彼此隔离的纯字符环境，窗格间无法进行结构化消息互通。 |
| **工作区并发隔离** | **Worktree 编码与 Project 全向双模式**：物理隔离 Git 分支检出，杜绝代码冲突；非编程场景全员扁平协作。 | **单一共享目录**：多窗格默认在同一物理文件夹下执行，并发写入直接导致 Git 锁冲突与文件踩踏。 |
| **渲染与工作台体验** | GPU 渲染核心 + **原子级命令块**、垂直标签、原生分屏、内置轻量文件树、代码编辑器与侧边栏实时 Git Diff 视图。 | 高性能纯文本网格渲染，但缺乏命令块交互、内嵌文件树、Diff 对比与可视化工作台支持。 |

- **Warpai 核心代差**：Warpai 完整继承了顶尖 GPU 终端的亚毫秒级渲染速度与超低资源开销，同时补齐了多智能体运行所必需的生命周期监控、输入安全门禁与协同底座。

#### 2. Warpai vs. 带 Agent 功能的现代文本编辑器
**代表系统**：Zed、Cursor、Windsurf、Trae。
- **核心优势**：出色的行内代码补全（Inline Completion）、代码语义检索与文件缓冲区直接覆写。
- **面对多 Agent 并发开发时的局限**：以代码文件为绝对中心，Agent 沦为侧边栏挂件；单一工作区检出导致多 Agent 并发写代码时冲突；强绑定云端网关与专有模型。
- **差异对比表**：

| 评估维度 | Warpai 解决方案 | Agent 文本编辑器基线表现 |
| :--- | :--- | :--- |
| **核心操作单元** | **以 Agent 会话为第一工作单元**：专为运行、监控与编排完整 Agent 进程打造的原生终端工作台。 | **以文本缓冲区为第一单元**：以文件编辑为主视角，智能体仅为从属的侧边栏或对话挂件。 |
| **并发隔离与分支防冲突** | **物理 Git Worktree 隔离机制**：每个编程 Agent 独占独立的物理 Git 工作树，多分支并行开发零锁争用。 | **单一检出目录**：多 Agent 同时写文件会引发 Git index.lock 报错、未提交更改覆盖与缓存污染。 |
| **CLI 生态开放度** | **全开放生态（感知 20+ 款独立 CLI）**：开箱接入 Claude Code、Codex、Antigravity、Gemini CLI、Cursor Agent、Trae、Goose 等。 | **厂商生态绑定**：绑定厂商专有编辑器扩展插件、专用云端模型中转接口与按人头月费订阅。 |
| **隐私保护与数据自主** | **100% 纯本地优先 · 0 KB 遥测打点**：零登录墙、零远端代码审查、零数据分析。AGPL-3.0 协议全开源。 | **云端代理中转**：代码上下文、Prompt 历史记录与使用行为数据高频上传至厂商云端服务器。 |
| **泛行业非 Coding 场景** | **Project 全向协同模式**：开箱赋能学术科研论文、视觉设计、量化金融、专利合规、新媒体矩阵与商业尽调。 | **受限于代码文件**：交互设计完全围绕源代码展开，在非代码泛业务团队中无法自然施展。 |

- **Warpai 核心代差**：Agent 编辑器适合人机共写单文件，而 Warpai 提供了多分支物理隔离与开放 CLI 底座，是真正支撑多智能体团队兵团化并发作战的指挥中心。

#### 3. Warpai vs. 传统综合 IDE
**代表系统**：VS Code、JetBrains (IntelliJ IDEA, WebStorm, PyCharm)、Eclipse。
- **核心优势**：深度的编程语言 AST 语法树解析、工业级断点调试器与成熟庞大的插件生态系统。
- **面对多 Agent 体系时的局限**：启动慢、常态占用数 GB 内存，多开 Agent 时严重发热降频；内置终端仅为二等公民小面板；全功能塞入单一窗口带来操作卡顿。
- **差异对比表**：

| 评估维度 | Warpai 解决方案 | 传统综合 IDE 基线表现 |
| :--- | :--- | :--- |
| **启动速度与内存开销** | **秒级冷启动 (<100ms)**：原生 Rust 编写，极小基础内存占用，GPU 加速光栅化，零冗余后台语法预编译索引开销。 | **资源负担沉重 (1.5–3+ GB 内存)**：启动漫长，后台不间断的重型索引严重消耗电量并导致多 Agent 卡顿。 |
| **终端架构层级** | **一等公民 GPU 终端核心**：原生原子命令块、内存 VT100 状态解析器、守护式 PTY 交付与垂直标签管理。 | **边缘附属小面板**：基于 Webview/pty 的基础模拟器，无进程状态感知、无任务排队与输入防护。 |
| **多 Agent 调度中枢** | **内置协调者（Coordinator）与对等评审**：自动化分发任务、代码评审检查点（`warp_task_review`）与主干受控集成。 | **无原生协同机制**：插件市场各 AI 插件各自为政，插件间缺乏共享通信总线与互锁机制。 |
| **开发者工具链协同** | **与 IDE 敏捷互补**：日常用 Warpai 调度多 Agent 在独立分支上并行开发与测试；需要复杂单步断点调试时再用 IDE 介入。 | **单体大一统陷阱**：试图在单个笨重窗口中包揽一切，带来严重的卡顿与上下文切换成本。 |

- **Warpai 核心代差**：Warpai 并不试图替代深度的单步调试器；它用极速轻量的多 Agent 指挥中心替代了 IDE 笨拙脆弱的内置终端，两者优势互补。

#### 4. Warpai vs. 自主 ADE 与云端智能体沙盒
**代表系统**：Orca、Conductor、OpenDevin (All-Hands)、Devin、SWE-agent。
- **核心优势**：在封闭的沙盒中自主循环执行任务，隔离系统环境风险。
- **日常本地开发时的局限**：需要拉取数十 GB 的 Docker 镜像或高昂云端订阅；代码被上传到第三方服务器；容器无法直接复用宿主机的本地环境（SSH 密钥、内网代理、GPU 硬件）。
- **差异对比表**：

| 评估维度 | Warpai 解决方案 | 自主 ADE / 沙盒基线表现 |
| :--- | :--- | :--- |
| **基础设施依赖** | **100% 纯本地优先 · 零 Docker 依赖**：直接基于本地操作系统原生运行，远程服务器通过轻量单二进制伴侣透明穿透。 | **重型容器 / 虚拟机依赖**：依赖后台 Docker 服务守护进程、数 GB 容器基础镜像或云端沙盒虚拟机。 |
| **数据主权与代码安全** | **100% 隐私安全 · 0 KB 遥测**：所有代码、API 密钥、命令输出与 Git 历史完全停留在本地物理磁盘，绝不上云。 | **云端托管 / 源码外传**：商业 ADE 强制将企业核心资产代码和执行过程上传至第三方云端执行机。 |
| **宿主环境无缝接入** | **原生系统环境直接可用**：智能体直接调用本地 SSH 配置、私有密钥、内网代理、本地 GPU 显卡与专用硬件调试器。 | **沙盒容器隔离壁垒**：难以直接访问本地硬件驱动、私有网络映射与宿主机现存的复杂工具链配置。 |
| **使用成本与订阅门槛** | **完全免费开源 (AGPL-3.0)**：自带自有 API 或本地大模型即可运行，零平台抽成与人头许可费。 | **高昂商业订阅**：高昂的月度席位订阅费（$50–$500+/月），且按次计费或按算力点数昂贵扣费。 |

- **Warpai 核心代差**：Warpai 在保证多智能体并发开发与分支物理隔离的前提下，彻底摒弃了数 GB 的 Docker 容器包袱与隐私外泄隐患。

#### 5. Warpai vs. 多智能体编排框架与平铺工具
**代表系统**：Herdr、AutoGen、CrewAI、ChatDev、tmux / zellij 脚本。
- **核心优势**：在代码层面抽象多智能体对话逻辑（AutoGen/CrewAI），或通过终端复用器进行平铺切分（Herdr、tmux）。
- **面对交互式软件工程时的局限**：纯 Python 框架运行在无头黑盒中缺乏可视化；tmux 仅切分窗格而无结构化 SQLite 协作总线；人工肉眼轮询多进程终端日志易遗漏。
- **差异对比表**：

| 评估维度 | Warpai 解决方案 | 多 Agent 框架与平铺工具基线 |
| :--- | :--- | :--- |
| **可视化集成工作台** | **原生 GUI 现代化工作台**：现代 GPU 终端、垂直标签、原生分屏、交互式文件树、内置轻量编辑器与侧边栏实时 Git Diff。 | **无头 CLI / 原始窗格平铺**：无头 Python 纯命令行输出，或单纯切分文本窗格，缺乏可视化代码与差异审查。 |
| **结构化协同总线协议** | **内置原生 MCP 服务（`warpai-local-com`）**：38+ 项工具支持 SQLite 任务排队、依赖解析、点对点问答与文件锁。 | **脆弱按键盲注或纯内存态**：终端复用器依赖易错的 `send-keys`；Python 框架状态随进程退出即丢。 |
| **人机协同监管效率** | **动态监督任务池与即时告警**：实时 Working/WaitingApproval/Idle 状态徽标与系统级桌面弹窗通知。 | **人工肉眼轮询日志**：需不断使用 `tail -f` 查看多进程终端日志，或自行搭建笨重的 Web 仪表盘。 |
| **任务排队与单任务独占** | **持久化 FIFO 任务排队**：任务进池安全排队（`warp_task_assign`），每 Agent 单一活跃任务独占（`warp_task_start`），防索引争用。 | **需自行编码调度**：无内置进程安全独占锁，需开发者自行编写复杂的并发控制脚本。 |

- **Warpai 核心代差**：Warpai 成功兼具了现代桌面终端的高颜值与交互直观性，以及工业级 MCP 协同总线的持久化调度能力。

#### 6. Warpai vs. 桌面端 AI 客户端应用
**代表系统**：ChatGPT Desktop、Claude Desktop、TypingMind。
- **核心优势**：界面精美极简，适合发散式对话与探索性问答。
- **面对工程落地时的局限**：脱离终端物理环境，无法执行代码、构建或测试；高强度复制粘贴 Prompt 与日志；单线程单轮对话，无法支持多分支并发协作。
- **差异对比表**：

| 评估维度 | Warpai 解决方案 | 桌面端 AI 客户端基线表现 |
| :--- | :--- | :--- |
| **真实运行时执行能力** | **原生终端物理环境**：真实 CLI Agent 在真实 PTY 会话中自主读写文件、运行编译、执行测试并操作 Git 分支。 | **隔离的聊天沙盒**：无法直接与本地编译器、构建工具、调试器或测试套件发生交互。 |
| **工作流自动化程度** | **零复制粘贴流转**：智能体在用户把控下自主闭环推进，证据链与结果一目了然。 | **高强度人工搬运**：开发者充当“人工剪贴板搬运工”，在聊天框和终端之间不断切换复制。 |
| **多 Agent 并发并行** | **同时编排 10+ 个智能体会话**：多智能体在独立 Worktree 或 Project 频道中协同工作并进行对等评审。 | **单线线性交流**：单一会话窗口，无法进行多分支、跨角色的多任务并行推进。 |
| **模型与 Agent 自由度** | **自由接入 20+ 款独立 CLI Agent**：Claude Code、Codex、Antigravity、Gemini CLI、Cursor Agent、Goose 等全支持。 | **单一厂商绑死**：严格绑定该桌面客户端所属的特定云端模型服务。 |

- **Warpai 核心代差**：Warpai 将 AI 从只能提建议的“外部咨询顾问”，变成了直接在终端中写代码、跑测试、打胜仗的“实战攻坚队员”。

#### 7. Warpai vs. 上游原版 Warp 终端
**代表系统**：原版 Warp。
- **核心优势**：开创了原子级命令块（Command Blocks）与现代终端交互界面。
- **面对多 Agent 生态时的局限**：强制邮箱注册登录；内置 Sentry 错误追踪与遥测采集；仅支持单一专有云端小助手，无法管理外部独立 CLI Agent；单人单聊缺乏团队协同。
- **差异对比表**：

| 评估维度 | Warpai 解决方案 | 原版 Warp 基线表现 |
| :--- | :--- | :--- |
| **隐私合规与账号门槛** | **100% 纯净剔除 · 0 KB 遥测 · 零登录墙**：彻底移除全部 51+ 个云端 AI 文件与打点上报代码。AGPL-3.0 协议全开源。 | **强制云端登录墙**：必须注册邮箱账号登录，常驻后台遥测与命令用量数据上报。 |
| **Agent 生态支持** | **开箱感知 20+ 款独立 CLI Agent**：通过专用 VT100 描述清单精准感知 Claude Code、Codex、Antigravity、Gemini 等状态。 | **仅限专有云端小助手**：完全无法感知或协同外部的各类独立 CLI 智能体。 |
| **团队与并发协作架构** | **Worktree 编码与 Project 全向双模式**：物理隔离 Git 分支检出，支持协调者调度、对等评审与非代码泛行业全向协作。 | **单人单机终端**：缺乏多 Agent 分支隔离、无任务池分发、无角色流转与评审机制。 |
| **协作总线与用量底栏** | **内置 38+ 项原生 MCP 工具 + 多模型配额底栏**：任务排队、文件锁、评审检查点，底栏实时监控多平台 Token 额度。 | **封闭私有云后端**：无本地 MCP 协作服务，配额仅服务于自家订阅体系。 |

- **Warpai 核心代差**：Warpai 将终端从云端账号与遥测监控的束缚中彻底解放出来，升级为真正掌控在开发者手中的工业级多智能体指挥中枢。

---

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

#### 4. `warpai-local-com` 原生 MCP 协同总线 (38+ 核心工具)
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
