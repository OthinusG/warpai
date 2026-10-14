# 1.6.4 usage and readiness repairs

## Scope and acceptance

### Follow-up: existing local checkout and test-package handoff

Owner reproduction: macOS, bare `codex` and `agy`; both panel and MCP report
missing main-checkout peers or stuck lifecycle state. Read-only local diagnostics
found activation failures and a main-checkout admission whose repository and
checkout fingerprints predate the current directories. Fresh-database fixtures
did not cover this upgrade/replacement case.

- A native launch into a different repository at an old path must create a new
  project admission without inheriting old agents, tasks or Coordinator authority.
  Keep old durable history and reject old capabilities. Replacing only a checkout
  within the same repository still requires explicit rejoin.
- Scope opt-out records to the current repository and checkout; an old directory's
  opt-out must not disable a new repository at that path.
- Include only fixed error codes in activation diagnostics, never commands,
  capabilities, configuration or draft text.
- Regress same-path repository replacement with persisted admissions, bare Codex
  and Antigravity MCP discovery, both panel modes and stale-client rejection.
- The default-branch Windows workflow must recognize artifact-only macOS handoffs.
  Run 38378106900 used the older main workflow and failed before compilation at
  LFS checkout of nonexistent v1.6.4. Branch test packages passed separately.

Validation: focused protocol regressions followed by source-matched cloud desktop
builds/native captures and remote suites. Do not claim live vendor lifecycle
acceptance from synthetic MCP discovery alone.

Owner follow-up: warning/error termination must recover idle only when the CLI
has actually returned to its input prompt. The existing broker lacks completion
observations for Antigravity and deliberately treats Codex OSC 9 text as opaque.
Therefore a failed turn that skips `warp_agent_ready` can leave stale working
state. A warning, quiet output or MCP query cannot establish idle. Investigate
native lifecycle evidence independently of the admission repair. Antigravity's
official Stop hook includes `fullyIdle` (background work may remain); Codex
0.161.0 exposes native task/MCP startup state through its terminal-title status.
References: https://antigravity.google/docs/hooks and
https://github.com/openai/codex/blob/rust-v0.161.0/codex-rs/tui/src/chatwidget/status_surfaces.rs.

### Additional acceptance: every managed Agent and WSL restart

Owner requires lifecycle repairs for every managed Agent, on both desktops;
warnings/errors must not leave a completed or failed turn permanently starting or
working. Use verified native idle evidence, preserving permission, background
task, user input and exact-run fences. Do not infer idle from error keywords,
silence, or successful MCP queries. Audit per-Agent native lifecycle coverage and
make unsupported observations explicit rather than claiming universal detection.

Windows reproduction uses test installer from run 38368976431. After successful
WSL installation, collaboration reports workspace unavailable, files report lost
connection, and restart shows Companion uninstalled. Two source defects:
guest setup creates the service namespace with default directory permissions
while the account service requires 0700; restoring a saved logged-in selection
does not trigger a read-only installation rescan.

1. Create the selected guest's setup namespace with 0700 and safely harden
   previous owned directories created by setup; reject symlinks/foreign owners.
   Preserve settings/history and separate SSH/distribution/account namespaces.
2. Rescan once when an enabled saved account becomes live. Preserve explicit
   retry after failure, and never continuously rescan a failed account.
3. Test setup-before-first-service connection and reconnect with the installed
   guest payload, plus fresh/reloaded desktop selection and non-selected accounts.
   Existing installation acceptance must connect the workspace after a nonempty
   setup scan rather than only test executable/runtime presence.
4. Run Windows application checks, real WSL capture and three-platform remote
   regressions. Do not replace the permission checks with permissive defaults.

### Revised lifecycle implementation: reuse open-source observation

