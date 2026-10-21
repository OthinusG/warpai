# Project Memory

Updated 2026-10-21. Archives: docs/history/MEMORY-2026-10-{13,19-SSH-MCP}.md.

## Working and delivery rules

- Rust builds/native acceptance: GitHub with caches. Local syntax/scripts only; no personal install/config changes.
- Windows fixtures must retain the real disposable KnownFolder profile; spoofing USERPROFILE can make LocalAppData unavailable.
- Peer wake cancellation fixtures must end the run on the captured paste event;
  scheduling all agents before cancellation can exhaust Claude's Enter delay.
  CI repair and receipts: specs/agent-communication-v2/CI-2026-10-21.md.
- Oct19 cloud acceptance passed: communication 39259499684, Companion 39254214995,
  Linux packages 39249589503. The original v1.7.6 draft/tag remains immutable.
- Dependencies: docs/FEATURE-DEPENDENCIES.md; desktop Git stays unbundled.
- Preserve assets/receipts and gh-pages; remove only authorized, merged, clean,
  unused branches/worktrees. Linux packaging: specs/linux-desktop/TECH.md.
  Debian bullseye Git 2.30.2 requires `git config --global --add safe.directory`
  with the canonical `/__w/...` checkout root; `-c safe.directory` is ignored and
  runner workspace paths resolve elsewhere.
- Release immutable tags through existing workflows; desktops share tagged source.
  Follow [release gates](specs/RELEASE.md), including five existing installers and
  checksums. Finish UI acceptance before screenshots/packages. SSH major 1; Agent Bus 2.

## Published releases and maintenance

- Oct19: preserve unpublished 1.7.2 and immutable v1.7.6 tag/draft. Oct20 owner
  authorized stopping earlier jobs, fixing Zsh bootstrap echo on main, validating,
  then publishing the combined changes as v1.7.7. Poll runs with `gh run view`
  every 30 minutes maximum. See specs/RELEASE.md.
- Owner Mac-to-Linux panels passed; 5.4.0 lacked MCP setup; supplied 5.5.0.
  Latest Windows failure was ProxyJump before capture, not a proven GUI timeout.
- Windows SSH defaults to PowerShell, never CMD. Select remote grammar before bootstrap;
  remote OS owns path/completion semantics. Audit: SSH-POWERSHELL-AUDIT.md.
- SSH tabs were correct while panel/MCP stayed Starting. Share WSL observation
  and guarded wake delivery with SSH; preserve native tab evidence and drafts.
- Public: 1.7.1 / Companion 5.4.0; README links must match published assets.
  Linux x64/ARM64: AppImage/DEB/RPM/Arch, glibc >= 2.31.
- Artifact-only macOS runs must not trigger Windows publication.
- Windows CI/release uses pinned official protoc 36.2 + SHA-256; Chocolatey NuGet
  protoc returned 503/504 and blocked Windows jobs. See specs/RELEASE.md.
- Zsh bootstrap PTYs start raw with ZLE off, then restore sane mode after the
  injected heredoc is read; remote Zsh follows the same sequence.

## Project, worktree and Agent ownership

- Dynamic supervision: dispatch anytime; track task/request IDs, retire handled
  outcomes, retain unresolved work across waits/yields. No batch/exclusive watch.
- Warp's native worktree system owns creation, catalog and tabs; creation opens a
  new tab without starting an Agent. No separate worktree registry/session manager.
- Canonical Git common-directory identity joins registered checkouts/subdirectories,
  including empty tabs/worktrees outside main. Ancestry, basename, branch and
  remote URL cannot establish membership.
- Reusing a terminal after changing checkout must scope identity lookup to the
  current workspace. Preserve the previous checkout's tasks and history; never
  move its durable identity or reuse its Coordinator authority.
- Non-Git folders are independent canonical-root projects with Worktree unavailable.
  They neither inherit nor clear a Git project's Coordinator. Preserve separate
  clones and SSH host/account/service authorities.
- Project lists online peers across checkouts before pagination. Worktree allows
  same-checkout messages; cross-checkout traffic uses the Coordinator. File scope
  stays checkout-local; panel reads never change admission/permissions.
- Project owns mode. Eligible online Agents may replace the Coordinator; demote
  the predecessor without stopping it or changing task ownership.
