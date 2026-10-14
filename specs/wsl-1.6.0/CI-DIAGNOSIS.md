# Windows CI diagnosis

## 2026-10-14 follow-up: run 38531256385

Source 010a9090 passed both protocol suites and both desktop configurations.
Real WSL acceptance failed before native MCP registration: its Python fixture
called `Path.parent` as a function. Mounted-directory save separately returned
PermissionDenied. macOS native capture reached literal message search, where
legacy string assertions did not account for structured task-linked messages.

Fix plan and scope:
1. Correct the fixture property access and execute its temporary-directory setup
   with a representative Path locally; syntax checks alone missed this error.
2. Assert the exact instruction and task association in structured native search
   and thread results; keep message transport and history immutable.
3. Preserve atomic save and mode preservation. On Unix, skip chmod when the
   newly created file already has the requested mode. DrvFs can expose inherited
   Windows modes without granting Linux chmod ownership. Continue propagating
   permission failures when a mode change is actually required. Existing Unix
   mode-preservation tests and real mounted-directory save exercise both paths.
4. Rerun source-matched native/WSL acceptance, including `/mnt/c/Users/`, then
   both desktops before packaging. This save correction remains unconfirmed on
   the owner's machine; do not classify CI fixture repair as owner acceptance.

Monitor active runs at 30-minute intervals until completion. A dispatched run
does not complete verification, and long builds alone do not justify a restart.

Run 38539223394 / 07ec34f0 passed the mounted-directory test, including save
and direct `/mnt/c/Users/` attachment. Native MCP registration and same-project
memory/resource reads passed; the foreign-account resource isolation assertion
failed at wsl_workspace.rs:164 with StaleAttachment. A missing reference returns
the Store's legitimate `not_found` domain error, absent from the shared remote
response decoder's allowlist. The decoder rejects it as malformed and closes
the connection. SSH on macOS/Windows/Linux uses this same code.

Correction: admit `not_found` in the existing bounded/sanitized error decoder,
retain unknown-code rejection, and test message redaction, non-retryability and
version removal. Assert exact `not_found` and continued panel availability after
the foreign WSL read, rather than accepting any failure. Cancel obsolete native
validation and rerun both remote Companion and full native/WSL acceptance.

Run: https://github.com/OthinusG/warpai/actions/runs/38051265508
Source: ca2b4f657bb20fa899c3ff3a1d6a592d6c0ca9b4

## Failure and impact

The Windows default application check fails with E0603 at
app/src/agent_communication/wsl_settings.rs:352. Its wsl_command helper calls
warp_agent_bus::session::without_terminal_binding, which is declared pub(crate)
in crates/agent_bus/src/session.rs:30. The session module is already public;
the function's crate visibility prevents the application's external call.

Protocol, persistence, native socket/MCP tests and history checks passed.
remote_server passed all 20 unit tests. The platform application check, WSL2
provisioning, guest build, workspace assertions, bundled installation assertions
and native captures were not reached. macOS was skipped in this focused run.
Warnings are not the terminating error.

## Minimal fix plan

1. Following fix-errors guidance, change the existing function from pub(crate)
   to pub, with a brief public API comment explaining the isolation requirement.
   Keep its environment removal behavior and every caller intact. Do not remove
   the WSL call or duplicate the environment list.
2. Scope: the existing helper, its external WSL caller and existing CI gates.
   Internal SSH, companion and native PTY callers need no behavior changes.
   This is a Rust API visibility correction, with no wire/schema change.
3. Verify the focused diff and Rustfmt parsing locally; run Rust checks/tests
   only on GitHub under project policy. The Windows application build exercises
   this external call directly, so a duplicate visibility unit test is unnecessary.
4. Run source-matched Windows default and warp_platform checks, protocol/MCP and
   remote_server tests, then actual WSL workspace/install assertions and captures.
   Inspect captures and assertion results. Before delivery, complete both OS
   acceptance from the corrected source; a dispatched run is not acceptance.

## Completion criteria

E0603 is absent and both Windows application configurations compile. Existing
SSH regressions remain green, WSL executable assertions pass, and source-matched
native captures pass inspection. No release or merge is justified by this failed
run.

