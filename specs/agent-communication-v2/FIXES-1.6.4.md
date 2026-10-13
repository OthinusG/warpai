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
