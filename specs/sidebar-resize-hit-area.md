# Sidebar resize hit areas

Owner report: 2026-11-16. Both sidebar resize targets sit inside their visible
column edges; the resize cursor also flickers and is difficult to acquire.

## Contract and scope

Preserve the accepted column surfaces, 6px left / 12px right horizontal padding,
6px margins, toolbar placement, widths, themes and saved state. Align the
existing five-pixel resize strip with the visible column edge facing the terminal.
Use the existing Resizable offset for tabs, tools and review on either side.
Horizontal layouts retain zero offset. Do not introduce new assets or controls.

The shared Resizable must keep the cursor at the dragbar's own hit-test layer,
refresh it while hovered and reject covered mouse-downs. Dragging preserves the
initial pointer-to-edge distance, including nonzero offsets and clamped bounds.
This prevents snapping on subsequent mouse events. Preserve existing size bounds.

## Verification and delivery

- Add shared resize regressions for all four directions, initial pointer offsets,
  repeated movement, size limits and reversing a clamped drag.
- Run those tests in the existing macOS/Windows communication workflow.
- Parse changed Rust and check diff/locale coverage locally; native compilation
  and rendering run on GitHub under the existing project policy.
- Capture both sidebar edges, hover and drag using the native fixture where
  feasible; report any remaining hardware or runtime verification limits.
- Commit the focused fix, merge into main and start source-matched CI. Preserve
  the owner's dirty documents in the original worktree.
