# Configuration migration and interface localization

## Existing implementation

- warp_core::paths owns the desktop managed root, profiles and channels.
- app::user_data_migration imports missing legacy files without overwriting either
  destination data or recovery sources, rejects destination symlinks and marks
  completion after successful writes. Extend this path; do not create a manager.
- Public settings use settings.toml; legacy JSON and private preferences remain
  separate. define_settings_group!/Setting supplies typed defaults and Reset.
- Native UI strings are currently embedded in Rust. No existing locale catalog
  was found. Native Text/widgets must continue rendering user content verbatim.
- Companion/WSL installers and application discovery hardcode their managed paths.
  Inventory these references before changing the desktop root; preserve support
  for installed 4.0.0 companions and avoid silently breaking remote discovery.

## Tasks and verification

1. Inventory managed-root callers, installers, scripts, fixtures and persisted
   executable paths. Add a separate root-migration completion boundary before
   existing legacy import so an imported old marker cannot suppress migration.
   Preserve channels/profiles, permissions, conflict precedence and recovery data.
   Never execute this migration against the owner's files during development.
2. Read only confirmed nonsecret local UI/terminal preferences. Build a reviewed
   public preference baseline using existing SettingsValue conversion. Exclude
   private/account/path/server/history data, reject invalid typed values, and make
   normal Reset and fresh installs agree. Do not overwrite existing preferences.
3. Add a persisted typed language setting through existing settings registration
   and the native dropdown. Apply it at startup, requiring restart so cached menus
   and active drafts stay consistent. Keep locale identity independent of labels.
4. Establish explicit UI text catalogs for en, zh-Hans and zh-Hant. Mark display
   boundaries rather than translating arbitrary Text/terminal/editor content.
   Translate dynamic templates before interpolation; verify identical arguments.
   Inventory reachable screens and shared controls, including menus, errors,
   tooltips, notifications, accessibility and native platform menus.
5. Add source/catalog coverage checks and focused migration/default/template tests.
   Local verification is syntax/script/catalog only; Rust and runtime checks run
   on GitHub concurrently with 1.6.0, with all three locales and representative
   narrow/high-zoom/light/dark screenshots. Preserve existing SSH/WSL assertions.
6. Incorporate later 1.6.0 fixes by their existing commits, then complete full
   source-matched acceptance and existing immutable-tag release gates for 1.6.2.

## Risks

Directory migration must not overwrite databases, partial writes or explicit new
preferences. A simultaneous older app must not create divergent active stores.
Symlinks and stale absolute MCP/Companion paths require explicit handling.
Local preferences may include platform-only fonts/options; retain existing safe
fallbacks. Chinese text can widen rows or change keyboard/search behavior. English
runtime fallback is not evidence that Chinese interface coverage is complete.
No new cloud translation runtime, telemetry, billing or bundled Agent capability.

## Implementation progress

The owner baseline contains 51 registered public preferences. Theme defaults use
built-in Claude Warm Light, Catppuccin Latte and Dark; the owner explicitly chose
Dark because the former LLM Dark file is unavailable. Saved explicit themes remain
unchanged. No machine-specific theme paths are bundled.

The explicit interface catalog currently covers marked static labels and selected
named templates. script/check-interface-locales.py validates catalog arguments and
common direct display boundaries, excluding reviewed brand/technical names and
terminal preview content. Passing this checker does not establish complete UI
coverage: indirect constants, optional help text, dynamic actions/statuses, search
terms and accessibility labels still require review and native locale walkthroughs.
The 1.6.2 workflow runs the checker and sets desktop version metadata to 1.6.2;
the owner now authorizes concurrent 1.6.0/1.6.2 cloud acceptance, checked every
30 minutes. Each version retains its own complete release gates.

Native captures now accept explicit language codes through the existing SSH/file
acceptance driver. Each native process loads its language from a newly created
public TOML file in its own protected capture profile before interface creation;
a startup assertion verifies the stored and active language agree. The workflow
runs English, Simplified Chinese and Traditional Chinese sequentially against the
same owned SSH fixture, retaining separate process/profile artifacts and the
existing WSL cases. No production language override or personal installation is
introduced. Cloud runtime results remain pending.

Focused cloud checks cover reordered confirmation arguments, preserved literal
user braces, typed public owner defaults and bilingual settings search. Native
capture setup persists each locale before application initialization and asserts
that both the typed setting and active catalog match. Dropdown labels used for
selection share the same translation path. Internal collaboration section IDs
remain English; translation is applied only to the rendered headings.

Additional literal action descriptions are checked at BindingDescription and
EditableBinding construction sites. The catalog includes terminal/editor actions
and compiled debug labels without changing action IDs or key sequences. Language
selector updates subscribe to the settings model so Reset and external preference
reloads reflect the persisted language; restart still applies the interface locale.

