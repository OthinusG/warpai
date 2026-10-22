# Windows WSL 安装与配置指南

[English](WSL-INSTALLATION.md) | [简体中文](WSL-INSTALLATION.zh-CN.md)

本文介绍 Warpai 1.7.8.2 在 Windows 上使用 WSL 2 的完整流程：启用所需的 Windows 功能、安装 Linux 发行版、在 Warpai 中打开 WSL 会话、安装专用的 WSL Companion、在发行版内部配置各 Agent 的 MCP，并获得与本地项目一致的多 Agent 协同体验。

相关文档：
- [OpenSSH Server 安装与配置指南](SSH-INSTALLATION.zh-CN.md)
- [远端组件与协议参考](REMOTE-INSTALLATION.zh-CN.md)
- [Agent 兼容性与原生 MCP 支持](AGENTS.zh-CN.md)

---

## 1. 准备工作

| 项目 | 要求 |
| :--- | :--- |
| Windows | Windows 10 版本 2004（内部版本 19041）或更高版本，或 Windows 11。 |
| 虚拟化 | 已在 BIOS/UEFI 中启用硬件虚拟化。 |
| Warpai | Windows 版 Warpai 1.7.8.2。WSL 集成是 Windows 专属功能。 |
| 发行版 | 一个 WSL 2 发行版，推荐 Ubuntu 22.04 LTS 或 24.04 LTS。内置 WSL Companion 支持 x86_64 客户机。 |
| 账号 | 发行版内的 Linux 用户。Companion 安装到该用户的主目录下。 |

WSL 属于 Windows 本地集成，不需要 SSH 服务、端口转发，也不需要 SSH Companion 安装包。

---

## 2. 启用 WSL 2 并安装发行版

### 步骤一：启用 Windows 功能

以**管理员身份**打开 PowerShell。在当前 Windows 版本上，一条命令即可启用所需组件：

```powershell
wsl --install
```

如需显式启用对应功能，也可以执行：

```powershell
dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart
dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart
```

系统提示时重启 Windows。如果 BIOS/UEFI 中禁用了虚拟化，请先启用 Intel VT-x / AMD-V（或平台对应选项）。

### 步骤二：确认 WSL 2

```powershell
wsl --update
wsl --set-default-version 2
wsl --status
```

`wsl --status` 应显示默认版本为 WSL 2。

### 步骤三：安装发行版

```powershell
wsl --list --online
wsl --install -d Ubuntu
```

首次启动会要求设置 UNIX 用户名和密码。该 Linux 账号将拥有 Companion 安装，也是 Warpai WSL 设置中显示的账号。

### 步骤四：验证

```powershell
wsl --list --verbose
```

发行版应显示 `VERSION 2`，状态为 `Running`（退出后为 `Stopped`）。

---

## 3. 在 Warpai 中打开 WSL 会话

在 Windows 的 Warpai 终端中执行：

```powershell
wsl
```

也可以显式指定发行版和用户：

```powershell
wsl.exe -d Ubuntu -u <linux-user>
```

Warpai 会识别 WSL Shell，加载会话级 Shell 集成，并启用命令块、历史记录与目录跟踪。进入项目目录后，左侧协同面板会跟随并绑定该 WSL 项目。

### 工作区建议

将代码仓库放在 Linux 原生文件系统中，例如：

```bash
cd /home/<linux-user>/projects/<repo>
```

避免在 `/mnt/c/...` 下频繁执行 Git 操作或重度编译。9p 挂载便于共享文件，但 I/O 吞吐明显更低，且容易引入文件锁竞争。

如果 WSL 会话或 Shell 集成卡住，可从 PowerShell 重置发行版后重新打开：

```powershell
wsl --shutdown
```

---

## 4. 安装 WSL Companion

Windows 版 Warpai 内置了专用的 WSL 客户机载荷。WSL Companion 按 Linux 用户、按发行版安装，它不是 SSH Companion 安装包。