Owner explicitly rejects further ad-hoc status patches. Research checkpoints:
Orca 38a4958cad8cc75331a8a0a1e7d28b4674b212f7 (MIT), Agent of Empires
aa02052f5cf2f12c3972fcd7569e0b650f8e2aa1 (MIT), and cmux
cfb20940f5ca643ed68d70b80ce59a88e9b5a1ec (mixed licensing; research only).
Orca separates actual idle evidence from its display-only name-title fallback;
AoE's pure Rust manifest engine supplies bounded screen-region/priority rules and
native permission/busy fixtures, including Antigravity. cmux's hook-loss issues
confirm that a hook-only replacement is insufficient.

- Reuse the MIT screen detector and verified Orca Codex/Antigravity ready-screen
  predicates. Read the reconstructed current terminal grid, not raw PTY chunks or
  historical output; Codex redraws its header/composer by cell diffs.
- Poll every managed native session independently of MCP registration. Native
  busy/permission/input overlays veto delivery; bare process names and missing
  rules do not establish an automatic idle lease. Keep screen text memory-only,
  bounded and secret-obfuscated; never send it over IPC or retain it in logs.
- Fuse observations in the shared Broker with native run/input-generation guards.
  Do not repeatedly reset stable idle timestamps or invalidate claimed wakes.
  MCP and panel must report the same resulting lifecycle.
- WSL uses the host's same grid detector and its authenticated guest-native route,
  transmitting only the derived activity and exact guest run, with capability
  negotiation and old-service fallback. Preserve default SSH service behavior.
- Validate every CLIAgent mapping, ready/busy/permission/error-return fixtures,
  stale run/input rejection, draft preservation and actual auto-wake behavior.
  Retain upstream licenses in desktop and Companion packages.
- Git main-only/no-Agent and active-Agent cases must both switch collaboration
  mode without creating a linked worktree. Regress persisted stale admission,
  subdirectory roots and separate-clone isolation alongside lifecycle detection.

- Antigravity shows only Gemini and shared Claude/GPT quotas. Each pool shows
  its five-hour window when detection is enabled, otherwise its weekly window.
  Other providers with five-hour windows use the same exclusive setting;
  sources without that window retain their reported quota/balance data.
  Preserve reported values, reset times and the existing account setting.
- Bare interactive launches, including Codex permission/model options, become
  idle at native MCP discovery. Startup menu acknowledgements and draft edits
  must not consume that lease. Initial prompts, real submissions, approvals,
  cancellation and replaced runs still prevent automatic execution.
- Sidebar and collaboration panel use the same observed lifecycle. Idle/startup
  must not be represented as running work or generate completion notifications.
- App-owned composer drafts and attachments remain in memory while queued peer
  work is submitted independently through the existing PTY pipeline. Automatic
  submissions must not trigger composer auto-dismiss/clear. Owner correction:
  native terminal drafts, including unknown history/cursor edits, must not block
  queued work. On 2026-10-13 the owner explicitly authorized discarding native
  unsent input. Clear it at the idle prompt before staging the wake instruction.
- Preserve run/generation fences, delayed Enter cancellation, private draft data,
  task receipts and MCP acknowledgement semantics. No personal configuration edits.

## Plan and validation

1. Reuse provider parsing/cache and centralize visible window selection; add
   representative five-bucket tests for both setting values.
2. Repair shared startup evidence and Codex argument classification; regress
   startup acknowledgements, nonempty submissions and every managed program.
3. Project broker activity into the existing session model without feeding
   presentation updates back into lifecycle evidence. Preserve native hooks.
4. Permit app-owned drafts during wake and preserve them in the shared submit
   pipeline; test saved/open drafts, attachments, approvals and stale Enter.
5. Run local Rustfmt parsing, diff and locale checks. Run focused Rust tests,
   both desktop builds and native capture on GitHub per project verification rules.

## Additional owner reports (2026-10-13)

- Language Restart closes the application without reopening it. Investigate the
  existing autoupdate relaunch path and verify ordinary restart independently.
- Windows WSL Companion installation reports service verification failure despite
  a logged-in Warpai WSL session. Capture the exact failed phase, exit code and
  stderr before changing production code. An unavailable diagnostic launcher
  does not establish a missing Companion or the suspected nested-runtime failure.