## Applied correction

The existing cleanup function is public with its environment removal behavior
unchanged. Internal callers and the WSL settings caller reuse the same helper.
Cloud acceptance remains pending until actual build, test and capture results
have been inspected. Monitor the source-matched run every 30 minutes.

## Archived guest packaging correction

Run 38052636231 at 54b02dc7 passes Windows default/platform checks and focused
application tests. WSL builds reach packaging, which fails because package-wsl.py
calls git rev-parse HEAD in an archived source directory without .git.

Resolve the SHA in the runner checkout, archive that exact revision and pass it
explicitly through the guest build to package-wsl.py. Validate the full commit SHA
before packaging and keep it in the checksummed manifest. Both validation and
release callers use verify-wsl-workspace.sh and receive the same correction.
The Python regression checks manifest provenance without a Git subprocess and
rejects malformed SHAs; existing installation/isolation tests remain required.
Rerun complete cloud acceptance from the corrected source.

## Native guest registration diagnosis

Run 38055377836 at e00caf1f passes the complete macOS job, both Windows
application configurations and guest packaging. The real WSL workspace test
fails waiting for native-session.json at ordinary guest MCP registration.
The fixture and test previously discarded child stderr/stdout, obscuring the
failed registration guard. Capture bounded owned-fixture output and stop waiting
when the child exits; rerun focused WSL acceptance before changing production
admission. Preserve UID, ancestry, foreground and enabled-program checks.

Run 38059372783 at b67189a8 reaches MCP initialization but tools/list fails
during automatic registration. Preserve only the domain error code in MCP error
data, and print that code in the owned fixture. Extend the guest lifecycle test
to register through the actual broker for both Git and non-Git roots; successful
kernel binding alone does not prove MCP registration. Keep diagnostic streams
on one shared file offset to preserve ordering.

Run 38060600725 at 7af5b0f0 reproduces registration failure directly in the
Git-root guest unit test: Git worktree registration unavailable. Its Ubuntu 22.04
guest has system Git 2.34.1, whose worktree list does not support -z. The fixture
copied a bare debug Companion without the packaged private Git, so registration
used the incompatible system executable. Official Git sources confirm the option
is absent in v2.34.1 and present in v2.36.0:
https://github.com/git/git/blob/v2.34.1/builtin/worktree.c#L646
https://github.com/git/git/blob/v2.36.0/builtin/worktree.c#L713

Package and install the guest through the existing generated installer before
acceptance; preserve its verified private Git and manifest. Put that installed
runtime on the debug test subprocess PATH only. Do not weaken worktree discovery
or add a legacy Git fallback. The production guest now exercises the release
payload, and debug guest regressions exercise its pinned Git. Rerun the focused
Git-root registration test and real Windows-to-WSL acceptance, then both OS gates.

Run 38061569038 at 1a136f09 passes Git/non-Git registration and reaches the
final native exit wait after wake acknowledgement. The fixture closes its PTY
master while the output thread remains blocked reading it, so the Agent does
not receive input EOF and Windows waits indefinitely. Send the terminal's EOT
on upstream stdin EOF, reap the shell/Agent, then close the master. Retain the
bounded exit assertion and Coordinator actual-process-exit check; do not kill
the fixture or extend timeouts to hide failed lifecycle behavior.

## Verification receipts

Focused run https://github.com/OthinusG/warpai/actions/runs/38062689536
passes at 16631452523f51436dba71c036c6b13984652b2d, including packaged private
Git, Git/non-Git guest registration and the real Windows-to-WSL workspace test.
The latter covers file conflicts, guest authority, wake/cancel/acknowledgement
and actual native process exit. Full both-OS acceptance uses the same source in
https://github.com/OthinusG/warpai/actions/runs/38063840642.

The full run passes macOS, Windows default/platform builds, actual WSL workspace
acceptance and bundled installation/removal isolation. Windows native capture
produces 274 images, then panics at grid/ansi_handler.rs:186 when the simulated
WSL prompt inputs characters without the required Reset Grid OSC. All six WSL
captures are missing. Mirror the existing SSH banner fixture's on_reset_grid()
after starting the block and before input; retain the production debug assertion.
Verify source parsing locally, then source-matched Windows native captures and
complete both-OS acceptance. Do not classify the other warning source locations
in panic_locations as actual panics.

