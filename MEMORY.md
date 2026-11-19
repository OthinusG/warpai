# Project Memory

Updated 2026-11-19. Archives: docs/history/MEMORY-2026-10-{13,19-SSH-MCP,30}.md.

- MCP renames must migrate local/SSH/WSL persisted add/get/remove argv and
  server_name together; QoderCN may return status-zero for not-found. See specs/RELEASE.md.

## Working and delivery rules

- Rust builds/native acceptance: GitHub with caches. Local syntax/scripts only; no personal install/config changes.
- Owner's CI monitoring preference: remain in the active working turn, poll every
  30 minutes, and merge only after required checks pass; do not detach a monitor.
- Capture baseline: `fixtures * 8 + 251` PNGs/language, including the third-page
  follow-up navigation frame and prior icon/reconcile/follow-up captures;
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
- Fork LFS quota exhausted: pull `*.pdb` via temporary upstream `lfs.url`, then unset and `git reset`.

## Published releases and maintenance

- Warpai 1.9.0 published 2026-11-17 from immutable tag source f2542163.
  Companion 6.1.0 confirmed by owner; protocol majors unchanged. All 19 release
  files fully downloaded and hashed; canonical public URLs return HTTP 200.
  Acceptance, native captures and release run receipts: specs/1.9.0/RELEASE.md.
  DMG independent public SHA-256 is recorded there. Homebrew PR #3 merged as
  3599d7f7 after parsing/style/audit/fetch/dry-run checks; website source f615f39a
  is deployed with 34 live pages checked. Mobile header acceptance is retained.

- Owner requested 1.9.0 release preparation on 2026-11-17. Contract/ledger:
  specs/RELEASE.md and specs/1.9.0/RELEASE.md. Target Companion 6.1.0 distinguishes
  additive shared supervision/follow-up features from published 6.0.0; protocol
  majors stay unchanged. Prepare source-matched native/WSL acceptance and draft
  installers with auto_publish=false before the publication gate.

- Public: Warpai 1.9.0 / Companion 6.1.0. All 19 assets verified; match README links.
  Linux x64/ARM64: AppImage/DEB/RPM/Arch, glibc >=2.31. Changes go in Release notes.
- Preserve unpublished v1.7.2, immutable v1.7.6 and unpublished v1.8.0 tags/drafts.
  Pane icons passed both desktops (41053972438); immutable release source 8ff0c00f.
  Authorization/receipts: specs/RELEASE.md. Release monitoring follows its 30-minute cadence.
- Windows SSH uses PowerShell, never CMD; remote OS owns grammar/path/completion.
  See SSH-POWERSHELL-AUDIT.md; preserve SSH evidence and drafts.
- Artifact-only macOS must not publish Windows. Windows CI/release pins official
  protoc 36.2 + SHA-256; Chocolatey NuGet 503/504 blocked builds. See specs/RELEASE.md.
- macOS DMG creation retries normal detach on temporary busy mounts; never
  convert before successful detach or delete a still-mounted workspace.
  Failure evidence and acceptance: specs/RELEASE.md (2026-11-16).
- `warpdotdev` mirrors `warpdotdev/warp` `master` fast-forward only; it is reference,
  never merged into `main`. Daily sync: `.github/workflows/sync-warpdotdev.yml` (03:17 UTC).

## Project, worktree and Agent ownership

- Task handoff contract: specs/agent-communication-v2/HANDOFF.md; immutable pins grant no execution or source-task access.
  1.9.0 plan: specs/1.9.0/TODO.md; process-mapping audit: specs/1.9.0/PROCESS-REVIEW.md.
  R2 prompt: AGENT-EXECUTION-1.9.0-R2.md; assume round one provisionally complete, work only the delta.
  research: specs/SUPERAGENT-REVIEW-1.9.0.md; Nov12 audit: specs/1.9.0/BRANCH-AUDIT-2026-11-12.md
  (attention byte budget; ACK versus terminal follow-up closure). Nov13 review:
  specs/1.9.0/IMPLEMENTATION-REVIEW-2026-11-13.md (retention guards and synthetic space cursors).
  Owner authorized reviewer repair, independent follow-up browsing and supervision
  recovery plus CI: specs/1.9.0/AGENT-EXECUTION-THREE-GAPS-2026-11-13.md.
  Assigned to qoder-cn via task `7aba27fa-7b33-41b5-83fa-53d43fd86e4f`.
  Implementation Agent reports directly to owner; sender must not monitor.
  Owner excludes external event intake
  and danger-full-access remediation; retain native authority/guarded recovery.

