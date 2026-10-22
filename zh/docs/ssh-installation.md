---
layout: docs
lang: zh
title: OpenSSH Server 安装与配置
description: "在 Linux、macOS 和 Windows 上安装并启用 OpenSSH Server，将 Windows SSH 登录 Shell 切换为 PowerShell，安装 Warpai Companion，并为 SSH 远程项目配置各 Agent 的 MCP。"
section: 远程接入
doc_id: ssh-installation
prev_url: /docs/install.html
prev_title: 安装指南
next_url: /docs/wsl-installation.html
next_title: Windows WSL 安装与配置
---

本文档覆盖 Warpai 1.7.8.2 的完整 SSH 远程配置：在 Linux、macOS 和 Windows 上安装并启用 OpenSSH Server，将 Windows SSH 登录 Shell 从 CMD 切换为 PowerShell，安装 Warpai Companion，按 Agent 配置远端 MCP，并让远程项目获得与本地一致的多 Agent 协作体验。

相关文档：
- [Windows WSL 安装与配置]({{ '/zh/docs/wsl-installation.html' | relative_url }})
- [远端组件与协议参考](https://github.com/OthinusG/warpai/blob/main/docs/REMOTE-INSTALLATION.zh-CN.md)
- [20+ 款 CLI Agent 配置]({{ '/zh/docs/cli-agents.html' | relative_url }})

---

## 1. 开始之前

| 项目 | 要求 |
| :--- | :--- |
| 客户端 | macOS、Windows 或 Linux 上的 Warpai 1.7.8.2 桌面端。 |
| 远端主机 | Linux、macOS 或 Windows，且拥有可安装软件的账号。 |
| 网络 | 客户端可访问远端 TCP 22 端口，或在 SSH config 中配置了自定义端口转发。 |
| 认证 | 强烈建议使用密钥认证或已加载的 SSH Agent。SSH Remote 面板使用非交互式 SSH 控制连接，无法输入密码。 |
| Host Key | 先在普通终端连接一次并确认主机指纹，再启用 SSH Remote。 |
| Companion | 远端账号下安装 Warpai Companion 5.5.0。文件浏览、Review 和 Agent 通信均依赖它。 |

只要 `sshd` 可达，普通终端 SSH 就能使用。SSH Remote 协作面板是独立开关的功能，还需要 Companion 和已确认的主机指纹。

---

## 2. 安装并启用 OpenSSH Server

### A. Linux

#### Ubuntu / Debian

```bash
sudo apt update && sudo apt install -y openssh-server
sudo systemctl enable --now ssh
sudo systemctl status ssh
```

如果启用了 UFW：

```bash
sudo ufw allow ssh
sudo ufw reload
```

#### RHEL / CentOS / Fedora / Rocky Linux / AlmaLinux

```bash
sudo dnf install -y openssh-server
sudo systemctl enable --now sshd
sudo systemctl status sshd
```

如果启用了 firewalld：

```bash
sudo firewall-cmd --permanent --add-service=ssh
sudo firewall-cmd --reload
```

#### Arch Linux

```bash
sudo pacman -S openssh
sudo systemctl enable --now sshd
```

#### 在服务端验证

```bash
systemctl is-active ssh 2>/dev/null || systemctl is-active sshd
ss -tlnp | grep ':22'
```

OpenSSH 的主配置位于 `/etc/ssh/sshd_config`，部分发行版还会读取 `/etc/ssh/sshd_config.d/` 下的片段。修改后先校验再重载：

```bash
sudo sshd -t
sudo systemctl reload ssh 2>/dev/null || sudo systemctl reload sshd
```

### B. macOS

macOS 自带 OpenSSH Server，对应系统设置中的“远程登录”，无需安装软件包。

#### 图形界面

1. 打开 **系统设置 > 通用 > 共享**。
2. 开启 **远程登录**。
3. 选择允许登录的用户。

#### 命令行

```bash
sudo systemsetup -setremotelogin on
sudo systemsetup -getremotelogin
```

#### 验证

```bash
sudo lsof -iTCP:22 -sTCP:LISTEN
```

用于 SSH 的账号必须拥有有效登录 Shell，并且被远程登录允许。Companion 控制连接还需要密钥认证或 SSH Agent。

### C. Windows 10/11

Windows 10 Build 1809 及更高版本、Windows 11 均以“可选功能”形式提供官方 Win32-OpenSSH 服务端。

#### 安装 OpenSSH Server

以**管理员身份**打开 PowerShell：

```powershell
Get-WindowsCapability -Online | Where-Object Name -like 'OpenSSH*'
Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0
```

如果系统无法在线下载该功能，请检查 Windows Update，或手动安装 Microsoft Win32-OpenSSH 发行版。

#### 启动服务并设置自启动

```powershell
Start-Service sshd
Set-Service -Name sshd -StartupType 'Automatic'

# 可选，但密钥认证建议一并启用
Start-Service ssh-agent
Set-Service -Name ssh-agent -StartupType 'Automatic'
```

#### 放行防火墙

安装器通常会自动创建规则。需要确认或手动创建时：

```powershell
if (!(Get-NetFirewallRule -Name 'OpenSSH-Server-In-TCP' -ErrorAction SilentlyContinue)) {
    New-NetFirewallRule -Name 'OpenSSH-Server-In-TCP' `
        -DisplayName 'OpenSSH Server (sshd)' -Enabled True `
        -Direction Inbound -Protocol TCP -Action Allow -LocalPort 22
}
```

#### 验证

```powershell
Get-Service sshd
Get-NetTCPConnection -LocalPort 22 -State Listen
```

#### Windows 认证说明

SSH Remote 面板使用 `BatchMode=yes` 和 `StrictHostKeyChecking=yes` 连接。仅支持密码的账号可以打开交互式终端，但面板的独立控制连接无法输入密码。请先配置密钥或 ssh-agent。

普通（非管理员）账号的公钥放在：

```text
C:\Users\<user>\.ssh\authorized_keys
```

Administrators 组账号则由 Windows OpenSSH 读取：

```text
C:\ProgramData\ssh\administrators_authorized_keys
```

该文件必须只允许 Administrators 和 SYSTEM 读取。典型 ACL 配置：

```powershell
icacls "C:\ProgramData\ssh\administrators_authorized_keys" /inheritance:r
icacls "C:\ProgramData\ssh\administrators_authorized_keys" /grant "Administrators:F" "SYSTEM:F"
```

### D. 从客户端验证

```bash
ssh -V
ssh <user>@<host>
```

首次连接时确认主机指纹。如果连接失败，请先解决 `sshd`、防火墙或认证问题，再安装 Companion。

---

## 3. 将 Windows SSH 登录 Shell 切换为 PowerShell

<div class="docs-callout warning">
  <div class="docs-callout-title">必须使用 PowerShell</div>
  <p>Windows OpenSSH 默认使用 <code>cmd.exe</code> 作为交互式 SSH 会话的 Shell。Warpai 的远程 Windows 集成只支持两类远端 Shell：POSIX Shell 和 PowerShell。<strong>CMD 不是受支持的远程开发 Shell</strong>。按 Agent 配置 MCP 和协作面板都依赖 PowerShell。</p>
</div>

在远端 Windows 主机上以**管理员身份**打开 PowerShell 执行以下操作。

### 选项 A：Windows PowerShell 5.1（系统默认，推荐）

```powershell
$key = 'HKLM:\SOFTWARE\OpenSSH'
if (!(Test-Path $key)) { New-Item -Path $key -Force | Out-Null }
New-ItemProperty -Path $key -Name DefaultShell `
    -Value 'C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe' `
    -PropertyType String -Force
```

### 选项 B：PowerShell 7

先确认 PowerShell 7 已安装：

```powershell
Get-Command pwsh -ErrorAction SilentlyContinue
```

然后设置：

```powershell
$key = 'HKLM:\SOFTWARE\OpenSSH'
if (!(Test-Path $key)) { New-Item -Path $key -Force | Out-Null }
New-ItemProperty -Path $key -Name DefaultShell `
    -Value 'C:\Program Files\PowerShell\7\pwsh.exe' `
    -PropertyType String -Force
```

### 验证

```powershell
Get-ItemProperty -Path 'HKLM:\SOFTWARE\OpenSSH' -Name DefaultShell
```

从客户端重新连接：

```bash
ssh <user>@<windows-host>
```

登录后提示符显示为 `PS C:\Users\<user>>`，即表示已切换到 PowerShell。注册表值在新建会话时读取，已有会话不会改变。通常无需重启 `sshd`；排查问题时可以安全地执行：

```powershell
Restart-Service sshd
```

注册表值必须是绝对路径，且不要包含外层引号。

---

## 4. 在远端主机安装 Warpai Companion

Companion 提供私有 Git 引擎、基于 SFTP 的文件浏览、VT100 状态观测和 Agent 通信桥。它必须安装在**执行 SSH 登录的同一账号**下。

| 远端系统 | 架构 | 安装包 | 安装方式 |
| :--- | :--- | :--- | :--- |
| **Linux** | x86_64 | [Companion Linux x64](https://github.com/OthinusG/warpai/releases/download/v1.7.8.2/WarpaiCompanion-5.5.0-linux-x64.run) | `sh WarpaiCompanion-5.5.0-linux-x64.run` |
| **macOS** | Apple Silicon (ARM64) | [Companion macOS ARM64](https://github.com/OthinusG/warpai/releases/download/v1.7.8.2/WarpaiCompanion-5.5.0-macos-arm64.dmg) | 打开镜像并运行 `Install Warpai Companion.command` |
| **Windows** | x86_64 | [Companion Windows x64](https://github.com/OthinusG/warpai/releases/download/v1.7.8.2/WarpaiCompanion-5.5.0-windows-x64-setup.exe) | 以远程 SSH 用户身份运行安装程序 |

Companion 5.5.0 只发布上述三个目标。其他远端架构不在本版本支持范围内。

### 固定安装路径

- **Linux / macOS：** `~/.config/warpai/bin/warpai-companion`
- **Windows：** `%USERPROFILE%\.config\warpai\bin\warpai-companion.exe`

### 验证运行环境

```bash
# Linux / macOS
~/.config/warpai/bin/warpai-companion --check-runtime
```

```powershell
# Windows PowerShell
& "$env:USERPROFILE\.config\warpai\bin\warpai-companion.exe" --check-runtime
```

安装包自带私有 Git 运行时，远端账号无需预装系统 Git、Python、Node.js、tmux 或 socat。各厂商 Agent CLI 仍使用自己的运行时和认证。

Companion 按需启动；所有连接和 Agent 停止后，空闲 60 秒自动退出，不注册常驻系统服务。

Windows 上请使用拥有 SSH 会话的同一账号安装。安装器写入该账号的用户目录；以其他用户安装会导致 SSH 账号找不到二进制文件。

---

## 5. 在 Warpai 中配置 SSH Remote 与按 Agent 的 MCP

### 5.1 连接并确认远程项目

在 Warpai 终端中：

```bash
ssh user@remote-host
cd /absolute/path/to/project
```

首次连接必须确认主机指纹。Warpai 跟随焦点终端中已确认的远程 Shell 和工作目录；仅安装 Companion 并不会让未确认的 SSH 会话自动报告项目。

Windows 会话中，终端横幅可能提示 **Integrate PowerShell**。按提示接受即可，该操作只对当前会话生效，不会写入 PowerShell profile。

### 5.2 启用 SSH Remote

1. 打开 **Settings > Features > Agent communication**。
2. 打开 **SSH Remote** 开关。
3. 在 **SSH account** 下拉框中选择已确认且当前登录的账号。此界面不接受手动输入凭据或别名。

该开关控制远程文件浏览、Review 和 Agent 通信。普通终端 SSH 不受影响。关闭 SSH Remote 会撤销远端 MCP 参与并解绑远程面板，但不会关闭终端会话。

### 5.3 配置按 Agent 的 MCP

Companion 5.5.0 安装完成且账号已选中后：

1. Warpai 会发现该远端账号下已安装的受支持 CLI Agent。
2. 每个 Agent 对应一个复选框。勾选后，Warpai 会把 `warpai-ssh-com` 通信桥写入该 Agent 自己的远端配置。
3. 安装或更新 CLI 后点击 **Rescan agents**。
4. **Remove Warpai MCP from all agents** 只移除 Warpai 自己写入的条目。
5. **SSH Companion setup** 打开远端组件安装参考。

MCP 变更只作用于当前选中的 SSH 账号。请在**新的远程终端会话**中启动对应 Agent，使其重新加载配置。版本过旧、不支持原生 MCP 配置的 Companion 会显示更新要求，而不是可用的复选框。

Warpai 会保留厂商认证和其他无关的 MCP Server。复选框配置的是协同通信桥，不是你的第三方 MCP 清单。

### 5.4 配置写入位置

在远端账号下，Warpai 会把通信桥写入各 Agent 的原生配置，例如：

- Claude Code：`~/.claude.json`（项目级：`.mcp.json`）
- Codex：`~/.codex/config.toml`（项目级：`.codex/config.toml`）
- 其他受支持 Agent：其文档规定的用户级 MCP 配置文件

这些都是**远端机器上的文件**，不是 Warpai 桌面进程的 MCP 配置。

升级至 **1.7.8.2** 后，重新扫描会将已保存记录中的安装、查询和清理命令迁移至 `warpai-ssh-com`。处于未启用状态的清理记录可重新勾选安装；已有用户 MCP 配置项和启用选择均保留。随后启动新的 Agent 会话以重新加载配置。

---

## 6. 两层 MCP 架构

<div class="docs-callout important">
  <div class="docs-callout-title">MCP 属于 Agent 的运行环境</div>
  <p>MCP 的配置、安装记录和执行进程属于实际运行 Agent 的那台机器、那个用户和那个环境。SSH 只是终端与控制传输层，<strong>不会把客户端的 MCP Server 隧道或同步到远端</strong>。</p>
</div>

实际影响：

- 在 Mac 上的 Warpai 中安装的第三方 MCP，不会自动出现在远端主机运行的 CLI Agent 中。需要在远端主机单独安装并配置。
- 远端项目里的 `.mcp.json` 不会被本地 Warpai 桌面进程自动发现；本地应用只扫描本地路径。
- 通过 Warpai Gallery 安装的 MCP 记录保存在本地 SQLite 数据库，不会跟随你到另一台电脑，也不会进入 WSL/SSH 环境。
- Agent 复选框写入的通信桥只包含 Warpai 协同工具，不包含你的个人 MCP Server。

关于完整的两层 MCP 模型、路径解析细节和 WSL 侧的对应行为，请参见：

👉 **[Windows WSL 安装与配置：深入理解两层 MCP 架构与环境隔离]({{ '/zh/docs/wsl-installation.html' | relative_url }}#two-tier-mcp-architecture-and-environment-isolation)**

---

## 7. 多 Agent 协同工作流

远程项目确认且 SSH Remote 启用后，协作面板会绑定到该项目。两种模式都可以在远端使用：

- **Project 模式：** Agent 之间交换消息、结构化问答、共享上下文并使用公共任务池，无需 Git 仓库。
- **Worktree 模式：** Coordinator 创建隔离的 Git worktree、分派 Worker 并组织对等评审；验收通过的改动由 Coordinator 集成。

更换远端 CLI 安装后使用 **Rescan agents**。排队任务只会在目标 Agent 处于安全状态（空闲 Shell、无待提交草稿）时投递。

### 子任务自治与就绪状态

<div class="docs-callout important">
  <div class="docs-callout-title">子任务自治与就绪状态</div>
  <p>Agent 内部执行的子任务状态不会被外部监督列表监控，因为此时 Agent 本身就是可接收任务的就绪状态。协同中枢跟踪的是跨 Agent 的任务生命周期；Agent 内部自行拆解的工作属于其自治范围。</p>
</div>

---

## 8. 与本地一致的远程开发体验

通过 SSH 连接后，Warpai 保持与本地项目相同的核心工作流：

1. **命令块：** GPU 渲染的命令块，包含退出状态、耗时和块级复制。
2. **Project Explorer：** 通过 Companion 的 SFTP 文件通道浏览和修改远端文件。
3. **编辑器与预览：** 在内置编辑器中编辑远端文件，并预览带图表和数学公式的远端 Markdown。
4. **可视化 Git Review：** 在图形化 Diff 视图中检查远端工作区改动。
5. **守护式 PTY 投递：** 后台监督器观察远端 VT100 流，仅在 Agent 空闲且无待提交草稿时投递排队指令。

---

## 9. 故障排查与安全

### 常见问题

| 现象 | 检查项 |
| :--- | :--- |
| `Connection refused` 或超时 | `sshd` 是否运行、防火墙是否放行端口、客户端能否访问主机。 |
| `Permission denied (publickey)` | 账号、密钥和 `authorized_keys` 权限是否正确。Windows 管理员账号使用 `administrators_authorized_keys`。 |
| 终端能连接，但 SSH Remote 提示认证不可用 | 控制连接使用 `BatchMode=yes`，无法输入密码。配置密钥或 SSH Agent。 |
| 面板提示主机密钥未知 | 先在 Warpai 终端连接一次并确认指纹。控制连接使用 `StrictHostKeyChecking=yes`。 |
| Windows 登录仍然是 CMD | 在 `HKLM:\SOFTWARE\OpenSSH` 设置 `DefaultShell`，然后新建 SSH 会话。 |
| Companion 缺失或不兼容 | 安装对应系统和架构的 Companion 5.5.0，运行 `--check-runtime`，然后重连或 Rescan。 |
| Agent 复选框不可用 | Companion 缺失/过旧、CLI 未被发现、账号未确认，或正在执行其他设置操作。 |
| CLI 中仍然没有 MCP | Agent 可能在勾选前已经启动。新建会话启动 Agent，并检查远端配置文件。 |
| 终端 Agent 有 MCP，Warpai 中没有 | 两层体系不同：终端 Agent 读取远端配置，Warpai 桌面进程读取本地配置。 |
| Windows 面板无法认证 | 原生 Windows OpenSSH 不支持 ControlMaster。独立控制连接需要密钥/Agent 认证，仅密码不够。 |

### 安全基线

- 优先使用密钥认证；在策略允许时关闭 `sshd_config` 中的密码认证。
- 不要把 22 端口直接暴露到公网；至少配合仅密钥认证、防火墙和速率限制。
- 使用实际开发所需的最小权限账号安装 Companion。
- 桌面端和 Companion 使用同一版本的安装包；绑定前会检查协议兼容性。
- SSH 控制连接会禁用 Agent 转发、X11 转发和本地命令，同时保留系统 SSH 配置、密钥和主机密钥策略。

### 验证清单

- [ ] 普通终端中 `ssh user@host` 成功。
- [ ] 已确认主机密钥指纹。
- [ ] Windows 上 `ssh` 登录后进入 PowerShell（`PS C:\...>`）。
- [ ] 远端账号下 `warpai-companion --check-runtime` 成功。
- [ ] **Settings > Features > Agent communication > SSH Remote** 已开启并选中账号。
- [ ] 所需 Agent 复选框已启用，且 Agent 已重新启动。
- [ ] 远端文件、Review 和协作面板可在已确认项目上正常工作。
