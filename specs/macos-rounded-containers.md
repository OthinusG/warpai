# Rounded application containers

## Product and visual contract

Preserve the existing native desktop interface as the visual source of truth.
Change only application border/background curvature. Use the current fonts,
assets, representative UI states and semantic colors; do not redesign layouts.
The terminal remains Warp-theme driven. No API or persistence changes.

Shared pixel radii in `warp_core::ui::corner_radii`:

- Control: 6 (buttons, search and text inputs, dropdown triggers).
- Panel: 8 (existing inset functional containers and review file sections).
- Surface: 10 (standalone surfaces, dialogs, menus and popovers).

Retain circular indicators, small keycaps, checkboxes and single-edge separators.
Continuous workspace sidebars and file/editor areas keep their flat shared edges;
do not wrap them in additional cards. Window chrome retains platform radii.

## Implementation tasks and scope

1. Add color-independent radii in the existing shared UI layer.
2. Apply them to both button implementations, UiBuilder controls, shared input
   wrappers, dropdowns, dialogs, menus, settings controls, collaboration usage,
   Agent management containers, Review sections and editor overlays.
3. Match exposed top/bottom corners on adjoining menu/modal/review backgrounds.
   Reuse Container's antialiased paint path. It does not clip children; preserve
   existing padding and scrolling rather than introducing content masks.
4. Audit all modified callers and existing overrides. Leave terminal/pane
   rendering, splitters, tab layout, hit testing and theme definitions unchanged.

## Acceptance and verification

- Diff contains only radii, imports and documentation; dimensions, spacing,
  backgrounds, border colors, opacity, actions and input handling are unchanged.
- No renderer changes, new dependencies, masks, allocations or layers.
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
- Passed: source invariant check across all 35 modified existing Rust files.
  After normalizing radius arguments/constants and the required imports/export,
  every file is identical to `main` at `e3fb784`. This verifies unchanged colors,
  opacity, spacing, sizes, labels, actions and event handling.
- Passed: protected-source audit. Terminal, pane/split/tab layout, theme
  definitions, both renderers, generic Container and dependency files unchanged.
- Passed: rustfmt parser validation for 36 Rust files, using package editions
  (2021 for app/warp_core, 2024 for ui_components), without formatting unrelated
  existing code. `git diff --check` passed.
- Blocked: `cargo check -p warpai --bin warpai --locked` and the same command
  with `--features warp_platform`. Both stop in the existing `warpui` build script:
  `xcrun: error: unable to find utility "metal", not a developer tool or in PATH`.
  Application type checking/linking and native tests have not completed.
- Runtime acceptance pending: modified-binary screenshots, light/dark/custom
  theme appearance, terminal resize/drag/scroll/TUI, Explorer/preview, Agent and
  Review interactions. The installed app is an older binary and cannot validate
  this patch. No claim of visual or runtime theme verification is made.
- Cloud validation: use `validate-agent-communication.yml` on this feature
  branch with `check_app=true` and `capture_ui=true`. This includes macOS and
  Windows default/platform builds, application regressions and isolated native
  screenshot artifacts. Local Metal availability is not an acceptance gate;
  this maintainer workspace uses GitHub for Rust builds, as documented in
  `docs/DEVELOPMENT.md`.

### Static fidelity ledger

| Aspect | Source evidence | Result / rendered evidence |
| --- | --- | --- |
| Geometry | Only radius arguments changed; Container layout untouched | Dimensions and spacing unchanged; native render blocked |
| Palette | No changes to color expressions or theme definitions | Source compatibility preserved; theme screenshots pending |
| Typography and assets | Source invariant comparison | Fonts, text and assets unchanged |
| Surface seams | Matching menu top/footer and reference-card top radii; Review keeps top/bottom 8 | No new card wrappers or content masks; pixels pending |
| Interaction | Event handling, splitters, hit testing and scroll code unchanged | Static audit passed; native interaction acceptance pending |

### Source inventory

Paths below are relative to the repository root. All changes are radius values,
token imports or module exports; existing 8px values retained via the panel token
are included to avoid future divergence.

- `crates/warp_core/src/ui/`: new `corner_radii.rs`, `mod.rs`, `builder.rs`.
- `crates/ui_components/src/`: `button.rs`, `dialog.rs`, `tooltip.rs`.
- `app/src/workspace/`: `mod.rs`, `view.rs` (standalone panel shell and adjoining
  worktree-menu chrome; continuous sidebars remain flat).
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