- Supervision dispatches anytime, retaining unresolved task/request IDs across
  waits/yields and retiring only handled outcomes. No batch/exclusive watch.
- Task-linked follow-ups pin the task revision at send; wake/wait delivery skips
  overtaken `message`/`request` entries (kept unread in inbox/history), and the
  panel derives a bounded, filter-scoped `attention` projection with
  reconcile/answer/review/recovery/permission/unreachable/dependency/overdue
  actions. Additive only: no lifecycle/schema/protocol change, and older
  Companions omit the field. See specs/agent-communication-v2/API.md (2026-11-08).
- Overtaken follow-ups end only via `task_followup_reconcile` (operator/issuer/
  reviewer/active Coordinator; version+revision checked; acknowledges without
  rewriting; releases a waiting sender with `result_failed`). A durable
  `followup_reconciled` event is the terminal fact: late results are refused
  without treating every acknowledged message as resolved, a self-ended request
  closes with no self-result, and questions derived from the ended follow-up are
  retired. "Resend" is a new `agent_send`, never a task restart. Attention is
  cursor-paged with whole-set `total`/`counts`; previews stay bounded (8 per task,
  truncation marked) and the full body stays readable through its thread; never
  derive totals from page length. Acknowledgement is read state only: an ACKed
  overtaken follow-up stays listed until the `followup_reconciled` event or a
  succeeded/failed result, so late results stay refused without hiding unhandled
  work. One panel response is bounded by an aggregate serialized UTF-8 budget
  (`MAX_PANEL_RESPONSE`, 448 KiB, under Companion `MAX_RESULT` 512 KiB):
  collections and attention previews shed tails, every cursor stays on the last
  real row (Private-only spaces resume with `""`), totals stay whole-filter, and an unfittable panel returns
  `capacity_exceeded` instead of an oversized frame. Capability
  `followup_reconcile`. See API.md (2026-11-11, 2026-11-12).
- TaskDelete/purge preview/HistoryPurge protect ACKed overtaken instructions
  until reconciliation/final result; read acknowledgement alone cannot release
  their history. Nov13 regression coverage: `storage::review_tests`.
- Nov13 three execution gaps (code `5e69f5e81`, base `9441e9f79`): an omitted
  reviewer on reassignment keeps the exact synthetic `warp:<project>` Operator
  without a persisted Agent lookup (explicit reviewers still replace it). Follow-ups page
  independently of the attention cursor via native `PanelQuery.followup_*`
  (task-scoped, nonmutating, 256 KiB page, resume
  cursor); those request fields are omitted when idle so older Companions still
  decode. `warp_project_bootstrap` adds a bounded `supervision` block
  (`Operation::SupervisionIndex`, capability `project_supervision`) rebuilding
  delegated tasks, outgoing requests without a final reply and unacknowledged
  results without changing state. Evidence: specs/1.9.0/TODO.md.
- Nov14 recovery repair: specs/1.9.0/COMMUNICATION-AUDIT-2026-11-14.md. Recovery
  retires self-reconciled requests, retains real unhandled results, uses active
  Coordinator scope and keeps archived pending facts. Questions addressed to the
  caller have distinct answer rows (`counts.questions`); totals count entities.
  Follow-up history stores current page cursors. Browsing uses the native Panel;
  no `warp_task_followups` MCP tool or broader message read permission was added.
- Vendor structured observation stays `existing_unverified`: only screen
  manifests exist; no adapter or whole-task retry wrapper. Keep unknown on
  missing identity, and never infer failure/health from idle, timeout, "429" or
  disconnect. See specs/1.9.0/TODO.md P0-07.
- Expired advisory reservations tied to `active` or unresolved `unknown` attempts
  keep their row and `abandoned` warning until execution resolves; TTL still
  permits new writes because reservations are not OS locks. See specs/1.9.0/TECH.md.
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

- Native system HTTP on all desktops (Nov3): specs/system-network.md.

## Usage accounts and native Orca port

- Antigravity child routing: specs/antigravity-cli/USAGE-NETWORK.md.
  Resolve the usage API through CFNetwork PAC / WinHTTP / GIO, apply proxy
  variables only to the usage child, and keep loopback direct. CLI environment
  supports one route per invocation, not arbitrary per-host PAC or failover.
  Native static/PAC/CONNECT/loopback acceptance, default/warp_platform app checks,
  agent_usage and release_updates regressions passed on all three desktop
  runners in run 43859471781. Owner authorized
  checking every 30 minutes and merging PR #12 into main once all checks pass.
