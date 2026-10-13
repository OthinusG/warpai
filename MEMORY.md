# Project Memory

Updated 2026-10-13. Read with [AGENTS.md](AGENTS.md) and [README.md](README.md).
Historical receipts: [archive](docs/history/MEMORY-2026-10-10.md); consult as needed.

## Working and delivery rules

- Rust/native acceptance runs on GitHub. Batch 1–17 before full acceptance.
  Protocol/desktop jobs run independently; reuse Cargo caches and capture builds.
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

- Owner authorized 1.6.4 / Companion 5.1.0 packaging/publication on 2026-10-13.
  Preserve immutable historical tags; acceptance and delivery: specs/RELEASE.md.
- 1.6.3 / Companion 5.0.0: tag 67057c0a; full acceptance 38323386419 and
  38323390269 passed. Release packaging: 38333613421. See specs/RELEASE.md.
  Main integrated; maintenance branch milestone/1.6.3; merged milestone/1.6.2
  removed. Preserve terminal compatibility, pinned dependencies and repo.

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
- Mode belongs to the project independently of Coordinator. Any eligible online
  Agent can become Coordinator; switching demotes the previous one without ending
  its process or changing task ownership.
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

- Orca source: https://github.com/stablyai/orca at
  `3e0b82835856dde57f43a47661cf736e3e3dd90d`. Preserve MIT notices in both bundles.
  [Support and compatibility matrix](specs/agent-communication-v2/USAGE-PROVIDERS.md)
  owns provider/auth details; do not invent unsupported adapters or quota values.
- Port to native reqwest/TLS roots/system proxy, secure storage, CLI discovery and
  widgets; no Electron/Node/preload/React or temporary-clone runtime dependency.
- Accounts/visibility persist application-wide; manually connected credentials
  use OS secure storage, never account metadata/IPC/logs. Readings are cached in
  the application singleton. Project/worktree/tab/pane/focus changes do not reset
  them. Restart restores accounts and queries fresh readings.
- CLI login means desktop vendor login; expired OAuth requires vendor sign-in.
  Use synthetic fixtures, never personal credentials; they do not prove live
  authenticated vendor availability.
- Data usage is pinned outside main scrolling: fixed header, independent scroll,
  at most four visible account rows, native Agent icons and percentage bars.
  Unknown limits stay unavailable; verified Zen USD balance stays a balance.
- Preserve request parity, fixed HTTPS hosts, sensitive headers, bounded IO and
  generation guards. Tests cover twelve HTTP contracts and representative payloads.
- Antigravity uses the plain resolved agy executable, version >=1.1.11 and the
  exact free /usage metadata flags. Accept successful usage envelopes with log
  noise; stop subsequent polling if a model turn is detected. Other refreshes
  preserve that guard and hidden caches. Update agy/restart to restore eligibility.
- Background CLI processes prepend the resolved binary directory to PATH and
  suppress Windows consoles, including setup/MCP probes and Companion Git/Codex
  checks; test window visibility, not code pages. Preserve PTY/detached services.
  Qoder CN has no verified safe
  background usage adapter; do not submit a potentially billed model prompt.

## SSH, task integrity and files

- WSL 1.6.3 install failure confirmed by owner v4 diagnostics: installed binary,
  runtime and empty scan pass; installed-Agent scan exits 101 at setup.rs:136.
  WSL setup/recovery must run on spawn_blocking; empty scans miss this fault.
  SSH control/MCP/broker routes already use blocking workers; audit in FIXES-1.6.4.md.
- Language restart uses ordinary relaunch with cancellation, not updater installation.
  macOS waits for old PID exit and quotes paths through positional arguments;
  Windows reuses the command wrapper's parent-job isolation.
- Owner correction 2026-10-13: diagnose reported installation failures from
  actual failed-stage exit/stderr before production fixes; do not infer missing
  files or a runtime panic from an unsuccessful diagnostic launcher.
  Diagnostic launcher: script/diagnose-wsl-companion.bat + .ps1. Confirm direct
  Linux account/HOME first; redact credentials and setup configuration JSON,
  preserve ordinary stderr, and reproduce the actual application command.
- WSL uses its separate Windows-bundled guest binary and confirmed WSL 2 accounts;
  retain distro/user-scoped setup. Contract: specs/wsl-1.6.0/TECH.md.
  Plain interactive WSL launches default to native terminal integration, respecting
  opt-outs and input/block guards. Account detection needs no Companion; never
  infer account authority from a launch command alone. Background SSH/WSL/SFTP
  processes hide Windows consoles; failed WSL connections wait for explicit settings
  retry/context change. Progress exposes phases. Recovery uses the trusted bundle.
  Explorer uses confirmed remote selection and respects the WSL switch; settings
  Mutations release guest clients; failed/unknown installs remain removable. Use sh -c.
  Steps 60s/process 420s; clear static polling fence before live WSL (CI-DIAGNOSIS.md).
  Check Windows rlto/release_bundle; debug-only WSL helpers share callee cfg.
  Accept WSL package 3.x previews; no wrapper or Windows host MCP injection.
