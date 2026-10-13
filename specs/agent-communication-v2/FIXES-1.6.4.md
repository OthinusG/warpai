# 1.6.4 usage and readiness repairs

## Scope and acceptance

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

Public release packaging remains outside this fix. Record cloud verification
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