### 推荐方式：通过 Warpai 设置安装

1. 打开 **Settings > Features > Agent communication**。
2. 开启 **WSL communication**。
3. 先在 Warpai 中打开一个 WSL 终端，然后在下方选择 **WSL account**（发行版与 Linux 用户）。
4. 勾选 **Install WSL Companion**。
5. Warpai 会通过 `wsl.exe` 标准输入通道，将内置且经过校验的 Linux x86_64 载荷传输到所选发行版中，无需从外部下载。
6. 如果操作耗时较长，可通过 **View installation progress** 查看进度。

取消勾选 **Install WSL Companion** 会移除该账号下的安装；维护按钮中的 **Uninstall WSL Companion** 执行相同的移除操作。

> [!NOTE]
> GitHub 上发布的 Linux `WarpaiCompanion-5.5.0-linux-x64.run` 安装包用于 **SSH 远端主机**，安装的是 `~/.config/warpai/bin/warpai-companion`，不会安装 `~/.config/warpai/wsl/bin/warpai-wsl-companion` 处的专用 WSL 服务。本地 WSL 请使用设置中的安装器。

### 安装路径

- 基础可执行文件：`~/.config/warpai/bin/warpai-companion`
- 专用 WSL 服务：`~/.config/warpai/wsl/bin/warpai-wsl-companion`

### 验证运行环境

在 WSL 终端中执行：

```bash
~/.config/warpai/wsl/bin/warpai-wsl-companion --check-runtime
```

Companion 按需运行：仅在连接或 Agent 协同会话需要时启动，空闲 60 秒后退出。它不会注册持久化系统服务，也不会修改 Shell 启动文件。

---

## 5. 在 WSL 中配置各 Agent 的 MCP

MCP 配置属于 Agent 实际运行的环境。对于 WSL 中的 Agent，就是发行版内部该 Linux 用户自己的配置。

在 **WSL communication** 已开启且已选择 WSL 账号的情况下：

1. Warpai 会探测该发行版中已安装且受支持的 CLI Agent。
2. 每个被探测到的 Agent 都有一个复选框。启用后，Warpai 会将 `warpai-wsl-com` 协同桥写入该 Agent 在 WSL 中的自有配置。
3. 安装或更新 CLI 后，使用 **Rescan agents** 重新扫描。
4. 使用 **Remove Warpai MCP from all agents** 仅移除 Warpai 自己的配置项。
5. 使用 **Uninstall WSL Companion** 移除客户机组件。

这些变更只作用于所选的 WSL 账号。请在一个**新的 WSL 终端会话**中启动 Agent，使其重新加载配置。

Warpai 会保留厂商认证信息和其他无关的 MCP Server。复选框配置的是协同桥，而不是你的第三方 MCP 目录。若 Agent 在 WSL 中运行，请用在普通 Linux 机器上相同的方式和配置文件，在 WSL 内部安装第三方 MCP Server。

升级至 **1.7.8.2** 后，重新扫描会将已保存记录中的安装、查询和清理命令迁移至 `warpai-wsl-com`。处于未启用状态的清理记录可重新勾选安装；已有用户 MCP 配置项和启用选择均保留。随后启动新的 Agent 会话以重新加载配置。

---

<a id="two-tier-mcp-architecture-and-environment-isolation"></a>
<a id="4-深入理解两层-mcp-架构与环境隔离"></a>
## 6. 两层 MCP 架构与环境隔离

> [!IMPORTANT]
> MCP 的配置、安装记录与运行进程，绑定在实际运行 Agent 的机器、用户和环境上。WSL 只是终端传输层，**不会**把 Windows 宿主机的 MCP Server 带入 Linux 客户机。

### 两层体系

