# macOS collaboration and usage corrections

Status: implementation in progress. Requested 2026-10-12; no release requested.

## Scope and acceptance

1. Language changes offer Restart through the existing cancellable relaunch flow.
2. Data usage stays at the bottom of the collaboration viewport, above overlapping
   content, with its independent bounded scroll and fixed heading.
3. Account setup initially offers Sign in and API input at the same level. API
   input hides Sign in, expands credential/context fields and offers Back.
4. Display each reported Antigravity pool separately; never invent quota data.
5. Each usage account has a persistent Detect 5h usage option; preserve existing
   account metadata and use only supported provider usage sources.
6. Peer instructions support correlated results. Busy recipients retain all
   replies in arrival order and consume them when ready or cooperatively waiting.
7. A Git main checkout works without linked worktrees, including mode switching.
8. Agents launched in the main checkout participate using repository identity.
9. Coordinator selection must not mutate the dropdown during its own callback.
10. One Send to Agent entry replaces Message/Assign. Track as task controls the
    existing task/message operation; replies from task details retain task IDs.
11. History actions reuse the normal secondary button style.
12. Worktree removal checks terminals attached to open tabs, not retained views;
    retain all main/dirty/locked/untracked/ignored checkout protections.
13. Worktree entries and their contents are indented under the Worktrees heading.
14. Windows WSL settings discover confirmed logged-in WSL 2 accounts even when
    the distribution was installed/upgraded after Warpai started. Refresh registry
    metadata without logging into arbitrary accounts or weakening guest identity.
    A literal interactive WSL command defaults to the existing native terminal
    integration flow, then discovers its confirmed account without Companion.
    Do not inject bootstrap into background commands or automatically assert an
    account from process names, launch flags or cached preferences.
15. Communication follows the focused terminal, including command-entered SSH
    and WSL sessions. Local, SSH and WSL enablement are independent; pending
    remote integration cannot fall back to the local project or local toggle.
16. Complete Simplified/Traditional Chinese settings, chrome, form defaults and
    helper-provided UI text identified in the 20 screenshots under the supplied
    lang directory. Windows text shaping must load native glyph fallbacks before
    caching lines, including search placeholders and Settings. Preserve terminal
    content, technical identifiers, native typography and layout. Add regression
    coverage for helper literals and both Chinese glyph sets; inspect native
    screenshots for English leakage, tofu, overflow and unchanged controls.

17. Windows WSL Agent detection and management use the verified Linux account,
    guest process and guest MCP configuration. Never borrow Windows executable
    inventory or run identity. Guest observations populate native Agent sessions,
    including manually started programs without a Windows-side command event;
    retain existing plugin listeners and input drafts. Multiple terminals may
    share a distribution/user/project with distinct sessions and runs; closing
    one does not terminate another. Desktop changes are Windows-only; Linux
    changes remain limited to the WSL Companion and controlled guest fixtures.

## Implementation and constraints

Reuse native settings, dropdowns, editors, buttons, layout and Agent Bus operations.
The existing collaboration UI is the visual source: existing fonts/icons/tokens,
8px spacing, 16px worktree indentation. Keep English and both Chinese locales.
No dependency, separate dispatcher, credential migration or CLI configuration edit.
Desktop and SSH share Broker/Store semantics; preserve run/scope fences, request
identity, permission/draft guards and uncertain execution receipts.

## Delivery contract and implementation tasks

Local, SSH and WSL use the same Store operations and MCP schema. Project mode
allows every participant to observe progress, questions, permission waits,
deadlines and uncertain execution, and to decide retry, reassignment or review.
The immediate issuer collects replies within user authorization. Worktree mode keeps the selected Coordinator's
orchestration authority. Nested Project delegation returns to the immediate
issuer, who combines results before submitting upstream.

- Task descriptions carry objective, context, scope, interface constraints,
  acceptance and verification instructions. Workers report changes, evidence
  and remaining work; submission is distinct from reviewer acceptance.
- `agent_send.expects_result` defaults to false for existing notification clients.
  True creates a request requiring a correlated result before acknowledgement.
- `agent_reply` derives its destination from the original message, retains its
  thread/task and records succeeded, failed or blocked. A final result and request
  acknowledgement commit together. Blocked reports leave the request outstanding.
  Result messages do not request another result. Identical retries reuse UUIDs;
  another final reply is rejected rather than executing work twice.