## WSL collaboration capture selection isolation

Runs 38069558419 (cae4d3be) and 38069772345 (4a39ee08) passed native
Windows builds and captured WSL Explorer/editor, then failed before the WSL
collaboration screenshot. The full run passed macOS. The earlier simulated
native-Companion panel deliberately uses a debug profile override; refresh keeps
that attachment while its target is native-companion-checkpoint. The WSL step
reused that panel without clearing the override, so the actual WSL selection
could not satisfy the guest-connection assertion. Clear only that debug fixture
override before opening WSL collaboration; keep production selection checks and
real guest transport. Add all WSL assertion names to the existing safe diagnostic
allowlist. The 276-image artifact includes both Explorer/editor WSL captures;
no production login or terminal assertion is weakened. Focused cloud rerun pending.

## WSL native attachment wait budget

Full run https://github.com/OthinusG/warpai/actions/runs/38076260347 at
d9dc6067 passes macOS but Windows captures 274 images and times out at
`owned WSL tree populated`. Safe diagnostics show the WSL selection is current,
with no connection error and attachment still pending. The UI driver defaults
to ten seconds, while the real guest installation probe alone allows fifteen
seconds before handshake, registration and file RPCs. Prior runs reached the
Explorer/editor screenshots, consistent with an insufficient cold-start budget.
Give only the six asynchronous WSL capture steps a sixty-second wait budget,
as with existing remote capture steps. Preserve transport timeouts, assertions,
zero retries and the process watchdog. This run did not reach collaboration,
so the earlier override correction still requires cloud verification.

## Aggregate native capture deadline

Run https://github.com/OthinusG/warpai/actions/runs/38088551842 produces
276 screenshots: Explorer and editor both reach the real WSL guest. ZIP creation
times place the editor at 22:29:54 UTC; the process exits at 22:30:17, exactly
300 seconds after launch. The WSL collaboration step therefore gets only about
23 of its declared 60 seconds before the global watchdog exits. No step assertion
failed. Raise the whole-process ceiling to 420 seconds, the parent subprocess
ceiling to 450, and the outer single-locale CI step to 15 minutes to include three
120-second SSH/SFTP checks. Keep RPC bounds, assertions and zero retries.
Diagnostic panic locations must come from actual panic records rather than
compiler warnings; report the watchdog separately. Cloud verification pending.

Windows GUI processes can discard redirected stderr. Persist the source-owned
watchdog marker beside existing checkpoint files, and test diagnostics with the
marker absent from the log. Only the derived deadline boolean is published.

## Static fixture polling fence carried into WSL

Run https://github.com/OthinusG/warpai/actions/runs/38092526979 reaches the
real WSL Explorer/editor (276 images), then fails the guest-Companion connection
assertion after its full step budget; the watchdog does not expire. The usage
layout fixture sets `in_flight = true` to freeze projection reads. Unlike the
earlier transition to live local projection, the later WSL transition does not
clear that synthetic fence, so `refresh` returns before making a guest request.
Clear it beside the existing debug-only native-Companion override reset. Keep
production polling, transport bounds and the real connection assertion unchanged.
Verify Rust parsing, rerun the focused Windows capture, then full both-OS
source-matched acceptance. Apply the same commit to 1.6.2 before its cloud gate.

## Windows release-profile guard — 2026-10-11

Full acceptance 38105942222 passes for 15a29850. macOS/three Companion
release run 38114188371 passes, but Windows run 38116679999 fails with E0599:
`wsl_checkpoint_ready` is Windows-only while its callee `ssh_checkpoint_ready`
is debug/local-fs-only. Give the caller the same debug/local-fs conditions.
Add the exact Windows rlto/release_bundle feature/target check to acceptance;
keep both existing default/platform checks. The draft v1.6.0 is not public.
Do not replace its tag or upload differently sourced Windows assets silently.
Synchronize this correction into 1.6.2, then rerun both desktop/native gates.
