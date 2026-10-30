# Custom tab icons

## Current contract: pane icons and tab config reuse (2026-10-30)

This section supersedes the tab-scoped ownership described below. The owner
requires independent pane icons, serialized by the existing tab config system.
Publication remains paused; preserve the existing v1.8.0 tag without moving it.

- Store the selection in PaneConfiguration and each leaf snapshot, not TabData.
  Context menus target the clicked pane, or the focused pane for a tab-level menu.
  Changing/resetting one pane must not change its siblings or another tab.
- Reuse the existing bounded asynchronous SVG/PNG discovery and native picker.
  Support bundled icons (`icon = "folder"`) and local files
  (`icon = { local = "my-icon.svg" }`) directly in each `[[panes]]` TOML entry.
  Local values are validated basenames, never arbitrary paths or image bytes.
- Save through tab_config_from_pane_snapshot/write_tab_config; load through
  render_tab_config and PaneTemplateType. Preserve old TOML without icon fields.
  Missing local files retain their selection and use the native fallback until
  refreshed. Reuse the same selection for session restoration and pane transfers.
- Vertical pane rows, focused-session views, summaries and group collages resolve
  each pane's own selection. Preserve Agent > selection > SSH > terminal precedence
  and unchanged horizontal rendering. Keep native focus restoration and locales.
- Keep historical tab-icon database migration intact; new selections live on
  pane leaves. Do not guess ownership for legacy tab-level overrides.

Tasks: move ownership and action target; extend leaf persistence and templates;
round-trip both icon kinds through saved tab configs; update callers, fixtures,
native captures and current product documentation. No new dependencies.

Verification: independent sibling selection/reset, stale pane targeting, SQLite
restore, nested TOML save/load/reopen with both kinds, legacy/no-icon TOML,
invalid local paths and missing-file refresh. Run locale/syntax checks locally;
use existing GitHub default/platform builds, focused Rust tests and native
macOS/Windows captures. Do not build Rust locally or touch personal settings.

## Scope and acceptance

- Add an icon picker to the existing tab context menu, using bundled SVG icons.
- Support Folder, Code, Globe, Cloud, Laptop, Tool, Lightbulb and Notebook, plus
  Default. Selecting Default removes the override.
- Apply the selection to ordinary terminal icons in vertical tabs,
  including compact, expanded, summary and grouped views. Horizontal tabs stay unchanged. Preserve CLI Agent
  branding/status and non-terminal pane icons.
- Store the selection per tab in the existing workspace snapshot/database. Restore
  it after restart; preserve it when moving the whole tab between windows.
- Existing databases migrate with no override. Unknown persisted icon names fall
  back to Default. No new dependencies, Agent detection changes, or herdr/tmux
  integration.

## Technical contract and tasks

1. Define a typed, serialized `TabIcon` with stable snake-case names and bundled
   icon/label mappings. Add `Option<TabIcon>` to tab state and snapshots.
2. Add a nullable `tabs.icon` text column; serialize known values and safely ignore
   unknown values. Extend constructors/fixtures and snapshot/restore paths.
3. Dispatch `SetTabIcon { tab_index, icon }` from the existing menu's icon row;
   bounds-check the target and notify the workspace. Reuse native hover labels,
   accent colors, and interface translations.
4. Thread the icon into vertical pane/summary rendering while leaving horizontal tabs and Agent/status branches unchanged.
5. Verify enum/menu behavior, set/reset isolation, persistence and legacy/unknown
   values. Run locale coverage and focused formatting locally. Use existing
   GitHub build infrastructure for Rust checks/tests and native UI where feasible,
   following MEMORY.md (no local builds or personal app/config replacement).

## Risks and verification

The schema/model column order must agree for Diesel. Missing snapshot fields in
fixtures must be updated. Summary/group collage paths must not hide the override.
Use an isolated database for migration/round-trip tests; do not access personal
terminal history. UI acceptance checks picker fit, hover labels, vertical tab modes,
restore/reset, and Agent badge precedence. This change does not publish a release.

## SSH default icon (2026-10-30)

- User reference: a split purple rounded-square SSH badge with `>_` above `SSH`.
  Approved transformation: circular silhouette, charcoal/near-black split, light
  lettering, transparent exterior. Preserve the reference's two-line composition.
- Bundle the edited RGBA asset at `app/assets/bundled/png/ssh-tab.png`; do not
  tint it or add another background. Render at the existing tab icon bounds.
- Resolve ordinary terminal defaults from native pending/active SSH metadata,
  including legacy SSH; WSL/shared sessions are not SSH merely because remote.
- Precedence: Agent branding/status, manual tab icon, SSH badge, terminal default.
  Leaving the Agent restores SSH; leaving SSH restores the local default.
- Reuse this resolution in vertical, summary and group collage views; horizontal tabs stay unchanged.
  Keep English/Simplified/Traditional menu text and manual reset behavior.
