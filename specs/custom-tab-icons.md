# Custom tab icons

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