All nonempty inventoried settings keyword groups have simplified/traditional
search aliases. Communication forms translate their labels and validation errors
at display boundaries, retaining original enum identities and literal confirmation
tokens (ALLOW OVERLAP, RELEASE RESERVATION, DELETE HISTORY). Companion response
status text is translated on the host without changing Companion 4.0.0 payloads.
Heading icons recognize their translated titles and have a focused three-locale
regression check. Font weight dropdown creation and selection use identical labels.

Task state labels are translated at the task detail, task list, worktree expansion
and task selector boundaries. Query state values and comparisons retain their
protocol identity; user descriptions and task IDs remain unchanged. Catalog tests
cover every current task lifecycle state in all three interface languages.

Constant-based find tooltips/placeholders, secret-redaction descriptions and MCP
empty states are localized at their callers. The catalog checker resolves local
constant and LazyLock references used by explicit localization calls, with
self-checks for both declaration forms; it does not translate arbitrary content.

The accessibility audit also covers the help argument of announcements and views,
including native menus, workflow selection, find/replace, command input, update
and notification guidance. Command/output content stays literal. Shared command
input labels are localized at consumers, retaining the original constant values.

Update availability and toast/link labels are localized. Static update status is
translated when rendered so the existing failure-prefix check that exposes the
manual release link retains its behavior. Privacy page titles use the selected
locale without changing SettingsSection identity.

Literal MenuItemFields labels are localized at construction sites, and that
constructor family is included in the strict display inventory. Platform reveal
actions and notebook block/heading labels share translated display values; file
paths, shell names, branch names and shortcut labels remain literal.

Tooltip, placeholder, toast link and formatted text fragment constructors are
included in the display inventory. Commands, regular expressions, config file
names and branch-name examples stay verbatim. Linux desktop updater UI is
explicitly excluded; its implementation is outside this release's desktop scope.
The macOS backend initializes application settings before building its menu bar,
so the selected locale is available when native menu labels are constructed.

Toast notifications and constant-based labels, including commit/find/keybinding
placeholders, theme dialogs, prompt setup and settings import, are localized only
at display consumers. Constant declarations retain their original values. Pane
kind badges and fallback titles translate fixed kinds; user titles stay literal.

Local preflight at b441332d: all 329 changed Rust files parse under their crate's
declared 2021/2024 edition. Strict catalog checks cover 2,573 translation pairs
and 2,675 marked display sites; changed Python, YAML and JSON syntax checks
pass. Companion sources and Linux desktop updater sources match the 1.6.0
acceptance baseline 8a366b1d. Cloud compilation, tests and three-locale native
visual acceptance remain pending; parallel cloud acceptance is now authorized.

Expand native acceptance to all seven additional visible settings sections:
Appearance, Features, both Code subpages, Keyboard shortcuts, Shared blocks and
Privacy. Capture each in light/dark at 800/1200 width and 125% zoom; assert the
actual selected section before capturing. Also capture the open Language menu
in each Appearance variant. Existing About state captures remain in place.
This adds 32 images per locale (macOS 918 total; Windows with WSL 936 total).
Use isolated profiles and the bounded 420-second watchdog; runtime and visual
results must pass the cloud gate before release.

The outer native capture CI step allows 30 minutes for three sequential locale
launches (450-second subprocess limit each), three existing bounded SSH/SFTP
checks (120 seconds each), and fixture setup/cleanup. Per-process 420-second
watchdogs and zero retries are unchanged; one stalled locale still fails early.

Git push/publish/PR confirmation and progress labels, secret-display dropdown
labels and terminal-kind badges translate their fixed display text. Typed actions,
redaction modes, Git commands and CLI agent program names are unchanged.

Accessibility announcements without a separate help argument are included in
the display inventory and localized, including selection, clipboard, find/replace,
scrolling and menu actions. Shortcut names keep their key identities.

Indirect display fallbacks (untitled files, unnamed messages, workspaces and
servers), detached-branch labels and reservation/attempt summaries translate at
their display producers. Real names, branch names, message bodies and protocol
values remain unchanged; history task-state labels use the existing catalog.

Review discard confirmation titles/descriptions, rule-initialization captions,
unsaved-file shortcut tooltips and MCP logout/deeplink results are explicit
localized display producers; discard scope and server installation behavior
are unchanged. Indirect producers remain part of manual coverage review.

Command-palette filter names, new-session descriptions and split directions,
conversation section titles and loading announcements translate explicitly.
Tab-config removal/parameter prompts and diagnostic severity prefixes are
localized; configuration identifiers, shell names and LSP messages stay raw.