- Busy recipients retain results in arrival order. Native wake selection skips
  already submitted notifications while inbox history retains them until handled;
  an unacknowledged head must not hide later messages. Manual inbox/wait is the
  cooperative path during active work. No approval or draft is overwritten.
- Task submission notifies both issuer and designated reviewer when different.
  Run identity, uncertainty and ownership fences remain authoritative. Restart
  does not imply successful handling or automatically re-execute old work.
- `agent_ask`, `agent_answer` and `agent_questions` persist task-linked decisions.
  Answers derive their destination from the question and UUID replay is stable.
  Project peers share the decision record; task history retains question/answer
  text. Old revisions retire unanswered questions. Submission/review cannot
  bypass unresolved questions. Execution ownership remains run-fenced.
- Task detail reports observation ages, pending answers, execution/review
  deadlines and uncertainty. Silence and timeout do not restart execution.
  Project decisions use current versions; the last valid decision applies,
  while late results from retired attempts remain rejected.

Execute: finish UI corrections; implement the shared contract and correlation;
cover independent Project issuers, Worktree roles, FIFO busy replies, duplicate
retries, failed/blocked outcomes, head-of-line delivery and SSH/WSL tool parity;
run source-matched GitHub desktop/Companion tests and native acceptance.

## Verification

- Focused regressions: standalone main Git checkout; result correlation/FIFO;
  account metadata compatibility and quota fixtures; form operation selection.
- Local: Rustfmt syntax checks, interface locale coverage and diff checks.
- GitHub: focused Rust tests, both application feature checks, native macOS and
  Windows screenshots of empty/full/scrolled usage, settings API/Back, Send/task,
  main checkout/Coordinator selection and expanded History/worktrees.
- Personal credentials and paid model calls are outside validation fixtures.

## Progress

- Repository rules and user crash report inspected. Crash report shows AppKit
  exception termination without a symbolicated application fault frame.
- Footer currently uses Shrinkable, which cannot fill unused viewport height.
- Existing durable messages, reply_to/task_id and FIFO inbox queries are reused.
- First source 88acea81 compiled the communication crate on all three platforms.
  Runs 38197712881 and 38197714902 stopped on the new checkout-change regression:
  registration reused a terminal identity bound to the previous checkout.
  Main-only and multi-program registration passed; UI acceptance was not reached.
  Registration now scopes terminal reuse to workspace identity, preserving old
  tasks/history instead of moving them. The regression also returns to the first
  checkout and verifies that its original identity is restored.
- Source 2a11a387 adds shared Project supervision, durable questions/answers,
  SSH/WSL decision fixtures and independent focused-environment enablement.
  Rustfmt parsing, Python AST, strict locale coverage and diff checks pass locally;
  cloud compilation and native acceptance remain required.
- Source cc8b9b16 passed the checkout identity regression. Subsequent runs
  38199890573 / 38199893051 found outdated Project privacy/reviewer assertions,
  a comparison of dynamic progress age, and a Windows fixture receipt exceeding
  console line width. Tests now assert shared visibility, stable task data and
  bounded fixture fields, retaining attempt identity.
- Follow-up persists native permission/cancellation/disconnect observations and
  issuer attention notices without draft/output content; cooperative waits skip
  prior receipts while retaining the durable inbox. Failed ordinary replies
  retire unresolved questions atomically. Remote forms use the same focused
  environment gate as the panel, covered by native WSL fixture assertions.
- Issue 16 references 20 supplied screenshot crops: settings labels/descriptions,
  usage setup, form defaults/buttons, history paging, toolbar tips, vertical tab
  options, block menus, and Windows search/Settings tofu. The locale scanner now
  detects helper arguments and section metadata; dynamic UI labels translate at
  the interface boundary. Added 53 translation pairs and both-locale regressions.
  Windows shaping loads existing native fallbacks before line/frame caching,
  reusing the font cache and preserving terminal/editor content. Native acceptance
  now also exercises inline Coordinator dropdown selection rather than only the
  separate form; its menu screenshot is retained for review.

- Validation is batched after corrections 1–16. Protocol suites run in independent
  macOS/Windows jobs; desktop jobs build their required native fixtures without
  waiting for the protocol jobs. Companion remains a separate three-platform
  matrix. Both desktops reuse Cargo outputs within each job and across runs;
  default and warp_platform checks remain required. Application modules share
  one Rust crate and cannot independently replace the complete desktop build.