- Diagnostic v4 checks the Windows bundle checksum and direct WSL account/HOME,
  gates later checks on confirmed executable availability, compares empty/nonempty
  read-only scans, and reproduces the application's current post-install command.
  Preserve actual nonsecret stderr; omit setup configuration JSON and credentials.
  No personal installation or configuration mutation is authorized by diagnostics.

## Confirmed WSL failure and SSH audit

Owner diagnostic v4 on 2026-10-13: the Windows bundle checksum, Ubuntu WSL 2
account, HOME, shell, installed Companion 5.0.0, and Git runtime all pass. Empty
Agent discovery succeeds. Discovery with installed Agents and the exact application
post-install command both exit 101 with a nested Tokio runtime panic at
crates/agent_bus/src/setup.rs:136. Installation files are already present; the
failure is the post-install configuration scan, not missing account authentication.

Move WSL setup and recovery cleanup to spawn_blocking at the async executable
entrypoint. Keep ordinary uninstall's bounded synchronous cleanup on that worker
pool as well. Add a real subprocess test with a synthetic installed Agent and
nonempty recovery scan. Extend Windows bundled installation acceptance to include
an installed fixture CLI; the previous empty-scan test did not exercise this fault.

SSH audit: Companion control handling (companion.rs), MCP discovery/tool calls
(mcp.rs), and broker request execution (transport.rs) already use spawn_blocking.
Broker runtimes run on dedicated native threads. SSH Codex launch probing uses
Tokio child processes directly. The synchronous setup scanner is reached by WSL
setup/recovery, not by the SSH executable's production entrypoints. No equivalent
SSH production defect was found in this source audit; verify all three Companion
platforms and native process/MCP regressions in cloud acceptance.

Language Restart now requests ordinary relaunch, preserving cancellation. Reuse
the existing process wrapper on Windows so children survive parent shutdown; on
macOS wait for the old PID to exit and pass the exact bundle path as a positional
argument. Never require an updater installer or mark a language restart as an update.

Release packaging is authorized by the owner on 2026-10-13 under
specs/RELEASE.md: Warpai 1.6.4 with Companion 5.1.0. Record cloud verification
receipts after source-matched desktop, WSL and SSH tests complete.

## Windows background console audit

Owner reports brief console flashes during WSL/Agent background operations.
The shared setup runner (`setup.rs`, used by CLI discovery and MCP configuration)
and Companion Git runtime / Codex capability probes used raw process commands
without CREATE_NO_WINDOW. Apply that existing platform flag at these three sites.
Windows-to-WSL, SSH/SFTP, usage polling, shell metadata, Node and LSP commands
already use the hidden-window wrappers or explicit flags. Foreground Agent launch
and ConPTY creation remain interactive; detached Companion services already run
without an attached console. No process dependency or launch abstraction is added.

Acceptance: run an actual Windows configuration child through the shared runner
and assert its console window is not visible, preserving captured output and exit status. Run
the setup suite in remote validation as well as the complete protocol suite.
Existing readiness regression must observe the worker from a second registered
peer: AgentList deliberately excludes the caller itself. This corrects the failing
test fixture without changing the production visibility contract.

Cloud Windows regression initially rejected code page 437 despite CREATE_NO_WINDOW:
console code pages do not establish window visibility. Check GetConsoleWindow with
IsWindowVisible instead, including null or invisible message-only window handles.

The desktop wake fixture replaces the entire session to simulate permission
states; set_session deliberately closes the composer and emits Ended. Compose the
saved draft after these fixture replacements, immediately before automatic wake,
so the draft preservation assertion tests delivery rather than session teardown.
Wait for InputSessionChanged's reactive editor restoration before typing the
fixture draft; inserting text in the same update as open races the opening clear.
Limit this test's CI logger to errors so assertion output is not buried in per-window
asset warnings. Preserve permission blocking and stale delayed-Enter assertions.

