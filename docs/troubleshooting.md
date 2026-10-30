---
layout: docs
title: Troubleshooting & FAQ
description: "Diagnose and resolve common issues with terminal rendering, agent detection, PTY input, Gatekeeper, SmartScreen, and SSH Remote companion."
section: Configuration & Help
doc_id: troubleshooting
prev_url: /docs/config.html
prev_title: Configuration & Appearance
---

This guide covers common questions and resolution steps for desktop setup, agent communication, remote SSH, and quota tracking.

---

## Installation & First Launch

### macOS: "Warpai cannot be opened because the developer cannot be verified"
- **Cause**: Warpai is distributed as an open-source, ad-hoc signed `.dmg` rather than an Apple-notarized commercial app.
- **Solution 1 (GUI)**: Open **System Settings > Privacy & Security**, scroll down to Security, and click **Open Anyway**.
- **Solution 2 (Terminal)**: Remove the macOS quarantine attribute:
  ```bash
  xattr -d com.apple.quarantine /Applications/Warpai.app
  ```

### Windows: "Windows protected your PC" (SmartScreen)
- **Cause**: Newly published open-source releases accumulate reputation over time in Microsoft's Defender database.
- **Solution**: Click **More info**, then click **Run anyway**.

---

## Agent Detection & Collaboration

### My CLI Agent is running in the terminal, but it does not appear in the left panel.
1. **Verify Agent Communication is ON**: Open **Settings > Features > Agent communication** and ensure the master switch is enabled.
2. **Confirm Agent Checkbox**: Make sure your specific agent CLI (e.g. `Claude Code`, `Codex`, `Gemini`) is checked in the list.
3. **Restart the Terminal Session**: If you enabled agent communication *after* opening your tab, open a fresh terminal tab (`Cmd + T` / `Ctrl + T`) and launch the agent again. Environment variables (`WARP_AGENT_ENDPOINT`) are injected at pane creation time.
4. **Custom CLI Names**: If you renamed or aliased your agent executable, register it as a **Custom Agent** in Settings.

### The Agent finished its work, but pending tasks are not executing automatically.
Warpai uses **Guarded PTY Delivery** to protect your workflow:
1. **Unsent Draft in Terminal**: If you typed text into the terminal prompt without pressing Enter, Warpai halts automatic task delivery so it doesn't overwrite your typing. Clear the prompt to resume.
2. **Approval Prompt (`WaitingApproval`)**: If your agent is paused waiting for you to authorize a tool call (e.g. running `rm` or applying a diff), Warpai will not press Enter for you. Provide your answer directly in the terminal.
3. **Screen Quiet Window**: Warpai waits approximately 300ms for terminal output to settle before declaring an agent `Idle`.

### Codex fails to register or conflicts with an existing daemon.
- Warpai automatically passes `--no-daemon` and per-invocation `-c` configuration parameters to Codex. Ensure you are launching Codex directly inside the Warpai terminal without wrapping it in conflicting daemon management scripts.

---

## Remote SSH & WSL 2

### When I SSH into a server, Project Explorer does not show remote files.
1. **Change Directory (`cd`)**: Remote attachment is triggered dynamically when you navigate into a project directory on the remote machine (e.g., `cd /var/www/my-app`).
2. **Check Remote Permissions**: Warpai stores the companion binary in `~/.config/warpai/` under your remote user account. Ensure your remote home directory has appropriate permissions (mode `0700` or `0755`).
3. **SFTP Support**: Ensure OpenSSH on the remote server allows SFTP subsystem access (standard on almost all distributions).

### Windows: WSL 2 Companion fails to connect.
1. **Check WSL Version**: Warpai requires **WSL 2**. Run `wsl -l -v` in PowerShell to verify your distribution is running on version 2, not version 1.
2. **WSL Feature Switch**: Verify **Settings > Features > WSL Integration** is enabled.

---

## Git & Worktree Issues

### Error: "System Git executable not found on PATH"
- **Cause**: While standard terminal usage and remote Companion sessions have zero Git dependencies, **local Worktree Mode** and **local Git Review** require Git on your host computer to inspect repository metadata.
- **Solution**: Install Git on your computer (e.g., via `brew install git` on macOS or `winget install Git.Git` on Windows) and restart Warpai.

### A worktree checkout shows as locked or in conflict.
- Warpai delegates directly to Git's native worktree engine. If a worktree directory was deleted manually from disk, run:
  ```bash
  git worktree prune
  ```

---

## Data Usage & Quotas

### Why does an account show "Unavailable" or no percentage?
- **Honest Metrics Principle**: Warpai strictly adheres to truthful data reporting. If an API returns missing quota data, Warpai displays `"Unavailable"` rather than fabricating a false "0%" or "100%".
- **CLI Authentication**: Ensure you are logged into your vendor CLI on your desktop (e.g. run `claude` or `codex` interactively once to complete OAuth sign-in).
- **Google Antigravity**: Requires `agy` version `>= 1.1.11` to parse the free `/usage` command.
- **Proxy Configuration**: Usage requests support macOS automatic proxy configuration (PAC). Check that the configured proxy can reach your provider's usage service.
- **Qoder / QoderCN**: Keep the vendor CLI installed and authenticated. Usage is read through its local daemon protocol, with a fallback for older CLI versions.
