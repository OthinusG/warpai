---
layout: docs
title: Windows WSL Installation & Setup
description: "Enable WSL 2, install a Linux distribution, open it in Warpai, install the dedicated WSL Companion, and configure per-Agent MCP inside the distribution."
section: Remote Access
doc_id: wsl-installation
prev_url: /docs/ssh-installation.html
prev_title: OpenSSH Server Installation & Setup
next_url: /docs/cli-agents.html
next_title: 20+ CLI Agents Guide
---

This guide covers Warpai 1.7.8.2 with WSL 2 on Windows: enabling the required Windows features, installing a Linux distribution, opening the WSL session in Warpai, installing the dedicated WSL Companion, configuring per-Agent MCP inside the distribution, and running the same multi-Agent workflow as a local project.

Related documentation:
- [OpenSSH Server Installation & Setup]({{ '/docs/ssh-installation.html' | relative_url }})
- [Remote Components and Protocol Reference](https://github.com/OthinusG/warpai/blob/main/docs/REMOTE-INSTALLATION.md)
- [20+ CLI Agents Guide]({{ '/docs/cli-agents.html' | relative_url }})

---

## 1. Before You Begin

| Requirement | Details |
| :--- | :--- |
| Windows | Windows 10 version 2004 (build 19041) or later, or Windows 11. |
| Virtualization | Hardware virtualization enabled in BIOS/UEFI. |
| Warpai | Warpai 1.7.8.2 for Windows. WSL integration is a Windows-only feature. |
| Distribution | A WSL 2 distribution, preferably Ubuntu 22.04 LTS or 24.04 LTS. The bundled WSL Companion supports x86_64 guests. |
| Account | A Linux user inside the distribution. The Companion installs into that user's home directory. |

WSL is a local Windows integration. It does not require an SSH server, port forwarding, or the SSH Companion package.

---

## 2. Enable WSL 2 and Install a Distribution

### Step 1: Enable the Windows features

Open **PowerShell as Administrator**. On current Windows builds, one command enables the required components:

```powershell
wsl --install
```

To enable the features explicitly instead:

```powershell
dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart
dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart
```

Restart Windows when prompted. If virtualization is disabled in firmware, enable Intel VT-x / AMD-V (or the equivalent) in BIOS/UEFI first.

### Step 2: Confirm WSL 2

```powershell
wsl --update
wsl --set-default-version 2
wsl --status
```

`wsl --status` should report WSL 2 as the default version.

### Step 3: Install a distribution

```powershell
wsl --list --online
wsl --install -d Ubuntu
```

The first launch asks for a UNIX username and password. That Linux account owns the Companion installation and is the account shown in Warpai's WSL settings.

### Step 4: Verify

```powershell
wsl --list --verbose
```

The distribution should show `VERSION 2` and `STATE Running` (or `Stopped` after exit).

---

## 3. Open the WSL Session in Warpai

In a Warpai terminal on Windows:

```powershell
wsl
```

Or select a distribution and user explicitly:

```powershell
wsl.exe -d Ubuntu -u <linux-user>
```

Warpai detects the WSL shell, loads session-level shell integration, and enables command blocks, history, and directory tracking. The left collaboration panel follows the confirmed WSL project after you `cd` into it.

### Workspace recommendation

Keep repositories inside the Linux filesystem, for example:

```bash
cd /home/<linux-user>/projects/<repo>
```

Avoid running Git operations or heavy builds under `/mnt/c/...`. The 9p mount is convenient for sharing files but has much lower I/O throughput and can introduce file-locking contention.

If the WSL session or shell integration becomes stuck, reset the distribution from PowerShell and reopen it:

```powershell
wsl --shutdown
```

---

## 4. Install the WSL Companion

Warpai for Windows bundles the dedicated WSL guest payload. The WSL Companion is installed per Linux user and per distribution; it is not the SSH Companion package.

### Recommended: install from Warpai settings

1. Open **Settings > Features > Agent communication**.
2. Turn on **WSL communication**.
3. Open a WSL terminal in Warpai first, then select the **WSL account** (distribution and Linux user) from the dropdown.
4. Check **Install WSL Companion**.
5. Warpai streams the bundled, verified Linux x86_64 payload into the selected distribution over `wsl.exe` standard input. No external download is required.
6. Watch progress with **View installation progress** if the operation takes a moment.

Unchecking **Install WSL Companion** removes the installation for that account. **Uninstall WSL Companion** performs the same removal from the maintenance buttons.

<div class="docs-callout note">
  <div class="docs-callout-title">WSL Uses Its Own Companion Payload</div>
  <p>The Linux <code>WarpaiCompanion-5.5.0-linux-x64.run</code> package published on GitHub is for <strong>SSH remote hosts</strong>. It installs <code>~/.config/warpai/bin/warpai-companion</code> and does not install the dedicated WSL service at <code>~/.config/warpai/wsl/bin/warpai-wsl-companion</code>. Use the WSL settings installer for local WSL.</p>
</div>

### Installation paths

- Base executable: `~/.config/warpai/bin/warpai-companion`
- Dedicated WSL service: `~/.config/warpai/wsl/bin/warpai-wsl-companion`

### Verify the runtime

Inside the WSL terminal:

```bash
~/.config/warpai/wsl/bin/warpai-wsl-companion --check-runtime
```

Companion runs on demand. It starts when a connection or Agent collaboration session needs it and exits after 60 idle seconds. It does not register a persistent system service or modify shell startup files.

---

## 5. Configure Per-Agent MCP Inside WSL

MCP configuration belongs to the environment where the Agent runs. For an Agent inside WSL, that is the Linux user's own configuration inside the distribution.

With **WSL communication** enabled and the WSL account selected:

1. Warpai discovers supported CLI Agents installed inside that distribution.
2. Each discovered Agent has a checkbox. Enabling it writes Warpai's `warpai-wsl-com` bridge into that Agent's own WSL configuration.
3. Use **Rescan agents** after installing or updating a CLI.
4. Use **Remove Warpai MCP from all agents** to remove only the Warpai-owned entries.
5. Use **Uninstall WSL Companion** to remove the guest component.

The changes apply only to the selected WSL account. Start the Agent in a **new WSL terminal session** so it reloads its configuration.

Warpai preserves vendor authentication and unrelated MCP servers. The checkbox configures the collaboration bridge, not your third-party MCP catalog. Install third-party MCP servers inside WSL with the same commands and configuration files you would use on a normal Linux machine.

After upgrading to **1.7.8.2**, rescanning migrates saved install, lookup, and cleanup commands to `warpai-wsl-com`. Inactive cleanup records can be selected again to reinstall; existing user MCP entries and activation choices are preserved. Start a new Agent session afterward to reload the configuration.

---

<a id="two-tier-mcp-architecture-and-environment-isolation"></a>
## 6. Two-Tier MCP Architecture and Environment Isolation

<div class="docs-callout important">
  <div class="docs-callout-title">MCP Belongs to the Agent's Environment</div>
  <p>MCP configuration, installation records, and running processes are bound to the machine, user, and environment where the Agent actually executes. WSL is a terminal transport layer; it does <strong>not</strong> carry the Windows host's MCP servers into the Linux guest.</p>
</div>

### The two tiers

| Tier | Configuration source | Execution location | Installation and persistence |
| :--- | :--- | :--- | :--- |
| **Warpai native MCP (desktop tier)** | The machine running Warpai. Reads `~/.warp/.mcp.json`, the client's global Agent home configs, and local project-level `.mcp.json` files. | A local Warpai subprocess, spawned by the desktop client. | Stored in the local SQLite database. Servers installed through the Warpai UI/Gallery stay local, are not written back to `.mcp.json`, and do not sync across devices. |
| **CLI Agent native MCP (environment tier)** | The configuration inside the Agent's own environment. Claude Code, Codex, Qoder, and other CLIs read their own user-level and project-level files. | Inside the environment where the Agent runs. The Agent reads config and spawns MCP server subprocesses there. | Independent per environment. The Windows host, each WSL distribution, and each SSH account are isolated. |

Typical native configuration locations:

| Agent | User-level | Project-level |
| :--- | :--- | :--- |
| Warpai | `~/.warp/.mcp.json` | `.warp/.mcp.json` |
| Claude Code | `~/.claude.json` | `.mcp.json` |
| Codex | `~/.codex/config.toml` | `.codex/config.toml` |
| Other Agents | `~/.agents/.mcp.json` | `.agents/.mcp.json` |

A CLI running inside WSL is a Linux process. It reads Linux paths such as `/home/<user>/.claude.json`, not `C:\Users\<user>\.claude.json`.

### Why there is no automatic bridge

1. **File-based MCP scans local paths.** Warpai searches project-level configuration by walking up from the current session's working directory on the host filesystem. User-level configuration also lives in the local home directory. The desktop process does not mount or traverse WSL virtual disks or remote SSH filesystems to discover configuration.
2. **MCP servers are child processes.** A host client spawns its MCP servers locally. There is no general MCP stdio tunnel across an SSH or WSL boundary.
3. **Gallery installs are local records.** An MCP installed through the Warpai Gallery is persisted in that desktop installation's SQLite database. Another computer, a WSL distribution, and an SSH host do not inherit it.
4. **The collaboration bridge is environment-specific.** The per-Agent checkbox writes only the `warpai-wsl-com` bridge for the selected WSL account. It does not copy or synchronize your other MCP servers.

### Common questions

**Why does a CLI Agent inside WSL report no MCP tools even though Warpai on Windows has MCP servers installed?**

Because the WSL Agent reads the Linux user's configuration. Install the MCP server's runtime and configuration inside WSL. The Windows Warpai MCP installation is a different tier.

**Does an MCP server configured in WSL appear in Warpai's own Agent panel?**

Only if the Agent runs in that WSL environment and reads its WSL configuration. Warpai's own desktop MCP list still comes from local files and its local database.

**Can I copy `~/.warp/.mcp.json` into WSL?**

You can copy a configuration file, but the server command must also exist and be executable inside WSL. Paths, runtimes, credentials, and environment variables are environment-specific. Installing and configuring the server in WSL is the supported path.

**Does the Agent Communication bridge carry my MCP servers to the remote side?**

No. It provisions Warpai's collaboration tools in that environment. It does not carry third-party MCP servers.

---

## 7. Multi-Agent Collaboration Workflow

After `cd` into the WSL project, Warpai's collaboration panel binds to that project. Both modes are available:

- **Project Mode:** Agents communicate as peers, exchange structured questions, publish shared context, and claim tasks from a shared pool. No Git repository is required.
- **Worktree Mode:** The Coordinator creates physical Git worktrees inside the WSL repository, assigns isolated workers, and requests peer review. Accepted changes are integrated by the Coordinator.

### Task lifecycle and subtask autonomy

1. The Coordinator can dispatch tasks at any time. New tasks enter the recipient's durable queue.
2. A worker executes one task at a time and starts the next queued task after submission.
3. Dispatched tasks join the supervision list and retire after review or a resolved terminal outcome.
4. **Subtask autonomy:**
   <div class="docs-callout important">
     <div class="docs-callout-title">Subtask Autonomy and Ready State</div>
     <p>An Agent's internal subtask states are not monitored by the external supervision list, because the Agent itself is in a ready state that can receive tasks. Micro-steps an Agent decomposes internally remain internal; the coordination fabric tracks cross-Agent task lifecycles.</p>
   </div>

---

## 8. Local-Parity Development Experience

With WSL configured as above, Warpai provides the same core experience as a local project:

- **GPU-accelerated terminal:** Fast rendering, command blocks with status and duration, smart completion, and keep-awake controls.
- **Project Explorer:** Browse, create, rename, and delete files inside WSL through bounded stdio file chunks.
- **Editor and previews:** Edit WSL files in the built-in editor and preview Markdown with diagrams and math.
- **Visual Git Review:** Inspect WSL working-tree changes in the graphical diff view.
- **Guarded PTY delivery:** The supervisor observes the VT100 stream and delivers queued instructions only when the Agent is idle with no pending draft.

---

## 9. Troubleshooting and Security

### Common failures

| Symptom | Check |
| :--- | :--- |
| `wsl --install` fails or WSL will not start | Virtualization is enabled in BIOS/UEFI; the Windows features are enabled; reboot after enabling them. |
| The distribution is not listed in Warpai's WSL account dropdown | Open a WSL terminal in Warpai first; confirm the distribution and user with `wsl --list --verbose`. |
| `wsl` opens but command blocks or `cd` tracking do not activate | Run `wsl --shutdown`, reopen the session, and confirm you are on Warpai 1.7.8.2. |
| WSL Companion install fails | The guest must be x86_64; check free disk space and the selected distribution/user. Rescan before retrying. |
| WSL Companion is missing after install | Verify `~/.config/warpai/wsl/bin/warpai-wsl-companion --check-runtime` in the same Linux account. |
| Agent checkbox is disabled | Companion is not installed or is incompatible, the CLI was not discovered, or the account is not selected. |
| MCP is still absent in the CLI | Start the Agent in a new WSL session; confirm its Linux user-level config file contains the Warpai bridge. |
| Git or builds are slow | Move the repository from `/mnt/c/...` to the Linux filesystem under `/home/<user>/...`. |

### Security baseline

- Treat the WSL Linux user as a separate account with its own credentials and MCP configuration.
- Keep third-party MCP servers installed inside WSL if the Agent runs there; do not rely on Windows paths or Windows credentials.
- Review MCP server commands before enabling them. The Warpai checkbox only manages its own bridge entry.
- Do not store credentials in project-level MCP files that may be committed to Git.

### Verification checklist

- [ ] `wsl --status` reports WSL 2 as the default.
- [ ] `wsl --list --verbose` shows the distribution on version 2.
- [ ] Warpai opens the WSL shell and tracks `cd`.
- [ ] **Settings > Features > Agent communication > WSL communication** is on and the WSL account is selected.
- [ ] **Install WSL Companion** is checked and `warpai-wsl-companion --check-runtime` succeeds.
- [ ] Required Agent checkboxes are enabled and the Agent was restarted in a new session.
- [ ] The collaboration panel binds to the WSL project and remote tools work as expected.