- Antigravity timeout diagnosis (2026-11-16, installed Warpai 1.9.0,
  agy 1.5.9): the exact free `/usage` invocation succeeded in 3.72s with
  shell HTTP(S) proxy variables (SUCCESS, zero model turns); removing proxy
  variables exceeded 35s despite `--print-timeout 20s`. The macOS launch
  environment had no proxy variables while system networking used PAC.
  CLI child networking does not inherit Warpai's native HTTP PAC transport;
  distinguish this path from desktop-owned HTTP before increasing timeouts.
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
  OpenSSH; current SSH/WSL needs Companion 6.0.0; native local needs none. Contracts: specs/agent-communication-v2/{SSH-MCP-SETTINGS,INSTALLATION}.md.
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
  startup/60-second idle shutdown. Owner-corrected 2.0 scope: optional/default-off
  SSH/WSL guardian, no native local persistence; low CPU/RAM, parallel sessions and
  existing serial per-Agent tasks. One guardian per execution account, not Agent;
  auto-fetch on reconnect; private CLI/no PATH writes. Owner requires bare resume
  current/unique/MRU; bare stop stops all account guarded runs without confirmation.
  CLI only list/resume/stop + help/version; extra Herdr tools need a proven gap.
  Methods/ledger: specs/2.0.0/README.md. Unverified 2026-11-18 slice
  (source through efe9fd12): guarded session retention plus exact attachment
  fence, CAS takeover, writer epoch and a bounded generation-keyed write-receipt
  ledger with non-evictable unknown delivery and between-write cancellation;
  fail-closed Unix peer UID and Windows client-SID recheck; guarded-only 300s
  ended-output retention; session-level stale or unknown delivery does not close
  the shared connection; a pure bounded catalog prototype (stable pages,
  revision-scoped watch gap, MRU resume, state age) is isolated-tested but not
  wired to sessions or exposed; guarded launches now require explicit
  session_kind (agent/shell/command) with pre-spawn policy binding checks;
  a Unix account-global guardian election now derives its root from the passwd
  home, holds one account lock, binds private per-route sockets under
  /tmp/warpai-guardian-<euid>, checks peer UID and a WAG1 identity preface, and
  scopes capabilities by route without merging SSH/WSL Stores; a durable
  guardian/catalog.sqlite3 (application_id 0x57474341, user_version 1) refuses
  unknown identity without overwrite, validates schema and rows before writing,
  marks prior live rows unknown at boot, and now records guarded launch/attach
  with an account-serial signed sequence before the corresponding ack;
  guarded exit is observed by the terminals registry on the existing guardian
  tick (native process gone plus drained output) without requiring a client
  read, and only after the durable write succeeds;
  clients verify private directory/socket ownership, mode and kernel peer UID,
  spawn only for an absent route, and never unlink an incompatible owner;
  prefaces run off the dispatch loop and zero admitted routes fail closed;
  an internal installed-helper `warpai-companion --guardian-stdio ssh|wsl`
  bridges bounded stdio to the guardian (the GUI never starts a guardian);
  additive proto tags 10/11 provide account-fenced catalog list/inspect with
  boot/revision-bound cursors, tags 12/13 freeze watch/stop shapes but return
  feature-unavailable, and `ManagedInitialized.account_fence=7` comes only from
  guardian admission;
  these components are host-tested (including a real-listener guarded launch,
  bridge drop, same-boot reconnect, attach and read-free ended observation), no
  production client selects the guarded transport, no CLI exists, and retention
  eviction is not synced to catalog rows;
  warp-agent-bus, remote_protocol and the warpai
  binary compile and their host test suites pass on macOS arm64 (remote SSH/WSL
  still unrun);
  guarded_remote_sessions_v1 stays unadvertised
  until native proof; CLI, stop RPC and WSL lifetime remain pending.
  No desktop auto-deployment.
  Archived step-by-step execution prompt: specs/2.0.0/AGENT-EXECUTION-PROMPT.md.
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
- A cancellation pause is transient: confirmed native/screen idle or accepted
  AgentReady restores receiving without another user submit. Keep input/run,
  settling, permission and execution-ownership guards; idle never retries a task.
- Back up schema migrations consistently; no downgrade or automatic legacy
  identity migration. Preserve read-only legacy metadata, uncertainty and receipts.
  macOS sockets require short private /tmp paths; Windows uses bounded asynchronous
  native named pipes, with a new listener before handing off a connection.