- Coordinator belongs to the project and exact native run, surviving pane/tab/
  project/focus changes; only process termination/revocation or app shutdown clears
  it. A same-name restart cannot inherit authority. Handoffs remain bounded and
  include unfinished-task counts plus paginated instructions.
- Remove local worktrees via native Git: confirm/recheck exact registered path;
  protect main/open/locked/dirty/untracked/ignored checkouts; never force/delete
  branches. SSH removal is outside this patch.
- Sidebar owns sessions; panel owns messages/tasks/reviews/history. Native widgets,
  8px spacing, default closed; preserve focus/drafts.
- Derive coverage/icons from CLIAgent; preserve aliases/custom native MCP sessions.
  No duplicate vendor allowlist or shell-tool fallback.

## Usage accounts and native Orca port

- Native Orca port: https://github.com/stablyai/orca at
  3e0b82835856dde57f43a47661cf736e3e3dd90d; preserve MIT notices. Provider contracts:
  specs/agent-communication-v2/USAGE-PROVIDERS.md; native UI/TLS, no new runtime.
- Accounts and OS-secure credentials persist application-wide; never put credentials
  in metadata/IPC/logs. Cache across focus changes; requery after restart. Vendor CLI
  owns login; synthetic fixtures cannot prove auth and personal credentials are off-limits.
- Preserve fixed HTTPS hosts, sensitive headers, bounded IO and generations. agy >=1.1.11
  uses verified free `/usage` flags. Qoder/QoderCN may use local JSONL `get_usage_info`,
  never a prompt/model turn or SDK. Poll once/account/five minutes; manual refresh bypasses.
- Unknown buckets stay unavailable; Zen USD remains a balance. Usage UI keeps a fixed
  header and independent scroll. CLI children prepend their binary path and hide Windows
  consoles; preserve PTY/detached services. Revalidate provider protocol changes.

## SSH, task integrity and files

- SSH masters need no retained launch arguments; independent native transport does.
  OpenSSH owns ~/.ssh expansion. SSH Remote defaults off and gates every Companion API; OpenSSH stays independent; native MCP needs Companion 5.5.0. Contract/receipts: specs/agent-communication-v2/SSH-MCP-SETTINGS.md and INSTALLATION.md.
- WSL setup/recovery run on spawn_blocking; owner v4 proved installed-Agent
  discovery's nested-runtime panic. Receipts/audit: FIXES-1.6.4.md. Empty scans
  cannot verify installed-Agent discovery.
- Diagnose actual failed-stage exit/stderr, not failed diagnostic launchers.
  script/diagnose-wsl-companion.bat + .ps1 confirms account/HOME and preserves
  nonsecret stderr; redact configuration JSON/credentials.
- Language restart: cancelable; Unix waits for exit; AppImage reopens its original
  file. Windows isolates jobs.
- WSL contract: specs/wsl-1.6.0/TECH.md. Use the Windows-bundled guest binary,
  confirmed WSL 2 distro/user and native integration; preserve opt-outs and guards.
  Account detection needs no Companion; launch text cannot grant account authority.
  Failed connections await explicit retry/context change; recovery uses the trusted
  bundle. Explorer respects the WSL switch; mutations release clients; uncertain
  installs remain removable. Use sh -c and hidden background consoles.
  Native capture/rlto/debug-cfg gates: CI-DIAGNOSIS.md. No host MCP injection.
- Active scope: specs/agent-communication-v2/{PLAN,API,WORKTREE-COLLABORATION}.md.
  Archived enrollment is historical; cross-host federation is outside scope and
  must not appear as missing functionality in product copy.
- Native SSH/cd selects the confirmed remote project; opening files must not
  change project context. Pending/unconfirmed SSH never falls back to a local root.
  Same selected remote project shares one account-owned Companion Broker/Store.
- Reuse Explorer/Code View/previews/original Review. Companion carries private Git
  and runtime dependencies; vendor CLI/auth remain user-owned. Keep on-demand
  startup/60-second idle shutdown; no tray or desktop auto-deployment requirement.
- Fence SFTP/control by confirmed host/account/root/session/generation. Reject
  traversal/aliases; keep atomic save, no-replace rename, bounded staging/deletion
  and stale-save checks. Never repeat uncertain writes.
