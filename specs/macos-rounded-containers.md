# Rounded application containers

## Product and visual contract

Preserve the existing native desktop interface as the visual source of truth.
Change only application border/background curvature. Use the current fonts,
assets, representative UI states and semantic colors; do not redesign layouts.
The terminal remains Warp-theme driven. No API or persistence changes.

Shared pixel radii in `warp_core::ui::corner_radii`:

- Control: 8 (buttons, search and text inputs, dropdown triggers).
- Panel: 12 (existing inset functional containers and review file sections).
- Surface: 20 (standalone surfaces, dialogs, menus and popovers).

Owner final direction: use larger Apple-style corners and merge directly into
`main`. This final code adjustment changes only the three shared pixel values
from 6/8/10 to 8/12/20. All existing layout, paint, clipping and interaction paths
remain as validated for `d256086`; the earlier screenshots show the smaller radii.
The final values pass formatting and focused source-diff checks. The existing
main-push workflow validates the integrated source after merge; the earlier
cloud receipts below do not claim to test these final numeric values.

Retain circular indicators, small keycaps, checkboxes and single-edge separators.
Configurable application panels must have visibly rounded outer contours in
the main window, including vertical tabs, collaboration/Explorer and Review.
Retain their exact bounds and keep terminal panes free of decorative masks.
Window chrome retains platform radii.

Owner correction: the first main-window screenshot still showed straight panel
boundaries. Shared control radii alone do not meet acceptance. Configurable
panels bypassed the legacy rounded surface wrapper. Add a shared outer outline
at the actual configurable-panel entry point, including maximized Review; round
the existing vertical-tab background. Replace separator paint with the same
one-pixel spacer so straight lines do not visually reconnect rounded contours.
Use foreground borders with existing outline colors: no inset, no padding or
margin changes, no content clipping, and no changes to splitter event handling.
Re-run cloud verification and inspect the replacement main-window screenshot.

## Implementation tasks and scope

1. Add color-independent radii in the existing shared UI layer.
2. Apply them to both button implementations, UiBuilder controls, shared input
   wrappers, dropdowns, dialogs, menus, settings controls, collaboration usage,
   Agent management containers, Review sections and editor overlays.
3. Match exposed top/bottom corners on adjoining menu/modal/review backgrounds.
   Reuse Container's antialiased paint path. It does not clip children; preserve
   existing padding and scrolling rather than introducing content masks.
4. Outline configurable main-window panels in their existing bounds, retaining
   separator widths and resize handles. Do not decorate terminal panes.
5. Audit all modified callers and existing overrides. Leave terminal/pane
   rendering, splitters, tab layout, hit testing and theme definitions unchanged.

## Acceptance and verification

- Diff contains radii, fixed-bounds panel outlines, imports and documentation; dimensions, spacing,
  backgrounds, border colors, opacity, actions and input handling are unchanged.
- No renderer changes, new dependencies, content masks or offscreen passes.
  Reuse the existing foreground-border paint layer on each outer app panel.
- Match expanded/collapsed Review headers and bottom content corners.
- Check formatting and application compilation (default and warp_platform).
- Render the modified binary if the local native build environment permits it;
  inspect light, dark and third-party themes with menus, settings, collaboration,
  Explorer, preview and Review. Exercise split resize/drag, scrolling, selection,
  and CLI TUI. If native tools block rendering, record the blocker and source
  checks explicitly; installed older app screenshots are not change validation.

## Risks

Large radii can expose square child backgrounds: check adjoining backgrounds
and retain smaller radii for tight controls. Never change generic Container
defaults or introduce global clipping. Shared control changes affect callers
throughout the app; their colors and dimensions must stay identical.

## Verification results

- Completed: shared tokens and consumers; no new framework or dependency.
- Passed for the first revision: source invariant check across all 35 modified existing Rust files.
  After normalizing radius arguments/constants and the required imports/export,
  every file is identical to `main` at `e3fb784`. This verifies unchanged colors,
  opacity, spacing, sizes, labels, actions and event handling.
- Passed: protected-source audit. Terminal, pane/split/tab layout, theme
  definitions, both renderers, generic Container and dependency files unchanged.
- Passed: rustfmt parser validation for 36 Rust files, using package editions
  (2021 for app/warp_core, 2024 for ui_components), without formatting unrelated
  existing code. `git diff --check` passed.
- Local environment blocker: `cargo check -p warpai --bin warpai --locked` and the same command
  with `--features warp_platform`. Both stop in the existing `warpui` build script:
  `xcrun: error: unable to find utility "metal", not a developer tool or in PATH`.
  Cloud checks supersede this local build limitation; see receipts below.
- Cloud validation: use `validate-agent-communication.yml` on this feature
  branch with `check_app=true` and `capture_ui=true`. This includes macOS and
  Windows default/platform builds, application regressions and isolated native
  screenshot artifacts. Local Metal availability is not an acceptance gate;
  this maintainer workspace uses GitHub for Rust builds, as documented in
  `docs/DEVELOPMENT.md`.

### Cloud receipts

#### Accepted implementation follow-up

- Tested source: `d25608650116ff5949418b93169020ed6b8bb301`.
- Run: https://github.com/OthinusG/warpai/actions/runs/41588569049 — success.
- Both protocol jobs and macOS/Windows desktop jobs passed, including default
  and platform checks, the Windows release-profile check, application regression
  suites, native review builds and native capture assertions.
- Each platform produced 1,008 native PNGs across the three interface languages.
  Artifacts: macOS `13210813102`, Windows `13212043827`. Downloaded to
  `/tmp/warpai-rounded-d256086/{macos,windows}`. Both diagnostics match the exact
  tested source, exit 0 and contain no failed steps or assertions.