## Product defaults, CLI integration and website

- Product documentation describes supervision and handoff as current workflows,
  without version comparisons or changelog framing. Keep bilingual READMEs,
  source Agent guides and gh-pages overview/features/workflow/MCP pages aligned.
  Distinguish read acknowledgement from explicit follow-up closure and submission
  from acceptance/integration; recovery includes unresolved archived facts.

- Hero screenshots: owner-approved locally edited light/dark PNGs (Nov6), both
  4032x2304, shared byte-for-byte by bilingual READMEs/gh-pages Overviews and
  homepage CTA previews. Preserve resolution; do not regenerate whole images.
  READMEs use picture media selection; site retains ink/paper CSS and CTA fade.
  Omit fixed README img dimensions: GitHub clamps width without resetting height.
  Homepage and installation-guide compatibility notices removed by owner request;
  retain current Companion downloads and setup steps. Provenance: docs/images/README.md.

- Icons belong to panes, not tabs (owner correction Oct30). Save each icon in its
  existing tab-config [[panes]] TOML entry: builtin string or { local = "file.svg" }.
  Reuse bounded async ~/.config/warpai/icon discovery; Agent > selection > SSH > WSL > terminal.
  Contract: specs/custom-tab-icons.md. Published in 1.8.1; preserve v1.8.0.
- Local SVG icons opt into theme tint with `autocolor="true"` + `currentColor`;
  normalize to a red mask and paint with `main_text_color`. Single-color only;
  PNG and non-opted icons keep original colors.
- Bundle btop/herdr/spotify/ssh/wsl in app/assets/bundled/pane-icons; user files
  override by name. Defaults: SSH ssh.svg, WSL wsl.svg. Keep herdr-white out;
  herdr.svg carries autocolor="true".
- Keep WARP_* compatibility and ~/.config/warpai recovery sources/profiles/values.
  English/zh-Hans/zh-Hant need restart; localize outcomes, preserve IDs.
- Native capture: each locale owns/closes an SSH master; close dropdowns; preserve
  UTF-8/POSIX tooling and PROTOC. Test real Windows glyph fallback/font rendering.
- The live collaboration seed sends nine revision-pinned follow-ups, then
  cancels/retries so the task continues at revision 3; later capture assertions
  on the private/retry state must track that revision.
- Preserve saved themes, Claude Warm Light/Latte/Dark and approved SVG branding.
- Vertical chrome: side toolbars live inside inset floating columns; no title
  search setting/config override. Closed columns retain reopening controls.
  Reserve the center toolbar height even with both columns open so terminal input
  stays at the closed-right-column top position; specs/terminal-top-alignment.md.
  Field correction contract: specs/macos-bugcheck-2026-11/TECH.md. Left inset: 6px; right: 12px. Column shadows use matching centered halos.
  Resize strips include the column padding and use their own cursor layer;
  dragging preserves grab offsets. Contract: specs/sidebar-resize-hit-area.md.
  History Archive is idempotent and visibly labeled; task Delete is versioned,
  scoped and retains existing certainty/reference/unread-message guards.
  Archive requires no reason. Home excludes cancelled tasks before pagination;
  Back clears archive/state/page filters. History and recovery retain records.
  Acceptance: specs/1.9.0/TASK-ARCHIVE-VISIBILITY.md (2026-11-16).
  Agent executable detection ignores ASCII case; argv stays verbatim. Shell override lookup
  follows shell semantics: PowerShell ignores ASCII case; POSIX shells retain it.
  Warpui shadow spread is drawable blur padding; zero clips the outer shadow.
- Explicit delegated tasks use warp_task_assign; warp_agent_send, including
  expects_result, creates messages only. Both persist in History; do not infer
  task authority from prose. Contract: specs/agent-communication-v2/API.md.
- Chrome: 8/12/20px tokens; main panel outlines must look rounded. See specs/macos-rounded-containers.md.
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

- 2.0.0 guardian stays default-off. Installed helpers advertise the private
  guarded capability to authenticated local operators. The desktop data bridge
  remains gated pending desktop terminal and shell-block restoration.
- Windows guardian startup verifies that its suspended service inherits no
  launcher Job before resuming. Private named-pipe peers check native account
  SIDs, and the guardian shares the ordinary SSH Store's ownership lock.
- Companion CLI `ctl list|resume|stop` probes only. Bare resume recognizes its
  native guarded session before selecting in the helper's logical scope. Bare
  stop is account-wide and operator-only, with boot-scoped receipts. Full
  V01-V30 release acceptance remains open.