- Active scope: [PLAN](specs/agent-communication-v2/PLAN.md),
  [API](specs/agent-communication-v2/API.md),
  [Worktree contract](specs/agent-communication-v2/WORKTREE-COLLABORATION.md).
  Archived enrollment designs are historical. Cross-host task federation is not
  a goal/acceptance/follow-up item; do not call out its absence in product copy.
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
- Fix audit: MACOS-FIXES-2026-10-12.md (1–17 + extras).
  Local/SSH/WSL share results and durable questions. Project
  peers all observe and decide retry/reassignment/review; execution stays run-fenced.
  Local/SSH/WSL enablement follows the focused terminal independently, including
  command-entered sessions. Worktree allocation remains Coordinator-only. Final
  replies atomically queue results/consume requests; blocked reports stay unresolved.
  Permissions/cancellation/disconnect observations persist with exact attempts;
  never store drafts/output. Wait receipts retain unacknowledged messages;
  notifications never prove execution or result handling.
- 1.6.4 fixes: specs/agent-communication-v2/FIXES-1.6.4.md. Usage selects
  five-hour OR weekly windows; Antigravity retains Gemini and Claude/GPT pools.
  Composer drafts/attachments stay in memory; owner authorized clearing native PTY
  drafts before peer submission on 2026-10-13. Startup retains MCP idle evidence;
  sidebar lifecycle projections never manufacture completion notifications.
- Wake polls until idle; native input/cancel/run replacement fences delayed Enter.
  Cancel pauses wake until resumed input; closing a view invalidates its run.
  Evidence/test labels never authorize command execution or secret-file access.
- Back up schema migrations consistently; no downgrade or automatic legacy
  identity migration. Preserve read-only legacy metadata, uncertainty and receipts.
  macOS sockets require short private /tmp paths; Windows uses bounded asynchronous
  native named pipes, with a new listener before handing off a connection.

## Product defaults, CLI integration and website

- Keep WARP_* compatibility. Data/settings live in ~/.config/warpai; retain
  sources/profiles/new values.
  English/zh-Hans/zh-Hant UI; restart to apply language.
  Translate menus/Review/toasts, dynamic roles/participants/outcomes; keep IDs/content.
  Windows text shaping loads native glyph fallbacks before caching UI lines;
  Simplified/Traditional use their regional font fallback. No cfg(test) drivers.
  Each capture locale owns one SSH master; close before the next process.
  Close dropdowns; selectors/defaults share localized labels.
  UTF-8/POSIX locale tooling; verify/export PROTOC on Windows.
  Themes: Claude Warm Light, Catppuccin Latte; Dark approved.
- Preserve saved themes; baseline Claude Warm Light. Keep-awake is session-only
  during tracked Agent activity and permits display/user sleep. No cloud AI/telemetry.
- Ordinary Codex invocations (including --yolo/resume) adapt at Warpai's execution
  event while communication is enabled: official per-session MCP and --no-daemon,
  preserving original CLI/args/cwd/PATH and package-manager ownership. Do not proxy
  its daemon, impersonate vendor commands or write persistent Codex configuration.
  Explicit legacy MCP cleanup is the sole read/edit exception for reserved entries.
- Legacy cleanup disables communication, removes only verified reserved bridge
  entries, preserves other servers and reports failures. Ordinary disable/deselect
  stops future Codex session injection without reading/removing its persistent MCP.
- Keep native Windows argv tests; TOML Unicode escaping precedes shell quoting.
  WSL guest commands stay native without Windows host bridge injection.
  Verified guest observations seed Windows terminal Agent sessions before any wake;
  each terminal/run remains independent. Retain plugin listeners and unsent drafts;
  remote tracking keys include distribution/user. Desktop WSL code is Windows-only.
- About Update checks stable releases; Download opens the exact platform installer
  or Releases fallback. Startup check defaults off/runs once; no installation writes.
- Docs are bilingual and Agent-first; update existing README sections, not a
  changelog. Keep native screenshots/provenance and approved bundled SVG branding.
- Website: https://othinusg.github.io/warpai/ on gh-pages, EN / and ZH /zh/.
  Preserve worktree/branding; update and cmap-check font subsets when copy changes.
  Jekyll pages own translated strings; production baseurl is /warpai.

## Memory maintenance

Keep under 200 lines / 12 KB. Replace superseded facts; keep detailed receipts
in owning specs. Archive useful evidence only; do not read all archives at startup.
