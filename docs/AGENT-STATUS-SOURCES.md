# Native Agent status sources

The common desktop detector reads the reconstructed terminal grid (up to 60 rows,
forced secret obfuscation), not raw PTY chunks or historical session files. No
screen/title text is persisted, logged or transferred. Broker observations fence
native run, input generation, redraw and fresh lifecycle events. Repeated samples
preserve idle timestamps and wake claims. Native approval/draft/cancellation guards
remain independent of readiness. Windows WSL transmits only derived activity and
exact guest run after negotiating `native_screen_activity`. Companion-owned SSH
PTYs reconstruct bounded memory-only grids with vt100; the same rules apply.

## Reused implementations

- [Agent of Empires](https://github.com/agent-of-empires/agent-of-empires/tree/aa02052f5cf2f12c3972fcd7569e0b650f8e2aa1/src/tmux/detect):
  MIT pure Rust manifest/region engine and bundled agent rules. This port retains
  rule ordering and parser validation, removes the tmux/runtime configuration
  dependency. Herdr manifests now supply the principal vendor rules. [License](AGENT-OF-EMPIRES-LICENSE).
- [Orca Codex predicates](https://github.com/stablyai/orca/blob/38a4958cad8cc75331a8a0a1e7d28b4674b212f7/src/main/runtime/agent-state-rules/codex-screen-predicates.ts),
  [Antigravity composer](https://github.com/stablyai/orca/blob/38a4958cad8cc75331a8a0a1e7d28b4674b212f7/src/main/runtime/agent-state-rules/antigravity.json),
  [Pi/OMP markers](https://github.com/stablyai/orca/blob/38a4958cad8cc75331a8a0a1e7d28b4674b212f7/src/shared/pi-state-title-marker.ts),
  [Qoder composer](https://github.com/stablyai/orca/blob/38a4958cad8cc75331a8a0a1e7d28b4674b212f7/src/main/runtime/qoder-terminal-readiness.ts):
  MIT native shape predicates complement the AoE manifests. [License](ORCA-LICENSE).
- [Herdr](https://github.com/herdrdev/herdr/tree/2f3432c6889d7e7cd8a0156a7daefa69d63891b2):
  thirteen relevant manifests, including Antigravity and Qoder. Known clients
  without a matching rule default to Idle after the native quiet window, as
  explicitly authorized on 2026-10-15. [Attribution](HERDR-NOTICE.md).
- cmux was researched; no cmux source is incorporated.

## Coverage and limits

All managed CLIAgent programs and aliases use shared native observation and
Broker projection. Additional profiles cover DeepSeek Harness, Auggie, Goose
and TRAE, using vendor documentation or published source. Waiting states retain
registration and MCP connectivity; they only prevent automatic native submission.
Neither Blocked nor Idle acknowledges messages or completes tasks.

Input/output/run fences and independent approval/draft/cancellation guards remain.
Unknown custom clients retain their prior lifecycle evidence. Vendor version/model
acceptance requires runtime checks; fixtures prove shared matching and delivery
contracts, not every authenticated vendor session.

## Draft and command transition follow-up

Orca's `terminal-composer-draft.ts` at the same pinned revision binds drafts to
visible cursor, prompt and footer/frame evidence. Its paste-ready predicate
intentionally excludes drafts. Warpai instead follows the owner's authorized
clear-before-peer-delivery contract: a cursor-backed composer containing text
can be idle. Strip only that verified composer region from activity matching so
words typed by the user cannot masquerade as approvals or work. Physical grid
rows keep multiline drafts aligned with the cursor; no text crosses WSL IPC.

Codex rust-v0.161.0 `slash_command.rs` permits `/status`, `/warnings`, `/mcp` and
other commands during an active turn. Command text therefore cannot grant idle.
Tests cover return-to-composer, live interrupt/spinner, palettes/approval menus,
Unicode/history/multiline drafts, claimed delivery, changed input, cancellation,
resumption, duplicate claims and unchanged MCP task/message receipts.


Claude also uses a cursor-bound `❯` composer between horizontal rules followed by
its native `⏵`/`⏸` mode footer (`shift+tab`). Orca's pinned `claude.json` documents
`* ` at rest and `. ` while working in ASCII titles. Test historical approval
text inside/above drafts and active selection hints independently. Native status
tests use explicit realistic dimensions so wrapping does not invalidate a fixed
layout fixture; production continues to sample physical rows and actual cursor.