## Native draft delivery correction

Remove native draft presence from candidate/claim/validation and displayed delivery
blockers. Keep draft contents private and memory-only; composer drafts remain intact.
A claimed wake may discard its native draft tracking and sends one Ctrl+C to the
idle native editor, waits for its input handler, then stages the fixed wake text.
Do not clear an empty native prompt or inject Ctrl+C into bracketed paste. Recheck
run, approval, cancellation and input epochs after clearing and before delayed
Enter. WSL uses the same clearing path and retains its guest-owned claim/validation.
The editing shortcuts of a third-party CLI are a native runtime dependency;
validate default interactive modes, multiline/history input, and unchanged drafts
in Warpai's composer. Do not use repeated interrupts or stop a working Agent.

### Native observation implementation checkpoint

Source attribution and coverage limits: [AGENT-STATUS-SOURCES](../../docs/AGENT-STATUS-SOURCES.md).

The MIT AoE engine/manifests are ported without tmux/runtime configuration, with
its no-match idle fallback changed to unknown. Orca's Codex/agy composer, Pi/OMP
state marker and Qoder ready-body predicates complement the rules. Native local
polling captures the current grid with forced secret obfuscation, never logs or
persists it, and fences run/input/output before applying derived activity.
MCP, panel and sidebar project this shared Broker state. Stable samples preserve
readiness timestamps, generations and claimed wakes. WSL capability negotiation
keeps older request/response shapes; edits fence projection and exact guest runs
fence observation before mutations. SSH transport behavior is unchanged.

Coverage must distinguish shared plumbing from vendor signals: all managed
programs/aliases enter the shared grid/native-title path, while only verified
manifest/composer/title shapes can supply positive evidence. Unknown custom
chrome retains native hook/MCP state rather than inventing completion. Tests of
synthetic explicit titles are not live-vendor acceptance. Required tests include
all-program input/redraw/restart fencing and idle wake stability, Codex/agy
welcome/error-return/busy/menu shapes, WSL stale-run and legacy serialization,
and stale-pane projection of main-only Git mode after repository replacement.
Both desktop and Companion installers retain AoE and Orca MIT licenses.

### Shared consumers and legacy interface audit

Owner requires MCP task transfer/supervision, shared sidebar supervision and keep
awake. Audit every CLIAgentSessionsModelEvent consumer and every readiness/task
projection rather than changing only collaboration panel labels. The sidebar's
existing notification mailbox is a session projection, not another task store.
Use the existing MCP Store for assignments, attempts, decisions and receipts;
project unacknowledged work into the sidebar without acknowledging on UI focus.
Typed native observations must not feed opaque notifications back into readiness
or the retained cloud TaskStatusSyncModel. Keep awake must prefer the verified
native activity over an opaque Stop success. Verify actual MCP subprocess task
assignment/start/observation/get/ack/fail for every managed program, stable notice
dedup, busy receiver queueing, run cleanup and independent UI read state.

### 2026-10-14 WSL namespace and shared-consumer follow-up

Owner tested Windows artifact 38393839766 (83c7274b) with Ubuntu, account
`wqin`, home `/home/wqin`. The supplied redacted diagnostic confirms the bundled
checksum, native account/home, installed version, private Git runtime, direct
setup and application login-shell setup all succeed. Closed-input service
startup inside Ubuntu exits zero. Preserve the original diagnostic locally;
do not commit personal diagnostic reports. Windows direct closed-input service startup also exits zero; real desktop
attachment remains open.

- WSL discovery must ignore Windows-installed commands imported into PATH,
  including npm shell shims. Resolve symlinks and use WSL's native `wslpath`
  mapping rather than assume `/mnt/c`; continue to a Linux installation of the
  same name. Apply this to fallback discovery, guest setup, saved participation
  and vendor configuration mutations. Keep ordinary SSH/local setup unchanged.
