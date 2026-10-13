# Project Memory

Updated 2026-10-13. Read AGENTS.md and README.md.
Earlier context: [archive](docs/history/MEMORY-2026-10-13.md).

## Working and delivery rules

- Rust/native acceptance runs on GitHub. Batch 1–17 before full acceptance.
  Protocol/desktop jobs are independent; reuse build caches.
  Local Rustfmt parsing, diff and script/document checks are allowed.
  Do not install on personal machines or change personal Agent/SSH configuration.
- Check cloud runs every 30 minutes, without frequent polling or empty updates.
  A user-reported error permits immediate diagnosis. Use focused retries while
  debugging, then one complete source-matched acceptance run before delivery.
- Native UI gates release. WSL polling must not re-read its model during update.
- Public dependency notes: docs/FEATURE-DEPENDENCIES.md; desktop Git is not bundled.
- Preserve assets/receipts and gh-pages; remove only authorized, merged, clean,
  unused branches/worktrees. Retain four workflows.
- Release immutable tags through existing workflows; both desktops build the same
  source. Follow [release gates](specs/RELEASE.md), including five installers and
  checksums. Finish UI acceptance before screenshots/packages. SSH major 1; Agent Bus 2.

## Published releases and maintenance

- 1.6.4 / Companion 5.1.0 publication authorized 2026-10-13; immutable tags
  and delivery gates: specs/RELEASE.md. Active branch: milestone/1.6.4.
- Previous release receipts: specs/RELEASE.md.

## Project, worktree and Agent ownership

- Warp's native worktree system owns creation, catalog and tabs; creation opens a
  new tab without starting an Agent. No separate worktree registry/session manager.
- Canonical Git common-directory identity joins registered checkouts/subdirectories,
  including empty tabs and worktrees outside main. Ancestry, basename, branch and
  remote URL cannot establish membership.
- Reusing a terminal after changing checkout must scope identity lookup to the
  current workspace. Preserve the previous checkout's tasks and history; never
  move its durable identity or reuse its Coordinator authority.
- Non-Git folders are independent canonical-root projects with Worktree unavailable.
  They neither inherit nor clear a Git project's Coordinator. Preserve separate
  clones and SSH host/account/service authorities.
- Project mode lists online project participants across checkouts, filtering before
  pagination. Worktree mode permits same-checkout direct messages; cross-checkout
  communication goes through the selected Coordinator. File authority stays local
  to each checkout. Panel reads must not change admission/permissions.
- Mode is project-owned. Any eligible online Agent may replace the Coordinator;
  demote its predecessor without stopping its process or changing task ownership.
- Coordinator belongs to the project and exact native run, surviving pane/tab/
  project/focus changes; only process termination/revocation or app shutdown clears
  it. A same-name restart cannot inherit authority. Handoffs remain bounded and
  include unfinished-task counts plus paginated instructions.
- Remove local worktrees via native Git: confirm/recheck exact registered path;
  protect main/open/locked/dirty/untracked/ignored checkouts; never force/delete
  branches. SSH removal is outside this patch.
- Sidebar owns sessions; panel owns project messages/tasks/reviews/history.
  Reuse native widgets/8px spacing. Default closed; preserve focus/drafts.
- Derive Agent coverage/icons from managed CLIAgent, preserving aliases and custom
  native-MCP-capable sessions. No duplicate vendor allowlist or shell-tool fallback.

## Usage accounts and native Orca port

- Native Orca port: https://github.com/stablyai/orca at
  3e0b82835856dde57f43a47661cf736e3e3dd90d; preserve MIT notices in both bundles.
  Provider/auth contracts: specs/agent-communication-v2/USAGE-PROVIDERS.md.
  Use native TLS/proxy/storage/widgets; no Electron/Node runtime or invented quotas.
- Accounts/visibility and OS-secure credentials persist application-wide; never
  put credentials in metadata/IPC/logs. Cache readings across project/focus changes;
  restart restores accounts and queries fresh readings.
- Vendor CLI login owns OAuth refresh/sign-in. Synthetic fixtures cannot prove
  authenticated availability; never use personal credentials for tests.
- Usage footer: fixed header, independent scroll, four visible rows, native icons
  and quota bars. Unknown limits remain unavailable; Zen USD balance stays a balance.
- Preserve fixed HTTPS hosts, sensitive headers, bounded IO, generations and all
  twelve provider HTTP contracts in USAGE-PROVIDERS.md.
- agy >=1.1.11 uses only verified free /usage flags via its resolved executable;
  tolerate log noise, stop polling after model-turn detection, preserve that guard
  and hidden caches across refreshes; update/restart restores eligibility.
- Background CLI children prepend resolved binary directory to PATH and hide
  Windows consoles. Test window visibility, not code pages; preserve PTY/detached
  services. Qoder CN has no safe usage adapter; never send a billed polling prompt.

## SSH, task integrity and files

- WSL setup/recovery run on spawn_blocking; owner v4 proved installed-Agent
  discovery's nested-runtime panic. Receipts/audit: FIXES-1.6.4.md. Empty scans
  cannot verify installed-Agent discovery.
