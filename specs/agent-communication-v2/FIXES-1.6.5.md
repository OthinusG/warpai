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
