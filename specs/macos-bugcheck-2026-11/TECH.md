# macOS field corrections

## November 4 hardware follow-up

Visual authority: owner-marked `Desktop/bugcheck/{1,2,3,4,5,6}` screenshots
and the November 4 request. Work directly on the current branch.

- Vertical only: remove the icon sidecar corner artifact; round only the outer
  history group corners whose actual boundaries are visible; move window controls
  and both toolbar groups upward with horizontal clearance for the window arc;
  align the terminal top with the inset side columns. Preserve horizontal chrome.
- Both layouts: use Secondary Back everywhere, visibly distinguish archived tasks
  and offer Delete with the same native controls. Archive retains history and is
  idempotent. Delete targets one versioned terminal task, reuses archive/certainty
  and purge reference guards, and preserves audit events and request receipts.
- Detect every registered agent executable without ASCII case sensitivity,
  preserving arguments, paths, custom aliases, print/protocol exclusions and
  launch authorization. Exercise uppercase and mixed case aliases.

Validation: focused protocol/application regressions and exact-source macOS and
Windows native captures on GitHub; inspect the resulting screenshots. Poll CI
every 15 minutes, repair failures and repeat. No release or personal config edits.

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

## Test installer authorization (2026-11-04)

The owner requested 15-minute validation probes and a macOS test installer after
acceptance. Reuse `release-macos.yml` on this branch with `artifact_only=true`,
`auto_publish=false`, and test package version `v1.8.1.1`. This version labels the
artifact only; it creates no tag or release. The workflow's release-target marker
makes the downstream Windows workflow skip packaging; Linux consequently skips.
Required Companion jobs are part of the existing macOS packaging dependency.
Verify exact source, DMG checksum/integrity, bundle version/identity/signature,
canonical artwork and bundled MCP bridge before delivery. Do not install it into
the owner's application/configuration directories.

Current validation: https://github.com/OthinusG/warpai/actions/runs/41668572216
at runtime source `816466321c97f981c63fad3424856cccd81961ca`.
Both protocol suites and both desktop default/platform checks passed. Application
regressions and source-matched native captures are still running; packaging has
not been started.

## CI correction plan: removed setting default

macOS run 41668572216 failed the existing
`owner_defaults_match_registered_typed_file_defaults` regression: the removed
`appearance.vertical_tabs.hide_title_bar_search_bar` registration still had an
entry in `crates/settings/src/owner-defaults.json`. Default/platform compilation
and both protocol suites had passed. Remove only the obsolete bundled default;
retain the schema consistency test and rerun the same cloud validation/captures.
The test installer remains gated on acceptance.

## CI correction plan: finite column toolbar layout

Run 41674696749 passed the default consistency gate, then the retained vertical
column regression caught an infinite horizontal constraint in the toolbar's
max-size Flex. The outer workspace lays out side columns without a fixed width;
that width belongs to the existing resizable panel bodies. Measure those bodies
first using the existing Stack, reserve the toolbar height in their container,
and position the toolbar with ParentBySize bounds. This keeps live resize state
in its original owner, avoids cached/manual width synchronization, and bounds
the toolbar before its max-size Flex is laid out. Preserve both regression and
native screenshot assertions; rerun exact-source cloud acceptance.

## CI correction plan: Windows native C++ runtime

Run 41680297211 passed every macOS check, application regression and native
capture assertion, plus both protocol suites. Windows default/platform checks,
release-profile checks and application regressions passed; the native review
executable failed to link with LNK2038/LNK2005/LNK1169. The new system_http C++
bridge uses MD while the locked esaxx-rs 0.1.10 library forces MT. Its downloaded
crate checksum matches Cargo.lock, and its build.rs calls static_crt(true).
Match that existing Windows C++ runtime in system_http's Windows branch only;
leave the Rust target flags, allocator boundaries and macOS/Linux branches
unchanged. Rerun Windows desktop/protocol/native acceptance. macOS UI sources
remain identical to the fully captured 5edbad2f revision; packaging repeats the
macOS compile/protocol gates on the final source.

## Visual correction: drawable shadow padding

The 5edbad2f archive contains 1,110 valid native PNGs (370 per language), with
exit zero and no failed steps/assertions. Inspected the Chinese three-column
Explorer/history, input/chips, tab rows, icon menu, light/dark language menus and
Reconnect views. Native test windows deliberately hide macOS traffic lights;
real hardware window controls remain part of owner test-package acceptance.