- Diagnose actual failed-stage exit/stderr, not failed diagnostic launchers.
  script/diagnose-wsl-companion.bat + .ps1 confirms account/HOME and preserves
  nonsecret stderr; redact configuration JSON/credentials.
- Language restart is cancelable ordinary relaunch: macOS waits for old PID and
  passes paths positionally; Windows preserves parent-job isolation.
- WSL contract: specs/wsl-1.6.0/TECH.md. Use the Windows-bundled guest binary,
  confirmed WSL 2 distro/user and native integration; preserve opt-outs and guards.
  Account detection needs no Companion; launch text cannot grant account authority.
  Failed connections await explicit retry/context change; recovery uses the trusted
  bundle. Explorer respects the WSL switch; mutations release clients; uncertain
  installs remain removable. Use sh -c and hidden background consoles.
  Native capture: 60s steps/420s process; clear static polling fence (CI-DIAGNOSIS.md).
  Check Windows rlto/release_bundle and matching debug cfg. Accept package 3.x
  previews; no wrapper or Windows-host MCP injection into the guest.
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
  focused-terminal enablement, including command-entered sessions. Project peers
  supervise retries/reassignment/review; Worktree allocation is Coordinator-only.
  Final replies atomically queue results/consume requests; blocked stays unresolved.
  Persist permission/cancel/disconnect evidence with exact attempts, never drafts
  or output. Wait receipts retain unacknowledged messages; notifications prove
  neither execution nor result handling.
- 1.6.4 fixes: specs/agent-communication-v2/FIXES-1.6.4.md. Usage selects
  five-hour OR weekly windows; Antigravity retains Gemini and Claude/GPT pools.
  Composer drafts/attachments stay in memory; owner authorized clearing native PTY
  drafts before peer submission on 2026-10-13. Startup retains MCP idle evidence;
  sidebar lifecycle projections never manufacture completion notifications.
- Same-path replacement repositories need fresh admission with old history/capabilities
  isolated; same-repository checkout replacement requires explicit rejoin. b16ec81e
  passed both desktop protocol suites (38390412725).
- Owner requires mature open-source lifecycle detection for all managed Agents.
  Codex opaque OSC 9 cannot prove completion; never infer idle from warnings or
  silence. Welcome-only startup and always-working Codex need native observation;
  vendor runtime acceptance remains pending.
- Windows test package 38368976431 passed. Automatic release 38378106900 failed
  before compilation at nonexistent v1.6.4; main 502b928a now skips test handoffs.
- WSL setup-first discovery created 0755 service namespaces, rejected by the 0700
  service guard. Harden only owned real directories, preserving SSH/account data.
  Restore saved live accounts with one deferred read-only rescan; don't retry-loop
  failures. Test installed workspace connection/reconnect and settings reload.
- Wake polls until idle; native input/cancel/run replacement fences delayed Enter.
  Cancel pauses wake until resumed input; closing a view invalidates its run.
  Evidence/test labels never authorize command execution or secret-file access.
- Back up schema migrations consistently; no downgrade or automatic legacy
  identity migration. Preserve read-only legacy metadata, uncertainty and receipts.
  macOS sockets require short private /tmp paths; Windows uses bounded asynchronous
  native named pipes, with a new listener before handing off a connection.

## Product defaults, CLI integration and website

- Keep WARP_* compatibility and ~/.config/warpai recovery sources/profiles/values.
  English/zh-Hans/zh-Hant need restart; localize UI roles/outcomes, preserve IDs.
  Windows glyph fallback precedes text caching; use regional Chinese fonts.
  Native capture drivers cannot use cfg(test); each locale owns/closes one SSH
  master. Close dropdowns; selectors/defaults share labels. UTF-8/POSIX tooling;
  export/verify PROTOC on Windows. Claude Warm Light/Latte/Dark are approved.
- Preserve saved themes; baseline Claude Warm Light. Session-only keep-awake tracks
  Agent activity, permitting display/user sleep. No bundled cloud AI/telemetry.
- Native Codex execution (including --yolo/resume) uses per-session MCP/--no-daemon
  while enabled; preserve original CLI/args/cwd/PATH/package ownership. No daemon
  proxy, impersonation or persistent Codex config edits. Only explicit reserved
  legacy-MCP cleanup may read/edit persistent Codex configuration.
- Legacy cleanup disables participation, preserves unrelated servers and reports
  failures. Ordinary disable/deselect only stops future session injection.
- Keep native Windows argv/TOML Unicode-before-shell-quoting tests. WSL stays native
  without host MCP injection; verified guest observations seed sessions before wake.
  Preserve per-run identity, listeners/drafts, distro/user keys and Windows-only UI.
- About Update checks stable releases; Download opens the exact platform installer
  or Releases fallback. Startup check defaults off/runs once; no installation writes.
- Bilingual Agent-first docs update existing READMEs; retain native screenshots,
  provenance and approved SVG branding.
- Website: https://othinusg.github.io/warpai/ on gh-pages, EN / and ZH /zh/.
  Preserve worktree/branding; update and cmap-check font subsets when copy changes.
  Jekyll pages own translated strings; production baseurl is /warpai.

## Memory maintenance

Keep under 200 lines / 12 KB; receipts belong in specs. Archive useful evidence,
not all startup context; replace superseded facts.
