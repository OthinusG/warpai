# Project Memory

Updated 2026-10-31. Archives: docs/history/MEMORY-2026-10-{13,19-SSH-MCP,30}.md.

- MCP renames must migrate local/SSH/WSL persisted add/get/remove argv and
  server_name together; QoderCN may return status-zero for not-found. See specs/RELEASE.md.

## Working and delivery rules

- Rust builds/native acceptance: GitHub with caches. Local syntax/scripts only; no personal install/config changes.
- Capture baseline: `fixtures * 8 + 232` PNGs/language, including two icon frames;
  update both workflows. macOS 14 GB runner: incremental off, clean warpai before relink.
- Windows fixtures must retain the real disposable KnownFolder profile; spoofing USERPROFILE can make LocalAppData unavailable.
- Peer wake cancellation fixtures end on captured paste; scheduling all agents
  first can exhaust Claude's Enter delay. Receipts: specs/agent-communication-v2/CI-2026-10-21.md.
- Dependencies: docs/FEATURE-DEPENDENCIES.md; desktop Git stays unbundled.
- Preserve assets/receipts and gh-pages; remove only authorized, merged, clean,
  unused branches/worktrees. Linux packaging: specs/linux-desktop/TECH.md.
  Bullseye Git 2.30.2 needs global safe.directory for canonical `/__w/...`;
  command-line `-c` and runner workspace aliases do not work.
- Immutable tagged releases: accept UI, create draft, then attach Windows/Linux.
  See specs/RELEASE.md gates/HTTP 500 recovery. SSH major 1; Agent Bus 2.

## Published releases and maintenance

- Public: Warpai 1.8.1 / Companion 5.5.0. All 19 assets verified; match README links.
  Linux x64/ARM64: AppImage/DEB/RPM/Arch, glibc >=2.31. Changes go in Release notes.
- Preserve unpublished v1.7.2, immutable v1.7.6 and unpublished v1.8.0 tags/drafts.
  Pane icons passed both desktops (41053972438); immutable release source 8ff0c00f.
  Authorization/receipts: specs/RELEASE.md. Release monitoring follows its 30-minute cadence.
- Windows SSH uses PowerShell, never CMD; remote OS owns grammar/path/completion.
  See SSH-POWERSHELL-AUDIT.md; preserve SSH evidence and drafts.
- Artifact-only macOS must not publish Windows. Windows CI/release pins official
  protoc 36.2 + SHA-256; Chocolatey NuGet 503/504 blocked builds. See specs/RELEASE.md.

## Project, worktree and Agent ownership

- Supervision dispatches anytime, retaining unresolved task/request IDs across
  waits/yields and retiring only handled outcomes. No batch/exclusive watch.
- Native worktrees own creation/catalog/tabs; creation never launches an Agent.
- Membership uses canonical Git common-directory identity across registered
  checkouts/subdirectories, including empty tabs. Never infer it from names,
  ancestry, branches or remotes. Checkout changes keep old tasks/history and
  cannot transfer durable identity or Coordinator authority.
- Non-Git canonical roots are separate projects with Worktree unavailable; they
  neither inherit nor clear Git Coordinators. Isolate clones and SSH authorities.
- Project lists all online peers before pagination and owns mode. Worktree messages
  stay checkout-local except Coordinator traffic; panel reads never grant admission.
- Eligible Agents may replace the Coordinator without stopping its process or
  changing tasks. Authority belongs to the project and exact native run, survives
  focus/view changes, and ends only on termination/revocation/shutdown. Restarts
  cannot inherit it. Bound handoffs with unfinished counts/paginated instructions.
- Worktree removal: native Git, recheck exact registered path, protect main/open/
  locked/dirty/untracked/ignored checkouts; never force or delete branches. No SSH removal.
- Sidebar owns sessions; panel owns communication/history. Native widgets, 8px
  spacing, default closed; preserve focus/drafts. Derive icons/coverage from
  CLIAgent, retaining aliases/custom MCP; no duplicate allowlist/shell fallback.

## Usage accounts and native Orca port

- Native Orca: https://github.com/stablyai/orca at
  3e0b82835856dde57f43a47661cf736e3e3dd90d; preserve MIT notices. Native UI/TLS,
  no new runtime. Contract: specs/agent-communication-v2/USAGE-PROVIDERS.md.
- Accounts/OS-secure credentials persist app-wide; never expose credentials in
  metadata/IPC/logs. Cache across focus, requery on restart. Vendor CLI owns login;
  synthetic fixtures cannot prove auth. Personal credentials are off-limits.
- Keep fixed HTTPS hosts, sensitive headers, bounded IO/generations. agy >=1.1.11
  uses verified free /usage flags. Qoder/CN query CLI --sdk daemon v2, falling back
  to legacy JSONL get_usage_info; no Prompt/model turn or SDK dependency.
  Poll once/account/5 minutes; manual bypass. Unknown quotas remain unavailable;
  Zen USD stays a balance. Keep fixed header/scroll; children prepend binary PATH
  and hide Windows consoles without breaking PTY/services. Revalidate protocol changes.

