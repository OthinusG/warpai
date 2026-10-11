# Project Memory

Updated 2026-10-12. Current decisions and unfinished work only. Read this with
[AGENTS.md](AGENTS.md) and [README.md](README.md). Detailed historical receipts
and superseded decisions are in [the archive](docs/history/MEMORY-2026-10-10.md);
consult it only when a specific historical question requires it.

## Working and delivery rules

- Rust compilation/tests and native desktop acceptance run on GitHub, not locally.
  Local Rustfmt syntax parsing, diff checks and script/document checks are allowed.
  Do not install on personal machines or change personal Agent/SSH configuration.
- Check cloud runs every 30 minutes, without frequent polling or empty updates.
  A user-reported error permits immediate diagnosis. Use focused retries while
  debugging, then one complete source-matched acceptance run before delivery.
- Starting CI is not acceptance. Report uninspected/running/failed gates accurately;
  finish authorized fixes and delivery without repeated permission questions.
- Keep historical assets/receipts and active gh-pages; remove only authorized,
  merged, clean, unused branches/worktrees. Retain four validation/release workflows.
- Release immutable tags through existing workflows; both desktops build the same
  source. Follow [release gates](specs/RELEASE.md), including five installers and
  checksums. Finish UI acceptance before screenshots/packages. Protocol major is 1.

## Published releases and maintenance

- Public 1.5.1/1.5.5 receipts remain in their owning patch specs; retain tags/assets.
- 1.6.0 is public at 80e71661, with both desktops and Companion 4.0.0.
  Source-matched native acceptance: 38119276766; builds: 38131069169/38134098256.
- 1.6.2 is public at 0b109b87; both desktop native acceptance: 38147733290.
  English/Simplified/Traditional captures and diagnostics pass; all five installers,
  both checksum lists and source provenance pass. Builds: 38152318120/38155542555.
  Companion stays 4.0.0. Published tags must never be retargeted.
- Main includes both releases and audited cleanup; milestone/1.6.2 snapshots main.
  Maintenance source 045ae5f4 passes both desktop and three-platform Companion
  checks (38162351334/38162351317). Receipt: specs/1.6.2/POST-RELEASE.md.
  Five stale remote branches and upstream tracking were removed; dirty local
  worktrees are preserved. The eight website labels now advertise public 1.6.2.
- Remove private upstream setup/routing, not shared terminal compatibility or
  pinned dependencies. Keep four workflows and the independent gh-pages branch.
  GitHub fork detachment requires a supported operation; never delete/recreate
  this repository or claim detachment without verifying fork=false.

## Project, worktree and Agent ownership

- Warp's native worktree system owns creation, catalog and tabs; creation opens a
  new tab without starting an Agent. No separate worktree registry/session manager.
- Canonical Git common-directory identity joins registered checkouts/subdirectories,
  including empty tabs and worktrees outside main. Ancestry, basename, branch and
  remote URL cannot establish membership.
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
- Sidebar owns sessions; panel owns project messages/tasks/reviews/history. Reuse
  native widgets, prominent modes and 8px spacing. Default closed; events must
  preserve focus/drafts and never open the panel automatically.
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
  suppress Windows console windows. Qoder CN remains without a verified safe
  background usage adapter; do not submit a potentially billed model prompt.

## SSH, task integrity and files

- App 1.6.0 is published; Companion: 4.0.0;
  WSL has a separate Windows-bundled guest binary/path. Windows MCP settings:
  logged-in WSL 2 accounts, install checkbox, distro/user-scoped Rescan/Remove all.
  Native Codex may add --no-daemon; no wrapper or host MCP injection.
  WSL steps 60s/process 420s; clear static polling fence before live WSL; CI-DIAGNOSIS.md.
  Check Windows rlto/release_bundle; debug-only WSL helpers share callee cfg.
  Contract: specs/wsl-1.6.0/TECH.md. Accept WSL package 3.x previews.
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
- Wake polls until idle; manual input/cancel/run replacement fences delayed Enter.
  Cancel pauses wake until resumed input; closing a view invalidates its run.
  Evidence/test labels never authorize command execution or secret-file access.
- Back up schema migrations consistently; no downgrade or automatic legacy
  identity migration. Preserve read-only legacy metadata, uncertainty and receipts.
  macOS sockets require short private /tmp paths; Windows uses bounded asynchronous
  native named pipes, with a new listener before handing off a connection.

## Product defaults, CLI integration and website

- Identity: warpai, Warpai, dev.warpai.Warpai, OthinusG/warpai; keep WARP_* compatibility.
  1.6.2 moves data/settings to ~/.config/warpai; retain sources/profiles/new values.
  Public defaults only; English/zh-Hans/zh-Hant UI, restart to apply language.
  Inspect menus/Review/loading/toasts; scan wrapped text/header_text; keep IDs; no cfg(test) drivers.
  Each capture locale owns one SSH master; close before the next process.
  Close dropdowns; selectors/defaults share localized labels.
  UTF-8/POSIX locale tooling; verify/export PROTOC on Windows.
  Themes: Claude Warm Light, Catppuccin Latte; Dark approved.
  CI: poll every half hour in-session; owner forbids local background monitors.
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
- About Update checks stable releases; Download opens the exact platform installer
  or Releases fallback. Startup check defaults off/runs once; no installation writes.
- Docs are bilingual and Agent-first; update existing README sections, not a
  changelog. Keep native screenshots/provenance and approved bundled SVG branding.
- Website: https://othinusg.github.io/warpai/ from independent gh-pages branch,
  EN / and ZH /zh/. Preserve its worktree and app branding assets. Jekyll pages
  assign their own translated strings; self-hosted font subsets must be updated
  and cmap-checked when copy changes. Use configured /warpai baseurl in production.

## Memory maintenance

Keep under 200 lines / 12 KB. Replace superseded facts; keep detailed receipts
in owning specs. Archive useful evidence only; do not read all archives at startup.