- A settings rescan must not remove a confirmed remote project selection.
  Successful verification advances the existing configuration generation in
  the shared Explorer/Review/collaboration selection key, releasing cached
  pre-install failures once. Review observes the same settings model; failures
  without explicit rescan/context change remain fenced, with no retry loop.
  Selected-account and WSL2 guards remain required. Files, Review and collaboration
  share the existing native connection. Successful setup now requires an actual
  Initialize/ProjectOpen attachment to the selected account home before claiming
  the component is installed.
- Regression: real WSL process with a Windows-mounted Codex shim, Windows-first
  PATH collision and native-only dsh. Host commands must never be probed or
  configured. The Windows native capture must retain project selection during
  deferred startup scanning.
- Source 4b73cdc2 macOS desktop acceptance passed; Windows native capture failed
  at confirmed WSL selection, and both OS protocol/remote suites failed a new
  test querying AgentList as the agent itself. AgentList deliberately excludes
  its caller: corrected the assertion to use the observer. These failed runs
  (38398097658/38398104664) are not delivery acceptance.
- Shared native activity drives sidebar icons, the existing notification mailbox
  and keep awake. Native idle cannot manufacture task completion; opaque CLI
  notifications cannot claim completion. Pending MCP work remains unacknowledged
  when a tab is focused. Permission/error/idle with an unfinished attempt sends
  a durable observation through the same MCP inbox/wait/task contract.
- Legacy TaskStatusSyncModel remains retained upstream compatibility; local-only
  AI-disabled sessions cannot issue its cloud status RPC. New native observations
  use a separate event and never feed that reporter or plugin completion paths.

Validation: syntax/diff/localization locally; source-matched protocol, desktop,
real MCP and real WSL/native capture on GitHub. No vendor credentials or user
Agent configuration are used in fixtures. Real owner WSL acceptance remains open.

### Companion installation root correction

Owner identified that desktop data moved to `.config/warpai` while SSH and WSL
Companion paths still used `.config/.warpai`. All active native path resolution,
Unix/Windows/WSL installation, uninstallation, recovery, runtime checks, diagnostic
launches, remote fixture runners and current installation docs now use the new
root. The retained remote-server setup path and native worktree template comments
follow the same root. Historical release docs and explicit desktop migration
sources keep their original paths.

Windows Companion installers disable reuse of the previous installation directory;
changing DefaultDirName alone would retain the old registered path. Installer
fixtures verify new-root installation and preservation of former components.
Owned guest TOML MCP entries migrate only when the entire old entry matches the
expected server except for this exact root change; preserve user edits/other MCP
servers. Current SSH/WSL installers own the replacement binary/runtime, with no
personal-machine configuration mutations from development tools.

Validation: Unix and generated WSL staged installers pass with spaces/Unicode,
repeat install, old-root preservation, symlink isolation and corruption rejection.
Rust/cloud desktop and remote acceptance must use the final corrected source.

Source-consumer audit: the retained SDK driver explicitly ignores native activity
events in its exhaustive match; these cannot stop a legacy harness or claim a
legacy task completion. Workspace redraw already consumes SessionUpdated.

### Expanded lifecycle/dispatch acceptance (owner 2026-10-14)

Separate native editor state, agent activity, MCP delivery and durable task state.
An unsent native draft must not block an otherwise idle run with no executing
attempt; clear only during a claimed fenced peer delivery. Preserve app-owned
rich drafts and attachments. Command text never proves completion: `/status`
may run during a live Codex turn (upstream rust-v0.161.0 slash_command.rs).

Acceptance matrix: startup -> idle; idle with ASCII/Unicode/multiline/history
native draft -> one fenced task wake; idle /status,/help,/mcp,/model -> menu hold
or idle when dismissed; working /status -> working; error/warning/quota notice ->
blocked while interactive, idle once actual composer returns; approval -> hold ->
working/idle; cancel -> no automatic wake until resumed user input; task assign ->
start -> submit -> review/retry/cancel with unchanged IDs/revisions; delayed Enter
must reject changed input, run, permission or disconnected transport. Repeated
screens must preserve a claim and not duplicate MCP notices. Task failure and
idle do not silently mark successful completion or release unfinished ownership.

