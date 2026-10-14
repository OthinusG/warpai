# Native Agent status sources

The common desktop detector reads the reconstructed terminal grid (up to 60 rows,
forced secret obfuscation), not raw PTY chunks or historical session files. No
screen/title text is persisted, logged or transferred. Broker observations fence
native run, input generation, redraw and fresh lifecycle events. Repeated samples
preserve idle timestamps and wake claims. Native approval/draft/cancellation guards
remain independent of readiness. Windows WSL transmits only derived activity and
exact guest run after negotiating `native_screen_activity`; SSH is unchanged.

## Reused implementations

- [Agent of Empires](https://github.com/agent-of-empires/agent-of-empires/tree/aa02052f5cf2f12c3972fcd7569e0b650f8e2aa1/src/tmux/detect):
  MIT pure Rust manifest/region engine and bundled agent rules. This port retains
  rule ordering and parser validation, removes the tmux/runtime configuration
  dependency, and changes no-match idle to unknown. [License](AGENT-OF-EMPIRES-LICENSE).
- [Orca Codex predicates](https://github.com/stablyai/orca/blob/38a4958cad8cc75331a8a0a1e7d28b4674b212f7/src/main/runtime/agent-state-rules/codex-screen-predicates.ts),
  [Antigravity composer](https://github.com/stablyai/orca/blob/38a4958cad8cc75331a8a0a1e7d28b4674b212f7/src/main/runtime/agent-state-rules/antigravity.json),
  [Pi/OMP markers](https://github.com/stablyai/orca/blob/38a4958cad8cc75331a8a0a1e7d28b4674b212f7/src/shared/pi-state-title-marker.ts),
  [Qoder composer](https://github.com/stablyai/orca/blob/38a4958cad8cc75331a8a0a1e7d28b4674b212f7/src/main/runtime/qoder-terminal-readiness.ts):
  MIT native shape predicates complement the AoE manifests. [License](ORCA-LICENSE).
- cmux was researched; no cmux source is incorporated.

## Coverage and limits

All managed CLIAgent programs and aliases use the shared native observation and
Broker projection. The bundled AoE profiles cover Claude, Codex, Cursor, Droid,
OpenCode, Gemini, Copilot, Pi, Oh My Pi, Hermes, Vibe and Antigravity.
Explicit native busy/idle/permission titles and existing native hooks/MCP remain
available to the remaining programs. A program name alone never establishes idle;
unknown chrome retains the previous state. This is shared integration coverage,
not a claim that each CLI version supplies every signal.

An empty Codex/agy composer can remain painted briefly during submission. Ready
shapes therefore require a quiet input/output/lifecycle interval and no detected
busy/permission evidence. Quiet alone is insufficient. Disabling native animations,
status/title updates or changing vendor chrome can remove observable evidence;
verify representative CLI versions before claiming live acceptance. The project
never rewrites personal vendor configuration or installs wrappers to obtain it.

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