Pixel checks exposed no shadow outside the column: spread_radius=0 gives the
renderer no drawable area outside the surface. Existing rect.rs uses spread as
quad padding, and rect_shader.wgsl integrates the blur over three sigma. Retain
blur=12, alpha=28 and offset y=2; set drawable spread=36. Re-capture and verify a
visible gradient in the margin against the same-frame flat background. No
renderer or clipping changes are needed. Other inspected geometry, palette,
typography, assets and menu highlights match the scoped visual contract.


### Follow-up validation receipt

- First candidate: `f105e957`, run
  https://github.com/OthinusG/warpai/actions/runs/41759499952.
- Existing OAuth permissions reject workflow-file changes. Reused the unmodified
  workflow: new application checks use its `collaboration_` group and the native
  submenu replaces the existing icon-picker checkpoint, preserving image counts.
- The vertical icon checkpoints exercise one failed history block and multiple
  failed blocks through native PTY commands that read fixed, absent keys from the
  owned fixture repository's local Git config. They write no project/config data;
  the existing ClearBuffer action removes their visible history afterward.
- Local Rust parsing, diff checks and strict translation coverage passed.
  Compilation, native rendering and functional acceptance remain pending CI.


### Icon corner root cause

`TabData::icon_option_menu_items` put a Right path tooltip on the Local icons
submenu parent. Hover rendered that tooltip at the same anchor as the sidecar;
the tooltip remained behind its larger rounded corner. Omit that redundant hint
only for vertical tabs. Retain horizontal tooltip behavior, sidecar geometry and
shadows. The native icon checkpoint now sends a real hover to the saved parent
row, exercising both the tooltip and sidecar trigger.


CLI launch guards share the session's alias/abbreviation/function override
check. PowerShell lookup ignores ASCII case; POSIX lookup remains exact. Agent
alias resolution follows the same shell rule. Regressions cover all three
namespaces and preserve case-sensitive arguments and model names.

### CI repair plan: capture 41761210562

Both protocol suites and macOS default/platform/application regressions passed.
Native captures completed English and Simplified Chinese (370 PNGs each), then
Traditional Chinese stopped at `owned remote tree populated` (213 PNGs). The
redacted diagnostic includes `SshAuthenticationUnavailable`; source changes do
not modify SSH transport. Do not weaken authentication, reconnect policy or the
Explorer assertion. Establish fixture multiplex readiness with OpenSSH's own
check command, retain bounded status-only diagnostics, and rerun on final source.

Visual inspection independently found that the multi-block outer corners were
not painted although the single-block corners were. Strict float comparisons
can classify an accumulated subpixel boundary as offscreen. Add regression
coverage for near-equal bounds, tolerate only layout rounding and clamp the mask
origin to its viewport. Real clipped boundaries must remain square. Re-run native
captures and inspect pixels before accepting this diagnosis.

Windows native compilation and all application regressions also passed, but its
capture stopped at the Reset Grid assertion in `grid/ansi_handler.rs`. Directly
inserting restored history into the live fixture PTY creates a new active grid
without its shell's reset handshake. Keep that production assertion. Replace
only the synthetic insertion with three harmless native Git read commands,
submitted separately through the existing pending-command API and awaited via
their completed failure blocks. Preserve capture names/counts and the live shell
for the later SSH scenarios; rerun both native capture jobs on corrected source.


History edges use the existing non-hidden block queries. Input pinned to the top
reverses physical block order, so swap the logical endpoints before computing
outer corners. Preserve the existing input-placement setting and scrolling.

## November 4 final verification

Runtime source `3b97abd7d939c2e0c27cc1b14da2c1e63b0aa648` passed
https://github.com/OthinusG/warpai/actions/runs/41800961052.
Both protocol suites, default/platform application checks, application regressions
and native capture jobs succeeded on macOS and Windows. Probes ran at 15-minute
intervals; the final run resolved the prior Windows Reset Grid fixture panic
without changing its production invariant.

Downloaded and verified 1,110 PNGs per platform (370 per language). Both diagnostics
report exit zero, no failed assertions and no panic; all three macOS SSH masters
remained alive. Reviewed final single/multiple history, icon sidecar and navigation
captures: exterior corners are rounded, internal joins remain square, the redundant
tooltip artifact is absent, and Back uses the Secondary button. Numeric geometry
confirms a genuinely clipped top edge is not rounded, while subpixel aligned
boundaries are rounded. No personal configuration or release was changed.

Native macOS traffic lights are excluded from GPU capture. Their titlebar geometry
was reviewed in source; physical traffic-light placement still needs hardware
confirmation. Older remote Companions need the additive TaskDelete operation to
support targeted deletion; no broad-purge fallback is used.
