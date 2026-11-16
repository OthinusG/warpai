# Stable terminal top position

## Bug and acceptance

The owner's November 11 screenshots show the initial terminal input moving up
when the right sidebar is opened. Screenshot 2 (right sidebar closed) is the
position reference. In vertical tabs mode, opening or closing collaboration,
Explorer or code review must preserve the terminal's top position and input
height. Reopening controls must remain available when either column is closed.

## Implementation

`Workspace::render_vertical_columns` currently omits the center toolbar row
when both side columns are open. Always reserve that row using the larger of
the existing left/right toolbar heights, including native window-control space.
Keep the existing 6px top, 8px bottom and 18px horizontal toolbar padding.
Only render a side's buttons in the center while its column is closed.

Scope: workspace layout and this receipt; no input/editor, API, assets, theme,
horizontal-tabs, panel width or persistence changes. No new dependencies.

## Verification

- Check all four left/right column visibility combinations keep the same center
  toolbar height, including right-side native window controls.
- Parse the changed Rust file and check whitespace locally.
- Native build and screenshot QA use GitHub per project policy. Compare terminal
  top, input height, toolbar access, sidebar geometry and theme across closed,
  collaboration, Explorer and review states; preserve horizontal-tabs layout.
- Runtime screenshots and application compilation remain pending.

## Local result (2026-11-11)

Rust parsing, formatting of the changed block and `git diff --check` passed.
A source/geometry self-check confirmed unconditional toolbar reservation and
equal calculated terminal insets for all four column states: 54px with left-side
native window controls, 88px with right-side controls. This is a static check,
not a rendered screenshot or application test. The branch starts at fetched
`origin/main` (`a70280b4c`); the original dirty worktree is untouched.