- Runs 38214441645 / 38214444990 passed prior checkout/coordination regressions
  but exposed a readiness fixture that selected the first globally visible task
  on its second iteration. It now locates the current task by ID. Added one
  actual Coordinator-dropdown screenshot and updated native capture counts.

- Source 6eb0b3ad passed macOS/Windows protocol suites and macOS/Windows
  Companion acceptance. Desktop checks found an EventContext dispatch mismatch
  for Restart and a Windows-only font cache excluded by an older cfg; both are
  corrected. Controlled Linux SSH found fixture receipts omitted correlation
  fields; compact receipts now retain routing, result kind and message IDs.
  Failed desktop attempts also save partial Cargo outputs for incremental retries.
- WSL supervision now seeds native terminal Agent sessions from the verified
  guest program/run before any queued wake. Existing command/plugin sessions and
  drafts are preserved; guest exit removes only observation-owned sessions.
  Added Windows regressions for two independent terminal Agent observations and
  two real PTYs sharing one WSL distribution/user/project. No new WSL desktop
  code is active on macOS. Usage footer statuses/tooltips and provider errors
  now translate at display boundaries, preserving account names and quota data.

- Source 376b17ba passed both protocol suites and all three Companion platforms
  (runs 38220306904 / 38220310879). Desktop application checks passed; Windows
  test compilation stopped on two stale GPU AdapterInfo fixtures, now aligned
  with locked wgpu fields. macOS native capture timed out at Agent projection;
  cached hashed test executables could shadow the current fixture. Native jobs
  now delete only the two small fixture outputs before rebuilding them, retaining
  application/dependency caches. Projection acceptance remains unverified.
- The user confirmed account/Agent discovery works after manually enabling WSL
  terminal integration and requested automatic entry. Literal interactive WSL
  launches now reuse native bootstrap by default, respecting the existing denylist
  and Agent-requested-command exclusion. User input/cancellation, block replacement
  and an already confirmed WSL session invalidate delayed bootstrap. Administrative,
  exec, operator, shadowed and SSH commands cannot trigger it. Windows Bash InitShell
  now reports WSL identity before full bootstrap, matching Zsh/Fish. Added focused
  parser/bootstrap regressions; account installation still needs confirmed identity.
- The user reported recurring Linux console flashes after WSL integration. Native
  background Companion/SSH/SFTP commands lacked CREATE_NO_WINDOW; it now applies
  only on Windows, preserving actual Agent PTYs. Failed WSL connections remain
  unavailable until an explicit settings retry or context change; live connections
  retain normal 1s observation polling.
  The automatic-entry and console corrections still need cloud/native acceptance.

### Transparent WSL installation and remote file enablement

- Show a native modal when installing/removing the selected account's Companion.
  Display the account, elapsed time, actual phase history and transferred archive
  bytes; retain the final receipt after success/failure. Closing the modal leaves
  the operation running and its progress can be reopened. No invented percentage,
  guest logs, credentials or additional dependencies.
- Bound each installation and guest-verification process to 180 seconds. Use a non-login shell so account startup
  scripts cannot consume archive stdin or wait for interactive input. Verify the
  bundled checksum and guest identity before installation, then verify guest setup.
  Report timeout/exit status and failed phase; do not retry partial mutations.
- File Explorer and Collaborate must use the same confirmed remote selection.
  WSL communication off prevents Companion connection/polling, including pending
  callbacks. SSH remains independently scoped to confirmed SSH integration.
- Verify controlled phase parsing, package reinstall/corruption/SSH isolation,
  Windows bundled installation and native progress/error/closed-toggle captures.
- Local checks passed: changed Rust sources parse, workflow YAML parses, Unix/WSL
  disposable package installation/reinstallation/corruption/isolation checks pass.
  UI captures and Rust compile/tests remain cloud gates; representative progress
  screenshots are explicitly fixtures, separate from real bundled WSL installation.

## Final requirement audit

Implementation is present for the following requirements. Completion still requires
source-matched desktop builds, focused tests and native capture review; a prior
commit's passing checks do not certify later changes. No personal vendor accounts
or paid model calls are used in acceptance.