Global-search counts retain the existing singular/plural branch and translate
its display noun. Markdown editor accessibility announces links, paste, inserted
blocks, conversions and language changes through catalog templates; file-open
banners and notebook movement labels preserve their user-supplied arguments.

Display inventory includes Dialog titles, Span labels and bold/italic text.
Native confirmation producers localize shared-session close, rewind, MCP removal
variants and unsaved-change dialogs while preserving each operation's scope
and warning text. Diagnostic logs and generic user-provided dialogs stay raw.

Default-session and SSH-install dropdowns translate enum labels only at display
consumers; selection still uses typed actions or matching translated labels.
New-session search ranking bases use the current interface language, matching
its localized result descriptions and accessibility launch announcements.

Native modal buttons and success toasts are part of the display inventory.
Quit warnings preserve their existing scope/count logic and plural branches
while localizing process/session nouns, unsaved-change messages and controls.
Local worktree removal keeps all checkout protections and the retained-branch
warning; the display text alone changes. Generic platform modal builders are
unchanged, preserving user-supplied titles and messages.

Command-palette accessibility includes files/directories, create-file actions,
launch configs, tabs, repositories, sections, conversation actions and session
command context. Navigation completion times and empty/running status hints use
the same explicit templates; real prompts, commands and paths stay unchanged.

Display inventory includes label/title setters, static dropdown headers and
optional tooltips. Runtime Git-action updates, comment button states, file-nav
and maximize controls translate at their producers so updates retain the
selected interface language. Generic component setters remain unchanged.

SSH/subshell Warpify footer updates and hidden environment-form submit labels
use explicit localized display values. Native file-type filters translate Image
while retaining Markdown/Yaml names and file-extension matching.

Localized concept names use consistent translations for Agent, Worktree,
Coordinator, Workspace and Spaces. Source keys, placeholder names, protocol
identifiers and user-provided values stay unchanged; Agent SDK and HTTP
User-Agent names retain their technical spelling.

Markdown inline-style accessibility translates the known Bold/Italic/Underline/
InlineCode/StrikeThrough display labels and on/off announcements. English keeps
its original debug-style wording; text formatting state and stored styles do
not change. Both editor and toolbar callers use the existing shared helper.

## Initial cloud gate correction

Run https://github.com/OthinusG/warpai/actions/runs/38106145597 fails before
native acceptance. Windows decodes the UTF-8 catalog using its default charmap;
macOS compilation reports runtime translation in four platform-specific static
Meta-key labels and a private Appearance module reference from the capture driver.
Read/write source catalogs explicitly as UTF-8 and normalize inventory paths to
POSIX spelling so Windows uses the same reviewed exclusions. Keep Meta-key
constants literal and translate their display consumers. Re-export only the
Appearance view type within the crate for the existing debug capture. Verify
encoding warnings as errors, strict coverage and Rust syntax locally, then rerun
both cloud build configurations, focused tests and three-locale native acceptance.

## Native locale review correction

The macOS portion of run 38106868836 passes its build/test/capture gates and
produces 918 valid images. Visual review finds missing catalog entries for
enum-derived navigation labels and fixture statuses, a literal Settings header/
tab title, and optional setting descriptions that bypass explicit localization.
Translate those fixed display producers and add a regression covering actual
SettingsSection labels in both Chinese locales while preserving English IDs.
Settings already uses dual-axis scrolling below its minimum page width; narrow
captures show only its left edge. Reuse native horizontal wheel events to capture
both edges at width 800 and open the Language menu after revealing its controls.
Add 14 right-edge captures per locale (macOS 960; Windows with WSL 978), retain
the existing page-selection checks and inspect both edges in cloud artifacts. Four native context-menu captures per locale
bring the final image counts to macOS 972 and Windows with WSL 990.

Windows completes the same run with 936 valid images and no failed native
assertions. Review also finds fixed labels outside the inventory's direct
literal patterns: keybinding descriptions after constant identifiers/comments,
keybindings help text, Markdown mode labels, Zoom and remote connection buttons,
and fixed vertical-tab titles. Expand the inventory for those binding arguments
and the page-local text helper; translate fixed producers without rewriting
user tab titles or commands. Review terminal, tab, file-tree, editor and
collaboration context-menu labels and include native menu captures where the
existing debug driver can exercise the menu. Verify strict UTF-8 coverage,
syntax and focused regression tests, then rerun both platforms from one commit.

Menu hover dispatch and saved-position anchors must compare the localized fixed
menu labels. Otherwise Move to group and New worktree config sidecars stop
opening in Chinese despite translated parent labels. User group names, tab
configuration names, commands, shell names and paths remain literal.