- Keep original request UUID/content, task revision/run identity and receipts.
  PTY submission is not Agent acknowledgement or execution. Disconnect/view loss
  does not prove a process stopped; unknown attempts block new ownership until
  explicit guarded reconciliation/override. Preserve unresolved history/export.
- Audit: MACOS-FIXES-2026-10-12.md. Local/SSH/WSL share results/questions and
  focused-terminal enablement (also command-entered sessions). Project peers
  supervise; Worktree allocation is Coordinator-only. Final replies atomically
  queue results/consume requests; blocked stays unresolved. Persist exact-attempt
  permission/cancel/disconnect evidence, never drafts/output. Wait receipts retain
  unacknowledged messages; notifications prove neither execution nor handling.
- Keep drafts in memory, MCP idle evidence and truthful sidebar lifecycle projections.
  Native draft clearing was authorized Oct13; receipts: FIXES-1.6.4.md.
- Replacement repositories need fresh admission; isolate old history/capabilities.
  Same-repository checkout replacement requires rejoin; receipt: FIXES-1.6.4.md.
- Codex opaque OSC 9, warnings and silence cannot prove completion. Native
  observations cover all managed Agents; vendor runtime acceptance stays pending.
- WSL setup hardens owned real namespaces to 0700; preserve SSH/account data.
  Restore saved live accounts with one deferred rescan, never a failure retry loop.
- Input/cancel/run replacement fences Enter; closing invalidates its run.
- Back up schema migrations consistently; no downgrade or automatic legacy
  identity migration. Preserve read-only legacy metadata, uncertainty and receipts.
  macOS sockets require short private /tmp paths; Windows uses bounded asynchronous
  native named pipes, with a new listener before handing off a connection.

## Product defaults, CLI integration and website

- Keep WARP_* compatibility and ~/.config/warpai recovery sources/profiles/values.
  English/zh-Hans/zh-Hant need restart; localize outcomes, preserve IDs.
- Native capture: each locale owns/closes an SSH master; close dropdowns; preserve
  UTF-8/POSIX tooling and PROTOC. Test real Windows glyph fallback/font rendering.
- Preserve saved themes, Claude Warm Light/Latte/Dark and approved SVG branding.
  Keep-awake tracks activity; no cloud AI/telemetry.
- Local Codex uses per-session MCP/--no-daemon while enabled, preserving CLI/args/cwd/
  PATH/package ownership. No proxy, impersonation or persistent config writes;
  only explicit reserved legacy-MCP cleanup may edit persistent configuration.
  Preserve unrelated servers, report failures, and test Windows Unicode argv/TOML.
- WSL stays native without host MCP injection; preserve listeners/drafts, run,
  distro/user identity and Windows-only UI. Verified observations precede wakes.
- About Update checks stable releases/exact platform installers. Startup check
  defaults off/runs once; no installation writes. Docs retain screenshot provenance.
- https://othinusg.github.io/warpai/ lives on gh-pages (EN /, ZH /zh/); preserve
  branding/worktree, cmap-check font subsets, and Jekyll /warpai production baseurl.
- Feature catalog: docs/FEATURES.md.

## Retained native lifecycle constraints

- Keep Herdr Idle/blocked semantics, run/input/redraw fences and vt100 capture.
- SSH/WSL installers use `.config/warpai`; setup locks explicitly unlock despite inherited descriptors.
- Signed birth-time hashes preserve positive identities; absent birth time cannot prove inode reuse.
- Focus publishes session before roots; replay cached roots and scope remote polls to admission.
- Bindings register before MCP discovery; readiness stays separate. Focus never acknowledges work.
- Preserve context/version/digest/public guards and task-anchored native threads/reviewed memory.
- Remote bootstrap input must be raw: zsh/bash init subshells set `stty raw` and the zsh wrapper
  restores sane before eval. The embedded PowerShell SSH seed is minified before UTF-16/base64
  expansion; 1.7.6's unminified ~6.2 KB line exceeded MAX_CANON and hung Windows-to-zsh SSH
  warpify at "Starting shell...".
- Remote-peer polls leave the WSL/SSH configuration generation untouched without a target;
  syncing it to the SSH generation bumped the epoch and aborted WSL auto-warpify (1.7.7; both fixes shipped in 1.7.8).

## Memory maintenance

Keep under 200 lines / 12 KB; receipts go in specs. Preserve evidence;
replace superseded facts.