| Requirement | Implementation | Required verification |
| --- | --- | --- |
| 1. Restart after language change | Appearance language widget uses existing cancellable restart action | Native language selection/restart capture and both desktop feature checks |
| 2. Fixed usage footer | `panel_usage.rs` expanded viewport with independent bounded scroll and fixed heading | Empty/full/scrolled usage captures at narrow/wide sizes and zoom levels |
| 3. API entry and Back | Usage account form reuses existing Sign in/API operations, hiding Sign in during API entry | Native API entry and Back captures |
| 4. Antigravity pools | Provider parsing retains Gemini and third-party pools separately | `antigravity_keeps_gemini_and_third_party_pools_separate` |
| 5. Detect 5h | Persistent per-account option, backward-compatible default and pool filtering | Usage account/provider regressions and settings captures |
| 6. Result routing and busy FIFO | Shared Store correlated replies, durable arrival-order inbox, native guarded wake and cooperative wait | Result correlation/reopen/FIFO, duplicate reply, blocked/failed and readiness suites on both OSes; controlled SSH/WSL |
| 7. Main-only Worktree mode | Repository-scoped enrollment accepts the main checkout without linked worktrees | `main_only_checkout_registers_each_program_and_switches_modes` and native mode selection |
| 8. Manually launched agents | Execution adapters and checkout-scoped terminal identity retain all supported CLI programs | Multi-program main-checkout registration, checkout-switch and native activity regressions |
| 9. Coordinator crash | Dropdown callback defers Coordinator mutation through existing action dispatch | Inline native dropdown selection, same-process survival and scoped handoff regressions |
| 10. Unified Send to Agent | One entry with Track as task dispatches existing message/task operations; task replies retain IDs | Native ordinary/tracked Send captures and task correlation tests |
| 11. History button style | History Back and paging reuse Secondary controls | Expanded History capture review |
| 12. Worktree removal | Workspace passes only terminals in open tabs; Git helper preserves checkout/branch protections | `remove_worktree_preserves_protected_checkouts_and_branch`; audit open-tab enumeration |
| 13. Worktree indentation | Existing native containers use 16px indentation beneath heading | Expanded worktree capture review |
| 14. WSL account discovery | Confirmed guest account and refreshed WSL registry metadata; automatic integration for literal interactive launches | Account/registry/bootstrap regressions and real WSL settings capture |
| 15. Focused remote enablement | Shared confirmed session selection; independent local/SSH/WSL gates, including command-entered contexts | Native local/remote disabled-toggle assertions and remote form/file selection |
| 16. Chinese and Windows glyphs | Display-boundary translations, helper-aware strict catalog audit and native Windows fallback before shaping/cache | Strict locale coverage, Hans/Hant glyph regressions and three-language native capture review |
| 17. Guest WSL agent management | Verified guest observations populate native sessions; terminal/run identities remain independent | Guest observation regressions and two real PTYs in one WSL account/project |
| Project peer supervision | Durable task observations/progress/questions/answers and shared versioned retry/reassignment/review decisions | Peer visibility, exact asker, UUID replay, retired questions, uncertainty/deadline and run-fence suites |
| Primary-agent delegation | Immediate issuer receives nested results; detailed existing instruction schema and reviewer acceptance preserve delivery ownership | Coordinator/worker/peer-review/integration and independent Project issuer fixtures |
| Local/SSH/WSL parity | Existing shared Broker/Store/MCP contract rather than separate dispatchers | Both desktop protocol suites and three-platform Companion suites, controlled SSH and real WSL acceptance |
| WSL console flashes/relaunch | Windows background CREATE_NO_WINDOW and unavailable-service latch until explicit retry/context change | Windows build, unavailable-service regression and real guest lifecycle fixtures |
| Transparent Companion installation | Native progress modal, fixed phases, real bytes/elapsed time, bounded child processes and persistent final receipt | Fixed-phase parser, disposable package install/reinstall/corruption/SSH isolation, real bundled WSL installation and progress/error captures |
| Remote File Explorer | Same confirmed communication selection as Collaborate; disabled/stale callbacks cannot reconnect; failed SSH/WSL connections require explicit retry | Native disabled-toggle and live SSH/WSL file/editor acceptance |
| Cloud errors | Locked GPU AdapterInfo fixtures corrected; rebuild only small hashed native fixture binaries; preserve diagnostic projection assertions | Final-source Windows test build and macOS/Windows native Agent projection acceptance |

The final audit also localized participant fallbacks, the inline Coordinator role,
attempt outcomes and local evidence controls. Agent names and message content are
preserved. Existing History Back used Text style despite Secondary paging controls;
it now shares their style. Native projection acceptance remains a required gate.
WSL delivery completion also checks the configuration generation and exact client
before clearing connection state. A failed Finish request latches unavailable just
like a failed observation; stale completions cannot overwrite an explicit retry.