Research references are pinned in docs/AGENT-STATUS-SOURCES.md. Orca distinguishes
its empty paste-ready composer from cursor-backed typed drafts; this product's
owner-authorized clear-before-delivery contract requires both shapes. Adopt its
cursor/frame evidence rather than classify arbitrary prompt-like text as idle.
Verify pure detector sequences, Broker leases/tasks, real MCP and native UI/WSL
on source-matched cloud runners. Credentials are not used by the fixtures.


### Structured context and native Send acceptance (2026-10-14)

- Shared typed resource DTOs and immutable revisions use the existing account Store,
  project authority, receipts and byte budgets. The MCP tools and native resource
  URI reads share validation; no automatic file reads or new persistence service.
- All task notifications carry typed bodies. Native Send uses typed instructions;
  Send/assignment/pool controls select project summaries and pin version/digest.
  SSH/WSL negotiate support and reject unsupported typed sends before transmission.
- Tests cover replay/restart, immutable old versions, changed/foreign/duplicate
  references, task retry pins, all managed/custom MCP programs, native MCP resource
  reads, controlled SSH/WSL resource transfer and native resource selection/drafts.
- The previous cloud failures exposed a DrvFS chmod assumption in the WSL fixture
  and unconsumed previous-task notices in the supervision loop; fixtures now
  respect imported-drive permissions and explicitly handle previous messages.
- Cursor-bound Codex/agy composers supersede historical warning/approval/spinner
  prose. Actual live interrupt rows and active dialogs still block delivery; Codex
  o3/o4 footers use the same composer rule. Native vendor/authenticated acceptance
  is separate from synthetic transport/state fixtures; owner WSL reproduction is
  not declared resolved from static or cloud fixture results.
- Source-matched cloud acceptance and native UI screenshots are required before
  this change is reported as verified; Windows interim builds remain artifact-only.

- Cloud 38446520358/38446523157 exposed a real Worktree notification defect:
  snapshot generation must use the Store's project-scoped internal task read,
  not participant visibility for the synthetic operator. Public reads keep their
  existing authorization. Fixed together with stale fixed tool-count fixtures.
- Windows lib-test compilation exposed App versus AppContext in the selection
  generation regression; sample selection through app.read. Desktop application
  compilation had passed; this failure was in the added acceptance test.

### Interim Windows test artifact and remaining acceptance (2026-10-14)

- Source d379a2dd produced `windows-release-installer` in run 38455056708,
  artifact 11848456523. Windows packaging and bundled WSL verification passed;
  owner Ubuntu/wqin attachment remains a separate reproduction gate.
- Communication 38455051895 and remote 38455054344 stopped on the standalone
  companion fixture's old capability array. Update the fixture to include both
  context capabilities and the WSL-only native activity capability.
- Both native desktop regression jobs reached the physical-row test. Its default
  mock was ten columns, wrapping the header before an assertion about row 1.
  Use an explicit 80-column fixture for that fixed layout; retain real grid row
  extraction, cursor evidence and forced secret obfuscation in production.
- Extend Claude's cursor/frame recognition with its mode footer and Orca's ASCII
  rest/working title prefixes. Live permission hints and busy titles retain their
  guards; an unknown cursor/layout cannot prove an idle composer.
- Resource selectors retain selections across pagination and pool forms load
  remaining resources. Native WSL request processing retrieves pinned context
  through the receiving Agent's own MCP, in addition to host control verification.
- Context summary scans use the existing primary-key prefix range, excluding
  payload versions and non-JSON metadata without a schema migration.

### Owner WSL attachment remains unavailable (2026-10-14)

- Owner reports Explorer, Collaboration and Review still unavailable after install.
  Packaging success does not establish a working owner workspace connection.