- Verify alpha, circular silhouette, black/gray palette, `>_` and `SSH` placement,
  bundled asset loading and native SSH/Agent transition screenshots. The generated
  asset is the production image, not a screenshot standing in for editable UI.

Asset QA: RGBA 1254×1254; transparent corners; alpha bounds 1095×1097
(circular within 1%); light `>_` and `SSH` retained; neutral dark split palette.
The bundled PNG is the imagegen edit output, with no opaque preview background.

Imagegen prompt: edit the supplied SSH icon into a perfect circular badge with
charcoal left half, near-black right half, off-white rounded `>_` above `SSH`,
generous internal padding and a true transparent exterior; preserve the original
composition and use no purple, shadows, external background or mockup.

## Local SVG/PNG discovery (2026-10-30)

- Discover direct `.svg`/`.png` files (case-insensitive extensions) in the exact
  home-relative `~/.config/warpai/icon` directory; do not recurse or follow symlinks.
- Add a native Local icons submenu with image previews and filenames. Keep
  English, Simplified and Traditional Chinese interface labels. An absent or empty
  folder is a normal empty state. Refresh on startup and whenever a tab/pane menu
  opens, asynchronously, without filesystem reads on the render path.
- Persist only the basename as `TabIcon::Local`; keep validated image sources in
  memory. On restart refresh the source from that directory. Reject traversal and
  unsupported files. A deleted/invalid file falls back to the SSH/local default;
  re-adding a valid file with the same name restores the selection on refresh.
- Bound scanning (1,024 directory entries / 128 icons), source files (4 MiB),
  dimensions (4,096 px) and total encoded data (16 MiB). Validate PNGs and cache
  at most 256 px previews without changing original files. Parse and normalize
  SVGs with the existing renderer while disabling external image-file resolution.
  Feed validated bytes through the existing async asset cache, retaining colors,
  alpha and vector scaling. Do not add a file watcher, import dialog or dependency.
- Retain Agent > selected local/bundled icon > SSH > terminal precedence in all
  vertical modes; leave horizontal rendering unchanged.
- Verify supported/uppercase extensions, broken files, symlinks/traversal, stable
  ordering, rename/delete/replacement refresh, persistence, missing directory and
  Agent precedence with isolated files and native captures where available.

## Verification receipts

- Final acceptance: run 41053972438 at source 693559e1 passed both macOS/Windows
  default/platform checks, focused regressions and native captures, plus both
  protocol jobs. Each native archive contains 1008 valid PNGs; the picker and
  selected-icon frames exist in all three locales. Representative macOS/Windows
  icon/menu frames passed visual review. TOML save/reopen, independent pane
  selection/reset/stale targets, transfer, SQLite and local asset checks passed.

- Pane ownership run 41041510659: both protocol jobs passed; both desktop checks
  failed because the moved icon module declaration and imports were missing.
  Fix plan: wire the existing module into pane/mod.rs, restore PaneIcon imports
  in app state, actions, tab menus and vertical rendering, and verify the module
  is reachable before rerunning both default/platform builds, regressions and
  native captures. Keep publication paused. Do not use partial formatting that
  can delete reordered imports without retaining their new location.

- Run 41046053799: macOS default/platform checks and both protocol jobs passed.
  Test compilation found one unqualified PaneTemplate initializer inside the
  existing leaf_pane! macro; its seven expansions omitted icon. Add icon: None
  there and audit qualified, unqualified, Self and macro constructors before
  rerunning application tests and native captures on both platforms.

- Run 41048421325: the test binary compiled; 12 focused regressions passed,
  including saved TOML, SQLite and local icons. The combined pane-isolation
  fixture called the normal drag-out API with one tab, which correctly returned
  None. Use the existing single-tab attach API for its transfer assertion;
  preserve the production last-tab guard and all remaining reopen/reset checks.

- Run 41000830906 (a5e72de8): macOS and Windows default/platform builds passed;
  both protocol jobs passed; local SVG/PNG validation/refresh and external SVG
  image isolation tests passed on both platforms. The workspace precedence test
  failed before its assertions because the shared workspace test initializer did
  not register LanguageSettings. Its eagerly-created settings pages also require
  UpdateSettings, ReleaseUpdates, AgentUsage and AgentCommunication; the workspace
  subscribes to KeepAwake. Register test defaults/mocks for these dependencies so
  this regression fixture matches the production initialization contract without
  loading credentials or starting external services. Re-run required after fix.

- Run 41006688910 (ba0aca90): the missing test-model setup is fixed; both
  desktop build/test phases passed. macOS produced the vertical-icon and picker
  screenshots, then the legacy capture's `seed unsent draft` step failed after
  the menu focus sequence. Restore active-pane focus when selecting/resetting an
  icon, assert this in the precedence regression, and explicitly target terminal
  input before typing the capture draft. Screenshot QA also found the ninth
  picker icon extending beyond the right inset; split the picker into five/four
  30px cells within the native 186px menu. Re-run native capture after both fixes.
