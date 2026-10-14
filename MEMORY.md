# Project Memory

Updated 2026-10-14.
Archive: docs/history/MEMORY-2026-10-13.md.

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
  Contracts: specs/agent-communication-v2/USAGE-PROVIDERS.md; native UI/TLS, no new runtime.
- Accounts/visibility and OS-secure credentials persist application-wide; never
  put credentials in metadata/IPC/logs. Cache readings across project/focus changes;
  restart restores accounts and queries fresh readings.
- Vendor CLI login owns OAuth refresh/sign-in. Synthetic fixtures cannot prove
  authenticated availability; never use personal credentials for tests.
- Usage footer: fixed header, independent scroll, four visible rows, native icons
  and quota bars. Unknown limits remain unavailable; Zen USD balance stays a balance.
- Preserve fixed HTTPS hosts, sensitive headers, bounded IO, generations and all
  twelve provider HTTP contracts in USAGE-PROVIDERS.md.
- agy >=1.1.11: only verified free /usage flags through its resolved executable.
  Tolerate noise, stop after model-turn detection; preserve hidden caches/guard,
  resetting eligibility on update/restart.
- CLI children prepend binary directory to PATH/hide Windows consoles; test real
  visibility, preserve PTY/detached services. Never bill Qoder CN usage polling.

## SSH, task integrity and files

- WSL setup/recovery run on spawn_blocking; owner v4 proved installed-Agent
  discovery's nested-runtime panic. Receipts/audit: FIXES-1.6.4.md. Empty scans
  cannot verify installed-Agent discovery.
- Diagnose actual failed-stage exit/stderr, not failed diagnostic launchers.
  script/diagnose-wsl-companion.bat + .ps1 confirms account/HOME and preserves
  nonsecret stderr; redact configuration JSON/credentials.
- Language relaunch is cancelable; macOS waits for old PID, Windows isolates jobs.
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
- Codex uses per-session MCP/--no-daemon while enabled, preserving CLI/args/cwd/
  PATH/package ownership. No proxy, impersonation or persistent config writes;
  only explicit reserved legacy-MCP cleanup may edit persistent configuration.
  Preserve unrelated servers, report failures, and test Windows Unicode argv/TOML.
- WSL stays native without host MCP injection; preserve listeners/drafts, run,
  distro/user identity and Windows-only UI. Verified observations precede wakes.
- About Update checks stable releases/exact platform installers. Startup check
  defaults off/runs once; no installation writes. Docs retain screenshot provenance.
- https://othinusg.github.io/warpai/ lives on gh-pages (EN /, ZH /zh/); preserve
  branding/worktree, cmap-check font subsets, and Jekyll /warpai production baseurl.

## Native lifecycle repair checkpoint

- Reuse MIT AoE aa02052f's pure Rust screen detector and Orca 38a4958c's
  Codex/agy composer, Pi/OMP marker and Qoder readiness predicates. Native grids
  are bounded, secret-obfuscated and memory-only. Missing rules retain state;
  name-only titles, warnings and quiet output cannot grant idle.
- Grid observations fence run/input/redraw; stable samples preserve wake leases.
  WSL negotiates `native_screen_activity`, sending only activity and exact run.
  Default SSH is unchanged.
- WSL package 38393839766/83c7274b still fails owner Ubuntu attachment despite
  successful account/home/version/runtime/setup and guest/Windows service probes. See
  FIXES-1.6.4.md Oct14 receipt; do not call it resolved from CI alone.
- SSH/WSL Companion install/call/uninstall use `.config/warpai`, matching desktop
  data; Windows installers must not reuse the former registered directory.
- WSL path mapping excludes Windows Agents. Verified rescans clear cached failures.
- MCP task observations, sidebar attention and keep awake share native activity;
  UI focus never acknowledges MCP work; local-only cloud reporting is disabled.
- Owner requires cursor-backed native draft/slash-command transition tests and
  structured task bodies plus reusable versioned context resources, shared by
  local/WSL/SSH MCP and panel Send to agent. Context snapshots stay project-scoped.
  Shared context/MCP/panel sends pin version/digest; legacy text stays readable.
  Store notification snapshots use internal project reads, retaining public guards.
  CONTEXT-RESOURCES.md defines scope.

## Memory maintenance

Keep under 200 lines / 12 KB; receipts belong in specs. Archive useful evidence,
not all startup context; replace superseded facts.
