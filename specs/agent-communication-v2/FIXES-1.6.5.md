# Warpai 1.6.5 repair contract

## Scope and acceptance

- Repair native idle detection, including DeepSeek Harness, and audit every
  managed alias. Follow Herdr: known clients without a matching screen rule
  default to Idle; matching blockers prevent submission. Idle never submits or
  acknowledges a tracked task result. This policy was explicitly approved by
  the owner on 2026-10-15, superseding the earlier no-rule Unknown policy.
- Repair WSL admission for enabled native clients, including Codex, preserving
  distro/account/process/run fences and unrelated MCP configuration.
- Remove repeated native screen extraction. Sample changed terminal evidence
  after input/output settles; preserve input/cancel fences, queued wakes, drafts,
  approvals and task observations. Move blocking projection work off the UI
  executor and Git subprocesses outside the Broker mutex shared by keystrokes.
- Make full participant names copyable in both Project and Worktree panels.
  Remove unread/pending-work yellow dots from vertical terminal tabs only.
- Monitor cloud validation/release runs hourly. Release desktop 1.6.5 through
  existing workflows; use Companion 5.3.0 if shared runtime changes, else 5.2.0.

## Tasks and verification

1. Reproduce detector/WSL failures with isolated fixtures; inspect common callers.
2. Change shared paths, preserving all communication behavior and data contracts.
3. Run focused detector, lifecycle, WSL process/admission and panel clipboard
   checks in cloud CI, both desktop configurations and native UI/WSL acceptance.
4. Inspect source-matched results; fix failures before immutable release tagging.
5. Trigger existing tagged packaging and verify published assets/checksums.

No personal CLI/configuration changes, local Rust builds,
terminal rendering rewrites, task acknowledgement shortcuts or polling-based
permission inference. The current untracked owner diagnostic remains untouched.

## Herdr and remote implementation

Reference: herdrdev/herdr commit 2f3432c6889d7e7cd8a0156a7daefa69d63891b2.
Reuse its thirteen relevant screen manifests with Apache-2.0 notices; retain
native input/output/run fences, contents-version reuse and native lifecycle
authority. Supplement remaining named clients with vendor documentation/source.

SSH-owned PTYs have no reusable screen emulator in the standalone Companion.
Use pinned vt100 0.16.2 for bounded, memory-only ANSI/Unicode/alternate-screen
capture instead of implementing another terminal parser or importing desktop UI
dependencies. It is required for matching the same screen rules remotely.
Keep the original authenticated MCP/Store and communication operations; screen
state is observation, never model/result acknowledgement.

## Findings

The local panel async closure executes synchronous project/Git/SQLite reads;
`operator_panel` runs branch subprocesses while holding the same Broker mutex
used by native input/output. The 250 ms screen sampler repeats unchanged grid
copies. These are source-proven blocking paths; owner Windows latency acceptance
is required to establish the practical improvement.

## Communication and change reuse

Blocked maps to the existing WaitingApproval/WaitingInput activities, not connection
loss or a task outcome. Queued messages retain their IDs and task revisions; return
to Idle only permits guarded native delivery. Add regressions for both blockers,
MCP inbox access while blocked, original-message delivery and unacknowledged receipts.

The Broker revision invalidates empty-queue reuse on communication/lifecycle changes.
Unchanged composer guards and read-only MCP queries no longer wake waiting clients.
SQLite mutation counts still invalidate reuse when a read emits lazy deadline notices.
Forwarded Stores retain polling; pending work retains settling/retry/cancellation checks.
The desktop still observes native input/output stamps without copying unchanged grids.

Explicit managed SSH launches use the existing vendor MCP adapters when inline
configuration is unavailable; serialize setup separately from PTY/Broker locks,
preserve unrelated entries and fail on reserved-entry ownership collisions. WSL
continues to use guest preferences and verified foreground process ancestry.

Initial cloud validation from b8b6fad9: communication 38660923047; Companion
38660926339. Results and later source-matched acceptance remain pending.

Latest complete validation source: 4949fa55; communication run 38662737160,
Companion run 38662740880. Hourly monitoring begins from the latest dispatch;
source-matched results, capture review and immutable publication remain pending.

## CI diagnosis and fix plan (2026-10-15)

Owner requested immediate failure inspection. Both runs on 4949fa55 failed before
native acceptance: E0277 at screen_detection/region.rs (str versus &str), and
Unix test E0412/missing json macro in companion_tasks.rs. Shared control-wire
checks and clippy passed on all three Companion platforms.

Use fix-errors: correct the extra dereference, import JSON types/macros in the
isolated setup test, and inspect new sibling-module test field/macro access.
Keep prior pending queue/deadline repairs. Verify syntax/diff/local scripts, then
rerun the existing complete desktop and Companion workflows on the repaired source.
No local Rust build, personal configuration change or release gate bypass.

Native lifecycle events retain Herdr's full authority for Pi/OMP/OpenCode when
available. A screen fallback cannot overwrite their explicit Working/Blocked/Idle
state; clients without active native lifecycle evidence still use shared screen
rules and known-Agent Idle fallback. Add a regression for this priority.

WSL only reports hook readiness after a confirmed plugin SessionStart/version;
a listener allocated from a command name must not feed screen-inferred state back
as full lifecycle authority. Unconfirmed clients continue native screen matching.