| 层次 | 配置来源 | 执行位置 | 安装与持久化 |
| :--- | :--- | :--- | :--- |
| **Warpai 原生 MCP（桌面层）** | 运行 Warpai 的本机。读取 `~/.warp/.mcp.json`、客户端各 Agent 的全局配置，以及本地项目级 `.mcp.json`。 | 由桌面客户端派生的本地 Warpai 子进程。 | 保存在本地 SQLite 数据库中。通过 Warpai UI/Gallery 安装的 Server 仅保存在本地，不写回 `.mcp.json`，也不跨设备同步。 |
| **CLI Agent 原生 MCP（环境层）** | Agent 所在环境内部的配置。Claude Code、Codex、Qoder 等 CLI 读取各自环境下的用户级与项目级文件。 | Agent 实际运行的环境内部。Agent 在那里读取配置并派发 MCP Server 子进程。 | 各环境完全独立。Windows 宿主、每个 WSL 发行版和每个 SSH 账号互不影响。 |

常见的原生配置位置：

| Agent | 用户级 | 项目级 |
| :--- | :--- | :--- |
| Warpai | `~/.warp/.mcp.json` | `.warp/.mcp.json` |
| Claude Code | `~/.claude.json` | `.mcp.json` |
| Codex | `~/.codex/config.toml` | `.codex/config.toml` |
| 其他 Agent | `~/.agents/.mcp.json` | `.agents/.mcp.json` |

在 WSL 中运行的 CLI 是一个 Linux 进程，读取的是 `/home/<user>/.claude.json` 这类 Linux 路径，而不是 `C:\Users\<user>\.claude.json`。

### 为什么不存在自动桥接

1. **文件型 MCP 只扫描本地路径。** Warpai 从宿主文件系统上当前会话的工作目录向上检索项目级配置；用户级配置同样位于本地主目录。桌面进程不会挂载或遍历 WSL 虚拟磁盘、远程 SSH 文件系统来发现配置。
2. **MCP Server 是子进程。** 宿主客户端在本地派发自己的 MCP Server。系统底层不存在跨 SSH 或 WSL 边界的通用 MCP stdio 隧道。
3. **Gallery 安装是本地记录。** 通过 Warpai Gallery 安装的 MCP 持久化在该桌面安装的 SQLite 数据库中。另一台电脑、一个 WSL 发行版和一台 SSH 主机都不会继承它。
4. **协同桥是按环境独立写入的。** 每个 Agent 的复选框只把 `warpai-wsl-com` 协同桥写入所选 WSL 账号，不会复制或同步你的其他 MCP Server。

### 常见问题

**为什么 Windows 上的 Warpai 已安装 MCP Server，WSL 中的 CLI Agent 仍提示没有 MCP 工具？**

因为 WSL 中的 Agent 读取的是 Linux 用户的配置。请在 WSL 内部安装该 MCP Server 的运行环境和配置。Windows 上 Warpai 的 MCP 安装属于另一层。

**在 WSL 中配置的 MCP Server 会出现在 Warpai 自己的 Agent 面板中吗？**

只有当 Agent 运行在该 WSL 环境并读取其 WSL 配置时才会。Warpai 自身的桌面 MCP 列表仍来自本地文件和本地数据库。

**可以把 `~/.warp/.mcp.json` 复制到 WSL 中吗？**

可以复制配置文件，但对应的 Server 命令也必须在 WSL 中存在且可执行。路径、运行时、凭据和环境变量都是环境专属的。在 WSL 中安装并配置该 Server 才是受支持的路径。

**Agent Communication 协同桥会把我的 MCP Server 带到远端吗？**

不会。它只在该环境中提供 Warpai 的协同工具，不携带第三方 MCP Server。

---

## 7. 多 Agent 协同工作流

`cd` 进入 WSL 项目后，Warpai 协同面板会绑定该项目。两种模式均可用：

- **Project 模式**：Agent 以对等方式协同，交换结构化问题、发布共享上下文，并从公共任务池认领任务。不要求 Git 仓库。
- **Worktree 模式**：Coordinator 在 WSL 仓库中创建物理 Git 工作树，分配隔离的 Worker，并组织对等评审。通过评审的变更由 Coordinator 集成。