- Inspected macOS and Windows main Explorer/terminal captures and macOS dark
  collaboration and light/dark language menus. The main-window tab sidebar and
  Explorer/collaboration outer outlines now visibly round their top and bottom
  corners at 10px. Terminal contents retain rectangular, unmasked bounds.
- User preview: unchanged native Simplified Chinese macOS
  `live-ssh-file-explorer.png`, copied to
  `/Users/wqin/Downloads/warpai-rounded-ui-d256086/warpai-main-macos-zh-Hans.png`
  with a source/artifact provenance receipt. The earlier `736f918` preview is
  retained separately for comparison and is superseded.
- Passed focused follow-up source checks: foreground-only panel outlines,
  unchanged 1px separator width, no added content insets/masks or resize logic,
  Rust syntax and whitespace checks. Terminal, pane-group, theme and renderer
  source remain untouched.
- Arbitrary third-party themes and exhaustive terminal/TUI/graphics and splitter
  gesture combinations remain outside these finite capture suites. The narrow
  multi-pane Review fixture retains its original crowded layout.

#### First revision (superseded visual result)

The owner rejected this revision's straight main-panel boundaries. These
receipts are retained as history; the follow-up above is the current result.

- Tested source: `736f918325768198b8d5fd56c9a4a84a66e22153`.
- Run: https://github.com/OthinusG/warpai/actions/runs/41580187623.
- Passed macOS and Windows protocol suites, macOS default/platform checks and
  application regression suite. Windows default/platform/release-profile checks
  and application regression suite also passed; native Windows capture pending.
- Passed macOS native application build and UI captures: 1,008 PNGs across
  English, Simplified Chinese and Traditional Chinese. Artifact:
  `Warpai-collaboration-native-captures-macOS` (ID `13207639793`). Its diagnostic
  matches the tested source, exits 0, and has no failed steps/assertions/panics.
- Downloaded screenshots to `/tmp/warpai-rounded-736f918/macos`. Selected the
  Simplified Chinese `live-ssh-file-explorer.png` for the user; copied unchanged
  to `/Users/wqin/Downloads/warpai-rounded-ui-736f918/warpai-main-macos-zh-Hans.png`
  with `provenance.json`. No screenshot assets added to repository source.
- Visually inspected the main Explorer/terminal layout, Agent/evidence layout,
  dark collaboration, light/dark language menus and the warm Review fixture.
  Controls and menu contours are rounded, flat workspace seams remain intact,
  and light/dark menus retain contrast. The 1440px multi-pane Review fixture
  squeezes terminal/preview columns and wraps text; geometry is unchanged by
  this radius-only patch. Do not use that fixture as a polished main preview.
- Not established by these captures: arbitrary third-party custom themes,
  exhaustive terminal/TUI/graphics-protocol behavior, every resize/splitter
  gesture, or a complete before/after pixel comparison.

### Static fidelity ledger

| Aspect | Source evidence | Result / rendered evidence |
| --- | --- | --- |
| Geometry | Only radius arguments changed; Container layout untouched | Main Explorer and Agent captures inspected; crowded multi-pane fixture recorded above |
| Palette | No changes to color expressions or theme definitions | Native light/dark menus and warm main view inspected; arbitrary custom themes untested |
| Typography and assets | Source invariant comparison | Fonts, text and assets unchanged; native Chinese glyphs present |
| Surface seams | Matching menu top/footer and reference-card top radii; Review keeps top/bottom 8 | Rounded menu/control contours inspected; no new wrappers or masks |
| Interaction | Event handling, splitters, hit testing and scroll code unchanged | macOS native assertions passed; exhaustive gesture/TUI acceptance not claimed |

### Source inventory

Paths below are relative to the repository root. The first revision changed
radius values, token imports and exports. The follow-up adds foreground outlines
to actual configurable-panel shells and rounds the existing tab-sidebar fill.

- `crates/warp_core/src/ui/`: new `corner_radii.rs`, `mod.rs`, `builder.rs`.
- `crates/ui_components/src/`: `button.rs`, `dialog.rs`, `tooltip.rs`.
- `app/src/workspace/`: `mod.rs`, `view.rs` (panel shells and adjoining
  worktree-menu chrome), `view/vertical_tabs.rs` (existing sidebar background).
- `app/src/`: `menu.rs`, `modal.rs`.
- `app/src/ui_components/`: `dialog.rs`, `item_highlight.rs` (also used by
  Explorer rows).
- `app/src/view_components/`: `action_button.rs`, `agent_toast.rs`, `alert.rs`,
  `clickable_text_input.rs`, `compact_dropdown.rs`, `dismissible_toast.rs`,
  `filterable_dropdown.rs`, `find.rs`, `markdown_toggle_view.rs`,
  `submittable_text_input.rs`.
- `app/src/agent_communication/`: `panel_usage.rs`.
- `app/src/ai/agent_management/`: `view.rs`, `agent_type_selector.rs` (existing
  feature gates unchanged).
- `app/src/code/`: `find_references_view.rs`, `local_code_editor.rs` (hover
  overlay only); `editor/comment_editor.rs`, `editor/find/view.rs`.
- `app/src/code_review/`: `code_review_view.rs`, `diff_menu.rs`.
- `app/src/settings_view/`: `appearance_page.rs`, `keybindings.rs`, `mod.rs`,
  `settings_page.rs`, `update_environment_form.rs`.
- Documentation: this contract and the durable token rule in `MEMORY.md`.
