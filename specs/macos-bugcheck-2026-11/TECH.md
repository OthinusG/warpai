# macOS field corrections

## Product contract

Source: owner screenshots in `/Users/wqin/Desktop/bugcheck/{1,2,3,5}` and
2026-11-03 instructions. Preserve native controls, theme colors and assets.
Use the Finder reference only for inset columns, toolbar placement and subtle
shadows; do not copy its buttons. Desktop targets and current terminal core
constraints remain unchanged.

- Align terminal input surfaces, context chips and tab/pane rows with shared
  surface/control radii. Audit horizontal tabs separately.
- Round menu selection backgrounds and submenu shadow geometry.
- Use existing Secondary buttons for Reconnect and history navigation/actions.
- Vertical tabs: permanently remove title search and its setting, ignore the
  legacy config key, place the two toolbar groups inside their side columns,
  reserve native window controls, and add inset rounded sidebar shadows.
  Horizontal layout keeps its toolbar structure. Closed panels must retain a
  way to reopen via the remaining toolbar.
- Audit task assignment, result-bearing messages, panel projection and ordered
  history across local/SSH/WSL. Do not infer task authority from message prose,
  erase old messages, or silently replay prior work.

## Tasks and acceptance

1. Inspect source and screenshots; fix shared consumers and caller overrides.
2. Move vertical toolbar groups into their columns, retaining drag/resize,
   fullscreen window controls and context menus.
3. Reproduce routing/projection differences with focused regression coverage;
   clarify explicit task dispatch at the MCP boundary when necessary.
4. Run Rust syntax/format and diff checks locally. Existing project policy runs
   compilation, protocol/application regressions and native captures on GitHub.
5. Inspect exact-source native screenshots in light/dark, vertical/horizontal,
   menus, collaboration/history and pane/tab states. Record blockers honestly.

## Risk boundaries

No generic Container clipping, renderer changes, new dependencies, personal
config edits, installation, deployment or release. Preserve task revision/run,
request replay, authorization and history/purge guards. Layout must keep
native window buttons, toolbar controls, splitters and terminal content usable.

## Investigation and local checks

The original incident has no recorded tool name/error. Source inspection shows
that AgentSend intentionally never inserts a tasks row, whereas TaskAssign
persists it and queues an assignment atomically. The panel uses operator_tasks;
History merges tasks, messages and events. Treat wrong-tool routing as a possible
cause, not a confirmed diagnosis of that incident. MCP discovery/tool help now
explicitly directs named tasks to assignment; no schema or inference is added.
A Broker transport regression checks task/inbox/history projection, idempotency,
message separation and retention after the recipient disconnects.

The History copy handler had no rendered button. Restore its native Secondary
entry alongside the existing pagination; preserve export scope and contents.

Local checks: changed Rust files parse with rustfmt; git diff --check passes;
strict locale coverage passes. Build, runtime and screenshot acceptance remain
pending the exact-source GitHub run. No personal settings were read or changed.

## Visual ledger (runtime pending)

| Point | Reference | Implementation / acceptance |
| --- | --- | --- |
| Columns | bugcheck/5 Finder left/right surfaces | Existing configurable panels grouped under their side toolbar; vertical only |
| Shadow | bugcheck/5 soft floating edges | Existing Container shadow, alpha 28, blur 12, offset y=2; inset 6 |
| Toolbar | Owner: retain existing button design | Reuse current toolbar renderers; reserve native controls |
| Curvature | bugcheck/1 and /2 mismatched outlines/highlights | Shared 8/12/20 tokens; rounded menu selection and shadow box |
| Palette/assets | Owner: no reference button redesign | Existing theme fills, fonts and icons retained |
| Navigation | bugcheck/3 reconnect | Secondary theme for reconnect and page controls; existing history copy restored |

A native screenshot assertion verifies that the toolbar occupies its sidebar
width and precedes the tab list, rather than spanning the entire window.