- S1-r review/repairs and source-matched host receipts are in
  specs/2.0.0/IMPLEMENTATION.md. Existing Store locks and exact logical service
  identity fence ordinary-helper forwarding during guardian cutover.

- A guarded writer acknowledgement follows native ownership. Catalog write
  faults leave the latest exact generation pending in the retained run and
  maintenance republishes it without replaying attachment.

- Guarded bridge admission matches scoped service and socket guardian/boot
  before project access. WSL guarded routing checks native guest root identity.
  Future-launch opt-in is per verified SSH service or WSL distro/user; ordinary
  collaboration retains its connection-owned policy.
- Linux OpenSSH run 44279117156 at `91f7a2aa` proved guarded bridge-loss
  continuity, exact run/boot rejoin and stale-writer refusal. Full desktop
  and V02/V04 release gates remain open. Details in specs/2.0.0/IMPLEMENTATION.md.

- Guarded resize is idempotently sequenced per writer. Pending PTY launches
  reserve a session slot before releasing the shared registry for native spawn.

- On WSL, SSH-first selects the owned manifest-checked dual-route guest where
  both helpers are installed. Ordinary stdio only follows a guardian whose
  logical service ID matches its legacy Store. Native run 44311567207 at
  `acd9e63e` passed both startup orders, a bridge-free exact-run reconnect and
  ordinary workspace coexistence. The full release matrix is tracked in specs.

- Companion installers create private component directories and repair legacy
  group/other write bits. SSH-first WSL election requires account-owned
  installation ancestors without group/other writes and verifies the manifest.

- The Unix guardian caps pending prefaces and established bridges separately.
  Long-running CLI attachments share bridge slots. Two local control permits
  keep account catalog and stop available when all 32 bridge slots are occupied.

- Local `ctl resume` can open another project in the helper's logical scope
  through the exact live run and its in-memory root. Root identity is rechecked
  and the catalog keeps no path. Project admission does not take the writer.

- Guarded Agent wake checks use runtime notifications and quiet/ready deadlines
  with 300 ms delivery pacing and five-second idle reconciliation. Direct wakes
  resolve by terminal using a streamed, recipient-indexed routing projection
  that avoids inbox bodies. Pool notices retain project-wide arbitration.
  Native delivery latency and resource benchmarks remain release gates.

- Bash/Zsh managed remote bootstrap installs a session-only `warpai` function
  with an absolute, manifest-verified SSH or WSL helper. User command collisions
  keep precedence. Native Windows still needs its guardian and shell path.
- Darwin leaves `PENDIN` when a raw CLI returning to canonical mode uses only
  `TCSANOW`. Flush pending input after restoring attributes without draining
  output. The full local comparison and real macOS/Linux SSH TTY test passed
  on `3d33a7d6` (remote run 44328848156).
- Strip `WSL_DISTRO_NAME` from `wslpath`'s child environment before comparing
  its native result against the parent hint. The native wrong-hint fixture
  and both route-election orders passed on `904801ed` (run 44333896370).
  Catalog writer occupancy is an
  optional exact-generation observation outside durable revision and MRU.
- A remote editor commit with a missing reply keeps its attempted hash alongside
  the baseline. Reconnect checks the original account/project and reads the hash.
  Automatic saves wait for explicit reconciliation; save-and-close needs the
  current content confirmed. The editor owns the receipt for its lifetime.
- Managed channels initialize native identity and clean up PTY leases/staging in
  the blocking pool. EOF holding a session lock must not stall other channels.
- Controlled Linux/macOS OpenSSH on `38d31b4f` proved a tracked task submitted by
  the same Agent after both SSH bridges exited (run 44344251976).
- Pool-wake dedup checks every pending pool notice already delivered in the
  current run. Earlier unacknowledged direct mail cannot hide a reservation.
- Bound OSC before the VT readiness parser accumulates it. Keep the original
  PTY stream and ring offsets for live terminal output.
- Guarded launches and resize reserve ring/input buffers and both screen grids
  against a shared 64 MiB charge. Retained runs keep their reservation. This
  bounds the charged data structures and does not measure total process RSS.
- Guarded project Broker leases keep live and in-flight work resident. Idle
  listeners can close under age or capacity pressure while Store history stays.

## Memory maintenance

Keep under 200 lines / 12 KB; receipts go in specs. Preserve evidence;
replace superseded facts.