### Installation failure recovery acceptance

The user identified installation failure followed by an unusable uninstall path as
blocking all real-machine validation. An unknown installation state must leave both
install and explicit uninstall available when the confirmed account is not busy.
Uninstall extracts the checksum-verified bundled helper instead of executing the
possibly missing/corrupt installed binary. Its private `recover-uninstall` operation
accepts only a RemoveAll setup request, revokes/removes recorded guest MCP bindings
through existing adapters, and removes only the fixed account-owned WSL component
namespace. No arbitrary target path is accepted. Preserve SSH, projects, task data,
shell profiles and unrelated installation entries; reject symlinked directories,
executables, manifests and runtime roots. Recovery may remove damaged recognized
runtime metadata, while ordinary self-uninstall retains its checksum requirement.

Release native clients during settings mutations and allow at most 65 seconds for
Companion's existing idle shutdown. Active sessions remain protected; no process is
forcibly terminated. Report failure and retain progress rather than pretending that
configuration cleanup or removal succeeded. All subprocesses remain bounded.

Expand real bundled Windows/WSL acceptance to install, reinstall, uninstall,
repeat uninstall, corrupt both binary/runtime metadata, uninstall, reinstall,
verify guest setup, remove the installed binary, uninstall, reinstall and uninstall
again. Verify the SSH fixture and empty WSL component directory after this cycle.
The progress modal must expose cleanup and waiting-for-disconnect stages.

Intermediate source f4426455 passed both OS protocol suites, all three Companion
platforms and macOS default/platform checks plus focused application tests
(runs 38246317180 / 38246318731). Windows stopped on the progress view's Timer::new
call; the pinned async-io Timer uses Timer::after, now corrected. WSL installation
acceptance was skipped behind that compile failure and remains mandatory on the
final source. Native macOS capture was still running at the 30-minute review.
That capture subsequently completed all native assertions, including real Agent
projection, for all three languages. Artifact inspection found 327 nonempty PNGs
per language (981 total); the workflow expected 325 and rejected a complete run.
The exact expected count now includes the two omitted shared captures on both OSes,
and failure messages report the observed count. Source 4f424260 passed Windows
default/platform/release checks, both protocol suites and all Companion platforms;
Windows application test compilation found four test-only App/AppContext mismatches
in the two-terminal guest regression. Those reads now use App::update's AppContext.
Real WSL installation/removal acceptance remains gated on the next Windows run.

Run 38249733951 also failed the Traditional Chinese native assertion `native tab
and persistent Coordinator`; English and Simplified Chinese passed that step.
The prior source completed the same assertions in all three languages. Preserve
the assertion and its timeout. Failure diagnostics now record each creation
predicate, tab count and modal presence as bounded counts/booleans, plus existing
Git exit categories. The diagnostic self-check rejects runtime payloads. A new
receipt is required before assigning a root cause to this native failure.

### Desktop without system Git

Windows desktop installations currently use system Git; Companion packages own
their private Git runtime. Missing desktop Git must be reported as a missing
capability, not invalid user input or a stale connection. Preserve other Git error
categories and repository identity checks. Show localized installation guidance
for repository collaboration. Verify in an isolated child process with an empty
PATH, without changing the test runner's environment or installing tools.

Source 528ee0a1 passed both OS protocol suites, including missing-Git child-process
acceptance, and all three Companion platform jobs (38260398074 / 38260403582).
The prior complete run 38255516864 passed both desktop builds, focused application
tests, real Windows-to-WSL workspace checks and bundled installation/removal
recovery. Native captures failed: Windows before the first step at the framework's
missing-singleton panic; macOS at remote Agent projection. The capture parser had
omitted existing projection receipts. Include those booleans and only the static
Rust type name from the exact framework panic; never publish arbitrary payloads.
Retain projection/Coordinator assertions and require another native acceptance.

Run 38264904379 repeated both capture failures, while protocol, desktop build/test
and real WSL installation/removal gates passed; Companion run 38264909535 passed
all platforms. macOS projection receipt shows disconnected with no snapshot.
The debug attachment helper called refresh before installing its transport;
context normalization cleared its failed/waiting latch, allowing a competing SSH
connection. Hold the existing in-flight guard during normalization and restore the
waiting latch until the explicit native transport is attached. Preserve assertions.
Windows's static type receipt must also accept workspace-library model types,
not only the main application crate. Keep exact framework-panic matching.
