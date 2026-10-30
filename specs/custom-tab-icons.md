# Custom tab icons

## Scope and acceptance

- Add an icon picker to the existing tab context menu, using bundled SVG icons.
- Support Folder, Code, Globe, Cloud, Laptop, Tool, Lightbulb and Notebook, plus
  Default. Selecting Default removes the override.
- Apply the selection to ordinary terminal icons in horizontal and vertical tabs,
  including compact, expanded, summary and grouped views. Preserve CLI Agent
  branding/status and non-terminal pane icons.
- Store the selection per tab in the existing workspace snapshot/database. Restore
  it after restart; preserve it when moving the whole tab between windows.
- Existing databases migrate with no override. Unknown persisted icon names fall
  back to Default. No arbitrary paths, image decoding, new dependencies, directory
  rules, Agent detection changes, or herdr/tmux integration.

## Technical contract and tasks

1. Define a typed, serialized `TabIcon` with stable snake-case names and bundled
   icon/label mappings. Add `Option<TabIcon>` to tab state and snapshots.
2. Add a nullable `tabs.icon` text column; serialize known values and safely ignore
   unknown values. Extend constructors/fixtures and snapshot/restore paths.
3. Dispatch `SetTabIcon { tab_index, icon }` from the existing menu's icon row;
   bounds-check the target and notify the workspace. Reuse native hover labels,
   accent colors, and interface translations.
4. Thread the icon into vertical pane/summary rendering and the horizontal
   ordinary-terminal indicator, leaving Agent/status branches unchanged.
5. Verify enum/menu behavior, set/reset isolation, persistence and legacy/unknown
   values. Run locale coverage and focused formatting locally. Use existing
   GitHub build infrastructure for Rust checks/tests and native UI where feasible,
   following MEMORY.md (no local builds or personal app/config replacement).

## Risks and verification

The schema/model column order must agree for Diesel. Missing snapshot fields in
fixtures must be updated. Summary/group collage paths must not hide the override.
Use an isolated database for migration/round-trip tests; do not access personal
terminal history. UI acceptance checks picker fit, hover labels, both tab layouts,
restore/reset, and Agent badge precedence. This change does not publish a release.