- Workspace selection must resolve Companion under the same confirmed distro/user
  home used by installation and setup, rather than re-reading a terminal home.
  Cache identity includes the Companion path so a changed installation cannot
  reuse an old failed connection. Regression covers account isolation and SSH home.
- Two native Windows WSL acceptance environment values still used the old hidden
  root while the fixture installer used the new root. Align both with
  `.config/warpai`; preserve explicit former-root migration fixtures.
- These are source inconsistencies, not a confirmed explanation for the owner's
  latest failure. Owner confirms run 38455056708/d379a2dd and persistent installed
  state. Setup already verifies a real control connection to the account home;
  failure therefore needs current-workspace versus home comparison, not another
  closed-stdin startup check. Owner workspace connection remains an acceptance gate.
- Source 679f23fc was pushed. Communication/WSL/native acceptance 38496547370
  and artifact-only Windows installer 38496549411 were dispatched. Local WSL
  installer self-check, Bash syntax, Rust parsing and diff checks passed; native
  regression execution remains cloud-only and pending.

### Windows-mounted WSL workspace reproduction (2026-10-14)

- Owner confirms `/mnt/c/Users/wqin2` fails while `cd ~` restores Explorer and
  Collaboration. Companion installation/home control verification succeeds.
- Project and worktree directory identity previously required `Metadata::created`
  unconditionally. Rust documents unsupported filesystems as an error; this can
  turn a mounted directory into `ManagedUnavailable` / `CompanionUnavailable`.
  Preserve the existing fingerprint when birth time exists. Only Unsupported
  falls back to native device/file identity plus an explicit domain marker;
  permission and other errors propagate. Shared identity hashing serves project
  attachment and worktree admission. Live handle/replacement guards remain.
- Native identity without birth time cannot distinguish inode reuse after every
  handle closes. This is not path-based admission and does not change account
  fences. See WORKTREE-COLLABORATION.md.
- Real WSL acceptance now opens a disposable Windows temp directory through
  `/mnt`, lists/downloads/saves files, opens Collaboration and reconnects with
  the same identity. Existing tests cover native replacement and account guards.
- Official behavior: https://doc.rust-lang.org/stable/std/fs/struct.Metadata.html#method.created
  Owner filesystem metadata has not yet been inspected directly; the source fix
  targets a verified compatibility defect, and real mounted acceptance is pending.

### Mixed-pane switching and recurring file refresh failure (2026-10-14)

- Owner additionally reports WSL/local Windows pane switching reverses which tools
  work, and a populated Explorer fails after seconds with refresh InvalidInput.
- `expand_ancestors_to_path(root, root)` skipped the root before looking for its
  boundary, so it collected ancestors outside the admitted root. Those paths
  entered the two-second remote poll. Bound traversal to descendants of the root
  for both SSH and WSL; a real FileTree regression checks its expanded folder set.
- Workspace root refresh formerly ran before updating ActiveSession. Move it
  after the focused session changes. Replay existing directory-model roots when
  the coding panel updates: a focus-only change may emit no directory event,
  leaving local roots hidden/removed after remote selection. Observe ActiveSession
  notifications instead of subscribing to an event it never emits.
- Review retains its existing selection/generation reply guards and resets remote
  views on local focus. File previews/editors retain their original RemoteFiles
  source and cache identity; switching focus cannot redirect saves to another host.
- Extend native acceptance with post-poll checks, SSH-to-local Explorer recovery,
  actual Windows/WSL split panes, local Explorer/Review/Communication recovery,
  then guest Communication and preview recovery. No owner configuration is changed.
- Re-audit SSH install/call/remove roots: active POSIX defaults and installer use
  `.config/warpai`; former roots remain only explicit migration/compatibility tests.
  Windows SSH uses `%USERPROFILE%\.config\warpai\bin` and its Inno installer
  disables reuse of the previous install directory. Source checks and native
  captures are pending cloud execution; owner reproduction remains a separate gate.