## SSH, task integrity and files

- SSH masters need no retained argv; independent transport does. OpenSSH owns
  ~/.ssh. SSH Remote defaults off and gates all Companion APIs independently of
  OpenSSH; MCP needs 5.5.0. Contracts: specs/agent-communication-v2/{SSH-MCP-SETTINGS,INSTALLATION}.md.
- WSL setup/recovery/discovery must use spawn_blocking to avoid nested-runtime
  panic; empty scans prove nothing. Receipts: FIXES-1.6.4.md.
- Diagnose real stage exit/stderr, not launcher failures. Use script/diagnose-wsl-companion
  .bat/.ps1 for account/HOME and nonsecret stderr; redact config/credentials.
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
- OSC 9/warnings/silence cannot prove completion; native observations cover all
  Agents. Vendor runtime acceptance stays pending.
- WSL setup hardens owned real namespaces to 0700; preserve SSH/account data.
  Restore saved live accounts with one deferred rescan, never a failure retry loop.
- Input/cancel/run replacement fences Enter; closing invalidates its run.
- Back up schema migrations consistently; no downgrade or automatic legacy
  identity migration. Preserve read-only legacy metadata, uncertainty and receipts.
  macOS sockets require short private /tmp paths; Windows uses bounded asynchronous
  native named pipes, with a new listener before handing off a connection.

## Product defaults, CLI integration and website

- Icons belong to panes, not tabs (owner correction Oct30). Save each icon in its
  existing tab-config [[panes]] TOML entry: builtin string or { local = "file.svg" }.
  Reuse bounded async ~/.config/warpai/icon discovery; Agent > selection > SSH > terminal.
  Contract: specs/custom-tab-icons.md. Published in 1.8.1; preserve v1.8.0.
- Keep WARP_* compatibility and ~/.config/warpai recovery sources/profiles/values.
  English/zh-Hans/zh-Hant need restart; localize outcomes, preserve IDs.
- Native capture: each locale owns/closes an SSH master; close dropdowns; preserve
  UTF-8/POSIX tooling and PROTOC. Test real Windows glyph fallback/font rendering.
- Preserve saved themes, Claude Warm Light/Latte/Dark and approved SVG branding.
  Keep-awake tracks activity; no cloud AI/telemetry.
- Local Codex uses session MCP/--no-daemon, preserving CLI/args/cwd/PATH/packages.
  No proxy/impersonation/persistent config writes except explicit reserved legacy
  cleanup. Preserve other servers, report failures, test Windows Unicode argv/TOML.
- Collaboration MCP registrations are scope-specific: local `warpai-local-com`, SSH
  `warpai-ssh-com`, and direct WSL `warpai-wsl-com`; updates remove only recognized
  legacy Warpai entries and preserve user-owned MCP servers.
- WSL stays native without host MCP injection; preserve listeners/drafts, run,
  distro/user identity and Windows-only UI. Verified observations precede wakes.
- About Update checks stable releases/exact platform installers. Startup check
  defaults off/runs once; no installation writes. Docs retain screenshot provenance.
- https://othinusg.github.io/warpai/ lives on gh-pages (EN /, ZH /zh/); preserve
  branding/worktree, cmap-check font subsets, and Jekyll /warpai production baseurl.
- Synchronize both READMEs, source SSH/WSL guides and bilingual gh-pages home,
  strings/footer/install/MCP/features/overview; verify scope names and actual assets.
  Keep change/migration history in Release notes, outside product documentation.

## Retained native lifecycle constraints

- Keep Herdr Idle/blocked semantics, run/input/redraw fences and vt100 capture.
- SSH/WSL installers use `.config/warpai`; setup locks explicitly unlock despite inherited descriptors.
- Signed birth-time hashes preserve positive identities; absent birth time cannot prove inode reuse.
- Focus publishes session before roots; replay cached roots and scope remote polls to admission.
- Bindings register before MCP discovery; readiness stays separate. Focus never acknowledges work.
- Preserve context/version/digest/public guards and task-anchored native threads/reviewed memory.
- Zsh/Bash bootstrap uses raw PTY/ZLE off; Zsh restores sane after reading heredoc,
  before eval. Minify PowerShell SSH seed before UTF-16/base64 to stay below MAX_CANON.
- Untargeted remote-peer polls must not bump WSL/SSH generations or abort auto-warpify.
  Both bootstrap/generation repairs shipped in 1.7.8; details in the Oct30 archive.

## Memory maintenance

Keep under 200 lines / 12 KB; receipts go in specs. Preserve evidence;
replace superseded facts.
