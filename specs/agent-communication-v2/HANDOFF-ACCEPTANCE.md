# Handoff implementation acceptance

Date: 2026-11-03. Status: implementation acceptance passed; no release or
personal installation changes. Contract: [HANDOFF.md](HANDOFF.md).

## Source and scope

Branch `feat/handoff-briefs` starts at local main `e3fb784`. The independent
`fix/native-system-network` worktree and its uncommitted HTTP changes are excluded.
The implementation covers shared DTOs, reserved immutable resources, task
transactions, pinned sharing/retention, MCP, Companion 5.6.0, SSH/WSL capability
gates, direct-source status projections and native lazy detail/history readers.
No new dependency, model, transcript reader, repository scanner or execution
authority is introduced.

## Static fidelity checkpoint

The existing native Collaboration panel, Appearance tokens and section/button
components are the visual source. Editable fixtures live in panel-fixtures.json;
the implementation remains native Rust UI, not a raster handoff.

Source `2ae139053b3b613da2dc4029f2074c99e2c88d0d`,
[Windows native capture](https://github.com/OthinusG/warpai/actions/runs/41574677649):
1,104 PNGs across English, zh-Hans and zh-Hant; diagnostic exit 0, no failed
assertions/steps or panics. Static screenshots were inspected before live wiring.

| Check | Reference / observed evidence | Result |
| --- | --- | --- |
| Layout | Existing panel; 320/600 widths and 1/1.25 zoom fixtures | Sections remain inside the panel; no overlap |
| Typography | Native heading/body roles; long English decision at dark/320/1.25 | Complete wrapping, readable text, no ellipsis |
| Palette | Shared Appearance light/dark fixtures | Existing backgrounds, contrast and borders preserved |
| Spacing | Existing 8-pixel section/row rhythm | Native spacing and fixed usage footer preserved |
| Copy and states | zh-Hans unavailable/loading and zh-Hant historical fixtures | Missing/not-applicable/explicit-empty remain distinct |
| Focus and drafts | Native terminal beside all inspected fixtures | Original unsent collaboration draft remains intact |

The older macOS static job stopped before screenshots at a subsequently corrected
test compilation error. It is not evidence of a macOS visual pass.

## Final source and verification

Final runtime/UI source: `4ca3f2f2c08b330abc3d5587c6acc4c004c9624e`.
The difference from `0845757cffc73cb24f7481831c44d9aae71ecf47` is exactly two
Chinese heading entries in interface.json. Shared Agent Bus, SSH and WSL code
is unchanged. Later receipt-only commits do not change tested code.

| Gate | Source / run | Result |
| --- | --- | --- |
| Protocol, persistence, native IPC/MCP, focused library/bin Clippy, query budget | `4ca3f2f` · [macOS/Windows](https://github.com/OthinusG/warpai/actions/runs/41620326280) | Both jobs passed |
| Default/platform desktop checks and application tests; native UI | `4ca3f2f` · [macOS/Windows](https://github.com/OthinusG/warpai/actions/runs/41620326280) | Both desktop jobs passed |
| Real direct WSL checkpoint, submit, replay and exact-reference reads | `0845757` · [complete desktop/WSL suite](https://github.com/OthinusG/warpai/actions/runs/41613853838) | All jobs passed |
| Real SSH two-Agent continuation through explicit TaskStart; standalone Companion and installers | `0845757` · [Linux/macOS/Windows](https://github.com/OthinusG/warpai/actions/runs/41613903821) | All three jobs passed |
| Linux x64/ARM64 AppImage, DEB, RPM and Arch packages; installed X11 desktop smoke | `4ca3f2f` · [artifact-only packaging](https://github.com/OthinusG/warpai/actions/runs/41620331662) | Both architectures passed; release attachment skipped |

Legacy compatibility is covered by strict request serialization and scoped
capability-gate tests. Absent optional fields remain omitted; new requests
require handoff_briefs_v1. Companion artifacts report 5.6.0, Agent Bus major 2
and SSH major 1 remain unchanged. Linux packages use test version 1.8.2.1;
no tag, release or personal installation was changed.

## Final artifact receipts

| Artifact | Receipt |
| --- | --- |
| macOS native captures | [13226520671](https://github.com/OthinusG/warpai/actions/runs/41620326280/artifacts/13226520671) |
| Windows native captures | [13227187129](https://github.com/OthinusG/warpai/actions/runs/41620326280/artifacts/13227187129) |
| Linux x64 packages | [13227308477](https://github.com/OthinusG/warpai/actions/runs/41620331662/artifacts/13227308477) |
| Linux ARM64 packages | [13227932525](https://github.com/OthinusG/warpai/actions/runs/41620331662/artifacts/13227932525) |
| Linux x64 / ARM64 desktop smoke | [13228755069](https://github.com/OthinusG/warpai/actions/runs/41620331662/artifacts/13228755069) / [13227763687](https://github.com/OthinusG/warpai/actions/runs/41620331662/artifacts/13227763687) |
| Linux / macOS / Windows Companion installers | [13220836586](https://github.com/OthinusG/warpai/actions/runs/41613903821/artifacts/13220836586) / [13221520315](https://github.com/OthinusG/warpai/actions/runs/41613903821/artifacts/13221520315) / [13220189247](https://github.com/OthinusG/warpai/actions/runs/41613903821/artifacts/13220189247) |

Each final desktop capture archive contains 1,110 PNGs across English, zh-Hans
and zh-Hant. Both diagnostics identify source 4ca3f2f, exit 0, no failed steps,
assertions or panics. Native assertions cover exact-version history, loaded
checkpoint/evidence, Agent-reported not_run with verified=false, late responses
after Back and retained drafts. Representative long/narrow, light/dark,
loading/unavailable/history and loaded handoff frames were visually inspected.
QA found missing Result/Evidence headings; both Chinese translations were
added, all eight handoff headings were checked and final captures re-inspected.
The Linux x64/ARM64 smoke screenshots show the installed desktop and live terminal.

## Corrections and validation limits

The native search regression exposed automatic URI decoration of ordinary peer
instructions. Decoration is now restricted to system task notifications;
a focused test preserves peer text and verifies the review URI. The WSL fixture
mode changes now use their selected scope. Both original failing paths passed
the complete desktop/WSL suite above.

Local changed-Rust parsing, new-module rustfmt, strict locale coverage,
Python/JSON/shell syntax and diff checks passed. Rust builds and native acceptance
ran on GitHub with caches. Repository-wide cargo fmt --check reports existing
differences starting in app/build.rs and codex_launch.rs; unrelated files were
not reformatted. Clippy covers the production library/binaries; an existing
non-octal permission literal blocks all-target Clippy in an unrelated test,
while the full protocol test suite still compiles and runs test targets.

The [draft PR #11](https://github.com/OthinusG/warpai/pull/11) is reviewable.
MEMORY.md was updated within 200 lines / 12 KiB. The original worktree's
independent HTTP changes remain untouched. There are no pending implementation
acceptance gates. Synthetic owned Agent fixtures prove transport, state,
authorization and rendering; they do not claim paid vendor/model authentication
or model comprehension.