- Cloud 38495615370/38495617151 exposed Claude's generic live prompt-box rule
  granting idle to a draft without cursor evidence. Require the verified native
  composer for that rule; preserve known completion/empty prompt/title rules.
  Detector rules explicitly requesting no state update now return no observation
  before title fallback, so command menus cannot become idle from a rest title.

- Latest code 7b7b431f: communication/native/WSL validation 38498468161,
  three-platform remote validation 38498468176, Windows artifact-only installer
  38498467650. Installation self-checks, shell syntax, Rust parsing and strict
  locale coverage passed locally. Source-matched native execution is pending.

### CI failure repair plan: source 7b7b431f (2026-10-14)

- Communication 38498468161 protocol jobs fail four screen observation tests.
  `Detection::unknown_by_default` has status None and rule `no_rule`; this must
  allow explicit native composer/title evidence. Only an actual matched rule
  requesting no state update may suppress fallback. Repair the discriminator;
  retain draft, busy, warning, menu and permission regressions.
- Remote 38498468176 fails `reviewed_workspace_is_pinned_and_visible_before_agent_discovery`
  on all three hosts. A revoked reviewed binding must keep its original panel
  domain, not fall back to a private project. Preserve root matching and the
  explicit replaced-repository exception; public authorization remains unchanged.
- Verify with focused cloud screen tests and local_spaces/worktree routing suites,
  then complete three-platform remote and native/WSL acceptance. Reuse existing
  workflows and preserve owner files/configuration. No PR or release tag changes.

- Desktop default check additionally reports E0502 after the focus/root ordering
  repair: retain an owned cloned PaneGroup handle before mutating Workspace.
  This restores the original borrow boundary and keeps the corrected ordering.

### Debug before packaging: source 8cb89d1e (2026-10-14)

- Owner requests stopping packaging until debugging and focused validation.
  Force-cancel submitted for Windows release 38499833429; no new package is
  dispatched until the debug validation succeeds.
- Communication 38499833377: both OS protocol jobs passed. Desktop lib-test
  compilation fails on missing SizeInfo in the physical-grid regression. Import
  the terminal SizeInfo explicitly; keep the 80-column fixture and its assertions.
- Remote 38499833387: Windows native remote shell test fails its version probe.
  Its workflow still exports the former `.config/.warpai` installation path while
  the source-matched installer uses `.config/warpai`. Align the acceptance path.
- Audit every workflow for active former companion paths. Re-run native desktop
  tests/WSL capture and three-platform SSH validation, then package accepted source.
- The macOS release workflow embeds the same Windows SSH acceptance step and
  also retained the former path; correct it without dispatching a release.

### Windows payload build lock failure: 38505805648 (2026-10-14)

- WSL payload compilation and managed Agent integration pass; the build stops
  at `account_lock_rejects_concurrent_setup_and_symlinks` reacquiring after Drop.
- A flock belongs to an open file description. Parallel test subprocess forks
  can briefly retain the description until exec, even with O_CLOEXEC. Closing
  only the parent descriptor therefore need not release the logical setup lock.
- Return a minimal owned preference-lock guard that explicitly unlocks on Drop,
  retaining owner/symlink checks, nonblocking acquisition and the stable lock file.
  Keep File private; callers only hold the guard for the setup scope.
- Deterministically duplicate the acquired description in the regression, drop
  the guard and reacquire before closing the duplicate. This models inherited
  handles without subprocess timing or sleeps; contention must still fail while
  the guard is held. Existing symlink rejection must still pass.
- Run focused real WSL tests through existing source-matched validation and
  retry the explicitly requested artifact-only Windows build after the fix.
- Linux flock reference: https://man7.org/linux/man-pages/man2/flock.2.html.
  Local owned-descriptor Python self-check reproduces close-only retention and
  verifies explicit unlock; Rust parsing/diff checks pass. Real WSL Rust execution
  remains cloud-only. This is consistent with the failed immediate reacquisition;
  the original failing process's descriptor inheritance was not traced directly.