The 738cbc08 cloud run (38113575843) fails both desktop compile gates because
the native capture assertions used `MenuItem::fields`, which exists only under
`cfg(test)`. Match the existing public `MenuItem::Item` variant directly in the
four debug assertions; retain real menu labels and all screenshot assertions.
Rerun complete source-matched desktop acceptance after this driver-only fix.

Run 38115465987 passes macOS compilation, regressions and 972 source-matched
PNG captures for 6b0995e5. Visual review still finds the shared pane maximize
label, exported Save file binding description, generated slash-command prefix
and Warpify success heading in English. Translate their fixed producers and
the remote-refresh error wrapper while retaining command names and error detail.
Extend constant binding inventory and assert the maximize label in the real
code-pane menu. Recheck both desktops and native images before publication.

Run 38119305483 passes macOS compilation/tests but capture fails in the second
locale before menu assertions: owned remote tree population reports
`SshAuthenticationUnavailable`. The capture fixture reused one OpenSSH master
for three application processes, allowing previous-process channels to survive
into the next locale. Give each locale its own short private master socket and
terminate that master in the existing finally cleanup. Assert it is live before
launch; retain authentication, real file-tree assertions and process deadlines.
Rerun the complete source-matched gate; do not treat the partial 499 images as
acceptance. If failures persist, inspect redacted owned-server failure categories.

Run 38122883940 passes macOS native acceptance with 972 valid source-matched
PNGs and clean diagnostics for b0a3aaa5; all four menus and shared pane labels
are Chinese in both Chinese locales. Further settings review finds the external
editor default-option variables unlocalized and New Tab selection still matched
the English label. Localize those fixed defaults and their matching selectors,
including the existing AI-layout consumer; actions/settings values stay unchanged.
Close the debug Language dropdown after each capture so it cannot carry its
modal state into another page/width. About fixtures must use the production
available-version translation template instead of an English preformatted status.
Add the default-option producer to strict inventory; rerun both platforms and
inspect full settings/error/update states before accepting this revision.

Run 38126828908 passes both platforms for dd55235d, with 972 macOS and 990
Windows source-matched PNGs, clean diagnostics and 324/330 images per locale.
Settings, updates and four native menus pass visual review. The live WSL Review
capture exposes two remaining variable-produced English labels: the header's
no-branch fallback and the selected uncommitted diff target. Localize those fixed
display producers and the empty selector fallback; preserve actual branch names
and DiffMode values. Extend strict inventory to DiffTarget display literals and
the existing WSL authority assertion to require the localized Head target label.
The catalog now has 2677 pairs and 2780 marked display sites. Rerun the complete
both-platform/native gate and inspect Review in both Chinese locales before
tagging 1.6.2; successful dd55235d CI alone is not final visual acceptance.

Run 38132366093 exposes E0308 in the new Windows native assertion: ViewHandle
read callbacks provide AppContext, while build_diff_targets unnecessarily
requires ViewContext. The producer only reads repository/model state. Accept
AppContext in that existing method, preserving its rows and every caller; do not
add a duplicate fixture producer or loosen the assertion. Rerun both platforms.

Run 38135111782 passes macOS native acceptance for a822977c; the live SSH
Review header and Head target are localized. Screenshot review exposes the
shared successful-save toast still using an English String::from literal.
Localize that display producer and extend strict inventory to String::from
arguments at existing display boundaries. Review and localize the 39 newly
exposed tooltip, toast, menu, accessibility and button literals, retaining
action IDs, error handling and user content. Add 27 catalog pairs and a scanner
self-check; no cloud feature is enabled by translating dormant shared UI.
Rerun full both-platform native acceptance and inspect successful-save toasts
in both Chinese locales before tagging.

Run 38139339617 fails Windows before application/native acceptance: Chocolatey
reports zero installed packages with a successful exit, then remote_protocol
cannot find protoc. Run this native package step in PowerShell, check its exit
status and the compiler version, and publish its verified executable path as
PROTOC for subsequent Cargo steps. Keep the protocol and native acceptance
gates intact. Rerun both desktops for the resulting source before release.

Run 38139771778 passes both desktops for 2580c5ef (972 macOS/990 Windows
source-matched PNGs; clean diagnostics). Windows Traditional Chinese WSL
review happens to capture the loading state, exposing the variable-produced
Loading open changes header. Localize that shared placeholder producer. Extend
header_text inventory and localize its one additional fixed conversation-summary
header; retain state transitions and user content. Run full native acceptance
before replacing the unpublished release candidate. Release run 38144883701
was started early with auto_publish=false at the owner request; its tagged
source predates this final loading-state correction and must not be published.