### 任务生命周期与子任务自治

1. Coordinator 可随时派发任务，新任务进入接收方的持久化队列。
2. Worker 一次执行一个任务，提交后启动下一个排队任务。
3. 已派发的任务进入监督列表，在评审通过或得到明确的终态结果后移出。
4. **子任务自治：**
   > [!IMPORTANT]
   > Agent 内部执行的子任务状态并不会被外部监督列表监控，因为此时 Agent 本身就是可接收任务的就绪状态。Agent 内部自行拆解的微观步骤属于其内部细节；协同总线关注的是跨 Agent 的任务生命周期。

---

## 8. 与本地一致的核心开发体验

完成上述 WSL 配置后，Warpai 提供与本地项目一致的核心体验：

- **GPU 加速终端**：快速渲染、带状态码与耗时统计的命令块、智能补全与防休眠控制。
- **Project Explorer**：通过有界 stdio 文件分块，在 WSL 内浏览、新建、重命名和删除文件。
- **编辑器与预览**：在内置编辑器中编辑 WSL 文件，并预览带图表与数学公式的 Markdown。
- **可视化 Git Review**：在图形化 Diff 视图中检查 WSL 工作区变更。
- **守护式 PTY 交付**：后台监督协程观察 VT100 流，仅在 Agent 空闲且没有未提交草稿时投递排队指令。

---

## 9. 故障排查与安全

### 常见故障

| 现象 | 检查项 |
| :--- | :--- |
| `wsl --install` 失败或 WSL 无法启动 | BIOS/UEFI 中已启用虚拟化；Windows 功能已启用；启用后已重启。 |
| Warpai 的 WSL 账号下拉列表中看不到发行版 | 先在 Warpai 中打开 WSL 终端；用 `wsl --list --verbose` 确认发行版与用户。 |
| `wsl` 能打开，但命令块或 `cd` 跟踪不生效 | 运行 `wsl --shutdown`，重新打开会话，并确认使用的是 Warpai 1.7.8.2。 |
| WSL Companion 安装失败 | 客户机必须为 x86_64；检查磁盘空间和所选的发行版/用户；重试前先重新扫描。 |
| 安装后找不到 WSL Companion | 在同一个 Linux 账号下验证 `~/.config/warpai/wsl/bin/warpai-wsl-companion --check-runtime`。 |
| Agent 复选框不可用 | Companion 未安装或不兼容、CLI 未被探测到，或未选择账号。 |
| CLI 中仍然没有 MCP | 在新的 WSL 会话中启动 Agent；确认其 Linux 用户级配置文件中包含 Warpai 协同桥。 |
| Git 或编译很慢 | 将仓库从 `/mnt/c/...` 移到 `/home/<user>/...` 下的 Linux 文件系统。 |

### 安全基线

- 将 WSL Linux 用户视为拥有独立凭据和 MCP 配置的独立账号。
- 若 Agent 在 WSL 中运行，第三方 MCP Server 也应安装在 WSL 内；不要依赖 Windows 路径或 Windows 凭据。
- 启用前审查 MCP Server 命令。Warpai 的复选框只管理它自己的协同桥配置项。
- 不要在可能被提交到 Git 的项目级 MCP 文件中存放凭据。

### 验证清单

- [ ] `wsl --status` 显示默认版本为 WSL 2。
- [ ] `wsl --list --verbose` 显示发行版运行在版本 2。
- [ ] Warpai 能打开 WSL Shell 并跟踪 `cd`。
- [ ] **Settings > Features > Agent communication > WSL communication** 已开启，且已选择 WSL 账号。
- [ ] **Install WSL Companion** 已勾选，且 `warpai-wsl-companion --check-runtime` 通过。
- [ ] 所需的 Agent 复选框已启用，且 Agent 已在新会话中重启。
- [ ] 协同面板已绑定 WSL 项目，远端工具工作正常。
