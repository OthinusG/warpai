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
