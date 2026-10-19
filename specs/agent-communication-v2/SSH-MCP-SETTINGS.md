# SSH Remote communication settings

## Product contract

Owner authorized on 2026-10-19: add an SSH Remote master switch and remote Agent
MCP setup controls analogous to the existing local and WSL settings. Include this
work with the two SSH connection repairs and the unpublished Qoder changes in
1.7.3. Do not publish the connection-only test source as the completed release.

Settings > Features > Agent communication contains a separate SSH Remote section:

- Master switch, initially off, persisted independently from local/WSL settings.
- Dropdown of confirmed, currently logged-in SSH accounts. No manual credentials,
  alias/path forms, automatic Companion deployment or FTP configuration.
- Remote Agent discovery, per-Agent MCP checkboxes, rescan and remove-all actions.
- Explicit busy, disconnected, unavailable/old Companion, no-Agent, partial error,
  enabled and disabled states. Offline controls cannot mutate a saved local target.

The master switch gates SSH Remote collaboration, MCP, file exploration and Review,
matching the existing WSL master switch. Ordinary SSH terminal IO remains native.
Enabling MCP configuration requires an explicitly selected live
SSH account; a scan never silently enables a vendor. Disabling revokes remote MCP
participation and removes only owned Warpai entries. Preserve existing vendor
authentication, unrelated MCP servers and user-modified/colliding entries.
Other host/account, WSL and local opt-ins remain separate. Starting an ordinary
selected CLI in the remote terminal loads its native MCP integration; no long
manual Companion launch command is required.

## Visual source and states

The existing WSL section in `app/src/settings_view/agent_communication.rs` is the
authoritative native component reference: existing switches, account dropdown,
Agent icons/checkboxes, secondary buttons, spacing and status text. Use existing
theme tokens and English/zh-Hans/zh-Hant localization. No new artwork or custom
button theme. Capture fixed representative states before binding live mutations;
review enabled/disabled/empty/busy/error/partial-error states and long labels.

## Architecture and API

1. Reuse confirmed SSH session/profile/transport selection, including literal
   `~/.ssh/` control sockets, native Windows arguments and SSH inside WSL. Account
   operations use the same verified private Companion connection/fence; no host
   credentials are persisted and no remote path is resolved on the desktop.
2. Reuse native vendor discovery/configuration adapters and the WSL setup request
   semantics: enable, select an Agent, rescan and remove-all. Keep the WSL public
   behavior and existing tests intact. Account-owned setup preferences live in
   the existing private SSH Companion namespace, separate from WSL namespaces.
3. Add a typed, bounded setup command to the existing private project control
   protocol, gated by an advertised native MCP setup capability. Return only
   nonsecret discovery/preferences/status metadata. Configuration and CLI probes
   run on a blocking worker, never inside the async runtime thread.
4. Configure the remote Companion's native MCP entry with its Agent program.
   Admission proves the same-account native process ancestry, exact process birth
   identity and project directory using native platform metadata. Reuse Linux WSL
   binding semantics; implement macOS/Windows proofs with existing native APIs and
   the already installed sysinfo dependency. Do not infer authority from names,
   command output or a same-name process restart. Unsupported proofs fail closed.
5. Opt-ins and pending work are remote-owned. A tab/root/account switch fences UI
   replies; a closed SSH master never reconnects to another host. Cross-account
   preferences cannot share selected Agent state. No capabilities enter config,
   JSON responses or logs; bindings are ephemeral and expire with their native run.
6. Codex uses the remote installed CLI's verified per-invocation mode, preserving
   arguments and shell expansion. Administrative/noninteractive commands remain
   native. Preserve unrelated remote configuration and the existing WSL path.

New native MCP behavior requires Companion 5.5.0, retaining SSH protocol major 1
and Agent Bus protocol major 2. Old companions show an update requirement instead
of a successful checkbox. Desktop targets: macOS, Windows, Linux. Remote targets:
Linux, macOS, Windows. Remote installers remain explicit user actions.

## Tasks and verification

- [ ] Account-owned setup command and adapter reuse, with bounded input/output,
  version/capability checks, lock safety and collision-preserving removal tests.
- [ ] Native MCP binding on the three remote platforms with process identity,
  ancestry/account/root isolation, lifecycle and restart regression coverage.
- [ ] Desktop SSH settings state/target selection and async generation fences.
- [ ] Native settings controls and fixed-state screenshot QA in all three locales.
- [ ] Ordinary remote Agent MCP initialize/tools/list and same-project task/message
  exchange using deterministic, installed Agent fixtures; no paid model requests.
- [ ] Disable/remove, disconnect/reconnect, multiple accounts, SSH-in-WSL, missing
  CLI/Companion and incompatible version failure cases.
- [ ] Preserve existing local and WSL tests; default/warp_platform builds and
  artifact-only packages on three desktops, Linux x64/ARM64, and all Companions.
- [ ] Owner Mac-to-Linux settings/checkbox/ordinary-CLI acceptance; formal tagged
  packaging only after acceptance. Verify all platform assets before publication,
  then update bilingual download surfaces, Pages and Homebrew as specified.

## Current checkpoint

Connection-only source 73f4fae3 is in cloud validation. The owner confirmed both
panels recover in the previous test app with an absolute SSH_SOCKET_DIR, proving
the home-relative socket issue. That source has no SSH MCP settings UI. The
existing explicit `companion agent` path remains available, but does not satisfy
the newly authorized settings-based workflow.
