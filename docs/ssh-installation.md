---
layout: docs
title: OpenSSH Server Installation & Setup
description: "Install and enable OpenSSH Server on Linux, macOS, and Windows, switch the Windows SSH shell to PowerShell, install Warpai Companion, and configure per-Agent MCP for SSH remote projects."
section: Remote Access
doc_id: ssh-installation
prev_url: /docs/install.html
prev_title: Installation & Setup
next_url: /docs/wsl-installation.html
next_title: Windows WSL Installation & Setup
---

This guide covers the complete SSH remote setup for Warpai 1.7.8.2: installing and enabling OpenSSH Server on Linux, macOS, and Windows, switching the Windows SSH login shell from CMD to PowerShell, installing Warpai Companion, configuring per-Agent MCP on the remote account, and running the same multi-Agent workflow as a local project.

Related documentation:
- [Windows WSL Installation & Setup]({{ '/docs/wsl-installation.html' | relative_url }})
- [Remote Components and Protocol Reference](https://github.com/OthinusG/warpai/blob/main/docs/REMOTE-INSTALLATION.md)
- [20+ CLI Agents Guide]({{ '/docs/cli-agents.html' | relative_url }})

---

## 1. Before You Begin

| Requirement | Details |
| :--- | :--- |
| Client | Warpai 1.7.8.2 desktop on macOS, Windows, or Linux. |
| Remote host | Linux, macOS, or Windows with an account that can install software. |
| Network | TCP port 22 reachable from the client, or a custom port forwarded through your SSH config. |
| Authentication | Key-based authentication or a loaded SSH agent is strongly recommended. The SSH Remote panel uses a non-interactive SSH control connection and cannot type a password. |
| Host key | Connect once from a normal terminal and confirm the host fingerprint before enabling SSH Remote. |
| Companion | Warpai Companion 5.5.0 on the remote account. It is required for file exploration, Review, and Agent communication. |

Basic terminal SSH works as soon as `sshd` is reachable. The SSH Remote collaboration panel is a separate, independently switched feature that also requires Companion and a trusted host key.

---

## 2. Install and Enable OpenSSH Server

### A. Linux

#### Ubuntu / Debian

```bash
sudo apt update && sudo apt install -y openssh-server
sudo systemctl enable --now ssh
sudo systemctl status ssh
```

If UFW is enabled:

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

If firewalld is enabled:

```bash
sudo firewall-cmd --permanent --add-service=ssh
sudo firewall-cmd --reload
```

#### Arch Linux

```bash
sudo pacman -S openssh
sudo systemctl enable --now sshd
```

#### Verify on the server

```bash
systemctl is-active ssh 2>/dev/null || systemctl is-active sshd
ss -tlnp | grep ':22'
```

OpenSSH reads its configuration from `/etc/ssh/sshd_config` (and drop-in files under `/etc/ssh/sshd_config.d/` on many distributions). Validate a change before reloading:

```bash
sudo sshd -t
sudo systemctl reload ssh 2>/dev/null || sudo systemctl reload sshd
```

### B. macOS

macOS ships OpenSSH Server as **Remote Login**; no package installation is required.

#### Graphical method

1. Open **System Settings > General > Sharing**.
2. Enable **Remote Login**.
3. Choose which users may connect.

#### Command-line method

```bash
sudo systemsetup -setremotelogin on
sudo systemsetup -getremotelogin
```

#### Verify

```bash
sudo lsof -iTCP:22 -sTCP:LISTEN
```

The account used for SSH must have a valid login shell and must be allowed by Remote Login. The Companion control connection also needs key-based authentication or an SSH agent.

### C. Windows 10/11

Windows 10 build 1809 and later, and Windows 11, include the official Win32-OpenSSH server as an Optional Feature.

#### Install OpenSSH Server

Run **PowerShell as Administrator**:

```powershell
Get-WindowsCapability -Online | Where-Object Name -like 'OpenSSH*'
Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0
```

If the capability cannot be downloaded, check Windows Update or install the Microsoft Win32-OpenSSH release manually.

#### Start and automate the service

```powershell
Start-Service sshd
Set-Service -Name sshd -StartupType 'Automatic'

# Optional but recommended for key-based authentication
Start-Service ssh-agent
Set-Service -Name ssh-agent -StartupType 'Automatic'
```

#### Open the firewall

The installer normally creates the rule. To confirm or create it manually:

```powershell
if (!(Get-NetFirewallRule -Name 'OpenSSH-Server-In-TCP' -ErrorAction SilentlyContinue)) {
    New-NetFirewallRule -Name 'OpenSSH-Server-In-TCP' `
        -DisplayName 'OpenSSH Server (sshd)' -Enabled True `
        -Direction Inbound -Protocol TCP -Action Allow -LocalPort 22
}
```

#### Verify

```powershell
Get-Service sshd
Get-NetTCPConnection -LocalPort 22 -State Listen
```

#### Authentication note for Windows

The SSH Remote panel connects with `BatchMode=yes` and `StrictHostKeyChecking=yes`. A password-only account can open an interactive terminal session, but the panel cannot authenticate its independent control connection. Configure a key or ssh-agent first.

For a normal (non-administrator) account, place the public key in:

```text
C:\Users\<user>\.ssh\authorized_keys
```

For an account in the Administrators group, Windows OpenSSH reads:

```text
C:\ProgramData\ssh\administrators_authorized_keys
```

That file must be readable only by Administrators and SYSTEM. A typical ACL setup is:

```powershell
icacls "C:\ProgramData\ssh\administrators_authorized_keys" /inheritance:r
icacls "C:\ProgramData\ssh\administrators_authorized_keys" /grant "Administrators:F" "SYSTEM:F"
```

### D. Verify from the client

```bash
ssh -V
ssh <user>@<host>
```

Accept the host key on the first connection. If the connection fails, fix `sshd`, the firewall, or authentication before installing Companion.

---

## 3. Switch the Windows SSH Login Shell to PowerShell

<div class="docs-callout warning">
  <div class="docs-callout-title">PowerShell Is Required</div>
  <p>Windows OpenSSH defaults to <code>cmd.exe</code> for interactive SSH sessions. Warpai's remote Windows integration only supports two remote shell families, POSIX shells and PowerShell. <strong>CMD is not a supported remote development shell</strong>. Per-Agent MCP configuration and the collaboration panel rely on PowerShell.</p>
</div>

Run the following in **PowerShell as Administrator on the remote Windows host**.

### Option A: Windows PowerShell 5.1 (default, recommended)

```powershell
$key = 'HKLM:\SOFTWARE\OpenSSH'
if (!(Test-Path $key)) { New-Item -Path $key -Force | Out-Null }
New-ItemProperty -Path $key -Name DefaultShell `
    -Value 'C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe' `
    -PropertyType String -Force
```

### Option B: PowerShell 7

Check that PowerShell 7 is installed:

```powershell
Get-Command pwsh -ErrorAction SilentlyContinue
```

If it is present, set:

```powershell
$key = 'HKLM:\SOFTWARE\OpenSSH'
if (!(Test-Path $key)) { New-Item -Path $key -Force | Out-Null }
New-ItemProperty -Path $key -Name DefaultShell `
    -Value 'C:\Program Files\PowerShell\7\pwsh.exe' `
    -PropertyType String -Force
```

### Verify

```powershell
Get-ItemProperty -Path 'HKLM:\SOFTWARE\OpenSSH' -Name DefaultShell
```

Reconnect from the client:

```bash
ssh <user>@<windows-host>
```

A successful login shows a PowerShell prompt such as `PS C:\Users\<user>>`. The registry value is read when a new session starts; existing sessions keep their old shell. Restarting `sshd` is not normally required, but it is safe if you are troubleshooting:

```powershell
Restart-Service sshd
```

Keep the registry value as an absolute path without surrounding quotes.

---

## 4. Install Warpai Companion on the Remote Host

Companion provides the private Git engine, SFTP-backed file exploration, VT100 observation, and the Agent communication bridge. It runs under the **same account you use for SSH**.

| Remote OS | Architecture | Package | Installation |
| :--- | :--- | :--- | :--- |
| **Linux** | x86_64 | [Companion Linux x64](https://github.com/OthinusG/warpai/releases/download/v1.7.8.2/WarpaiCompanion-5.5.0-linux-x64.run) | `sh WarpaiCompanion-5.5.0-linux-x64.run` |
| **macOS** | Apple Silicon (ARM64) | [Companion macOS ARM64](https://github.com/OthinusG/warpai/releases/download/v1.7.8.2/WarpaiCompanion-5.5.0-macos-arm64.dmg) | Open the image and run `Install Warpai Companion.command` |
| **Windows** | x86_64 | [Companion Windows x64](https://github.com/OthinusG/warpai/releases/download/v1.7.8.2/WarpaiCompanion-5.5.0-windows-x64-setup.exe) | Run the installer as the remote SSH user |

Companion 5.5.0 is published for these three targets. Other remote architectures are not supported by this release.

### Stable installation paths

- **Linux / macOS:** `~/.config/warpai/bin/warpai-companion`
- **Windows:** `%USERPROFILE%\.config\warpai\bin\warpai-companion.exe`

### Verify the runtime

```bash
# Linux / macOS
~/.config/warpai/bin/warpai-companion --check-runtime
```

```powershell
# Windows PowerShell
& "$env:USERPROFILE\.config\warpai\bin\warpai-companion.exe" --check-runtime
```

The installer bundles a private Git runtime and does not require system Git, Python, Node.js, tmux, or socat on the remote account. Vendor Agent CLIs keep their own runtimes and authentication.

Companion starts on demand and exits after 60 idle seconds once connections and Agents stop. It does not register a persistent system service.

For Windows, install with the same account that owns the SSH session. The installer writes into that account's profile; installing as another user puts the binary where the SSH account cannot find it.

---

## 5. Configure SSH Remote and Per-Agent MCP in Warpai

### 5.1 Connect and confirm the remote project

In a Warpai terminal:

```bash
ssh user@remote-host
cd /absolute/path/to/project
```

The first connection must confirm the host key. Warpai follows the focused terminal's confirmed remote shell and working directory; installation alone does not make an unconfirmed session report a project.

On Windows, the SSH terminal may offer **Integrate PowerShell** in its banner. Accept it for that session if prompted. It does not write a PowerShell profile.

### 5.2 Enable SSH Remote

1. Open **Settings > Features > Agent communication**.
2. Turn on **SSH Remote**.
3. Select the confirmed, logged-in **SSH account** from the dropdown. Warpai does not accept manually typed credentials or aliases here.

The switch gates remote file exploration, Review, and Agent communication. Basic terminal SSH remains independent. Turning SSH Remote off revokes remote MCP participation and detaches remote panels; it does not close your terminal session.

### 5.3 Configure per-Agent MCP

With Companion 5.5.0 installed and the account selected:

1. Warpai discovers managed CLI Agents installed on that remote account.
2. Each discovered Agent has a checkbox. Enabling it writes Warpai's `warpai-ssh-com` bridge into that Agent's own remote configuration.
3. Use **Rescan agents** after installing or updating a CLI.
4. Use **Remove Warpai MCP from all agents** to remove only the Warpai-owned entries.
5. **SSH Companion setup** opens the remote component installation reference.

MCP changes apply only to the selected SSH account. Start the selected Agent in a **new remote terminal session** so it reloads its configuration. An old Companion that does not advertise native MCP setup shows an update requirement instead of usable checkboxes.

Warpai preserves vendor authentication and unrelated MCP servers. The checkbox configures the collaboration bridge, not your third-party MCP catalog.

### 5.4 Where the configurations live

On the remote account, Warpai writes the bridge entry into each Agent's native configuration, for example:

- Claude Code: `~/.claude.json` (project scope: `.mcp.json`)
- Codex: `~/.codex/config.toml` (project scope: `.codex/config.toml`)
- Other supported Agents: their documented user-level MCP file

These are remote files on the remote machine. They are not the MCP configuration of the Warpai desktop process.

After upgrading to **1.7.8.2**, rescanning migrates saved install, lookup, and cleanup commands to `warpai-ssh-com`. Inactive cleanup records can be selected again to reinstall; existing user MCP entries and activation choices are preserved. Start a new Agent session afterward to reload the configuration.

---

## 6. Two-Tier MCP Architecture

<div class="docs-callout important">
  <div class="docs-callout-title">MCP Belongs to the Agent's Environment</div>
  <p>MCP configuration, installation records, and server processes belong to the machine, user, and environment where the Agent actually runs. SSH is a terminal and control transport; it does <strong>not</strong> tunnel or synchronize MCP servers from the client.</p>
</div>

The practical consequences are:

- A third-party MCP installed in Warpai on your Mac is not available to a CLI Agent running on the remote host. Install and configure it on the remote host.
- A remote project's `.mcp.json` is not auto-discovered by the local Warpai desktop process. The local app scans local paths only.
- MCP servers installed through the Warpai Gallery are stored in the local SQLite database and do not follow you to another machine or into a WSL/SSH environment.
- The bridge configured by the Agent checkboxes contains only Warpai's collaboration tools, not your personal MCP servers.

For the full two-tier model, path resolution details, and the WSL equivalent, see:

👉 **[WSL Installation & Setup: Two-Tier MCP Architecture and Environment Isolation]({{ '/docs/wsl-installation.html' | relative_url }}#two-tier-mcp-architecture-and-environment-isolation)**

---

## 7. Multi-Agent Collaboration Workflow

Once the remote project is confirmed and SSH Remote is enabled, the collaboration panel binds to that project. Both modes work remotely:

- **Project Mode:** Peers exchange messages, structured questions, shared context, and pooled tasks. No Git repository is required.
- **Worktree Mode:** The Coordinator creates isolated Git worktrees, assigns workers, and requests peer review. Accepted work is integrated by the Coordinator.

Use **Rescan agents** after changing a remote CLI installation. Queued tasks are delivered only when the selected Agent is in a safe state (idle shell, no pending draft input).

### Subtask autonomy and ready state

<div class="docs-callout important">
  <div class="docs-callout-title">Subtask Autonomy and Ready State</div>
  <p>An Agent's internal subtask states are not monitored by the external supervision list, because the Agent itself is in a ready state that can receive tasks. The collaboration fabric tracks cross-Agent task lifecycles; work an Agent decomposes internally remains internal to that Agent.</p>
</div>

---

## 8. Local-Parity Remote Development Experience

Connecting over SSH in Warpai keeps the same core workflow as a local project:

1. **Command blocks:** GPU-rendered blocks with exit status, duration, and block-level copy.
2. **Project Explorer:** Browse and mutate remote files through the Companion's SFTP-backed file channel.
3. **Editor and previews:** Edit remote files in the built-in editor and preview remote Markdown with diagrams and math.
4. **Visual Git Review:** Inspect remote working-tree changes in the graphical diff view.
5. **Guarded PTY delivery:** The supervisor observes the remote VT100 stream and delivers queued instructions only when the Agent is idle with no pending draft.

---

## 9. Troubleshooting and Security

### Common failures

| Symptom | Check |
| :--- | :--- |
| `Connection refused` or timeout | `sshd` is running, the firewall allows the port, and the client can reach the host. |
| `Permission denied (publickey)` | The correct account, key, and `authorized_keys` permissions are in place. On Windows administrators, use `administrators_authorized_keys`. |
| Terminal connects, but SSH Remote says authentication unavailable | The control channel uses `BatchMode=yes` and cannot prompt for a password. Configure a key or SSH agent. |
| Panel says host key is unknown | Connect once from the Warpai terminal and confirm the fingerprint. The control channel uses `StrictHostKeyChecking=yes`. |
| Windows login still opens CMD | Set `DefaultShell` under `HKLM:\SOFTWARE\OpenSSH`, then open a new SSH session. |
| Companion missing or incompatible | Install Companion 5.5.0 for the remote OS/architecture, verify `--check-runtime`, then reconnect or rescan. |
| Agent checkbox is disabled | Companion is missing/old, the CLI was not discovered, the account is not confirmed, or a setup operation is busy. |
| MCP still absent in the CLI | The Agent may have started before the checkbox was enabled. Start a new session and confirm the remote config file. |
| MCP works in the terminal Agent but not in Warpai | They are different tiers: the terminal Agent reads remote config; the Warpai desktop process reads local config. |
| Windows panel cannot authenticate | Native Windows OpenSSH has no ControlMaster. The independent control connection needs key/agent auth; password-only login is not enough. |

### Security baseline

- Prefer key-based authentication, then disable password authentication in `sshd_config` if policy allows.
- Do not expose port 22 to the public internet without key-only auth, a firewall, and rate limiting.
- Install Companion under the least-privileged account you actually use for development.
- Keep desktop and Companion installers from the same release; protocol compatibility is checked before binding.
- The SSH control channel disables agent forwarding, X11 forwarding, and local commands. It preserves your system SSH config, keys, and host-key policy.

### Verification checklist

- [ ] `ssh user@host` succeeds from a normal terminal.
- [ ] The host key fingerprint is confirmed.
- [ ] On Windows, `ssh` lands in PowerShell (`PS C:\...>`).
- [ ] `warpai-companion --check-runtime` succeeds on the remote account.
- [ ] **Settings > Features > Agent communication > SSH Remote** is on and the account is selected.
- [ ] The required Agent checkboxes are enabled and the Agent was restarted.
- [ ] Remote files, Review, and collaboration panels operate on the confirmed project.
