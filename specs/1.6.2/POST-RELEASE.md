# 1.6.2 independent repository maintenance

## Authorization and timing

The owner requests these operations only after public 1.6.2 delivery. Both
1.6.0 publication and independent three-language 1.6.2 acceptance are complete. Tag
accepted 1.6.2 feature source and use the existing main-branch release workflows;
merge 1.6.2 into main after publication. The snapshot branch is
`milestone/1.6.2`, corrected by the owner on 2026-10-11.

## Scope and constraints

- Preserve the dirty root worktree, unrelated edits, copyright/licenses,
  immutable release tags, public assets, acceptance evidence and gh-pages.
- Remove only audited unused source, configuration and documentation. Retain
  shared terminal/editor/agent compatibility types and required pinned Git
  dependencies; independent maintenance does not mean rewriting dependencies.
- The four tracked workflows remain necessary: both release platforms and
  desktop/three-platform remote validation. The dynamic Pages workflow serves
  the active site and must remain.
- The independent gh-pages site's shared English/Chinese strings still advertise
  1.5.0 in eight download/version labels. After public 1.6.2 delivery, update those
  existing labels and verify links, font glyph coverage and Pages deployment;
  preserve its layout, assets and separate history.
- Existing feature/patch/release branches are ancestors of 1.6.2. Delete their
  completed remote refs after main contains them. Delete local refs only when
  unoccupied or after safely removing clean owned worktrees. Preserve the
  currently occupied dirty root branch and independent gh-pages history.
- Remove the local upstream remote and its tracking refs after maintenance.
  Preserve original authorship records rather than rewriting Git history.

## Concrete cleanup candidates

Replace inherited GitHub issue/support routing and private-team PR instructions
with Warpai-owned bug/feature forms and one PR template. Remove the two obsolete
specialized Warp SSH templates and two redundant PR templates. Replace inherited
STAKEHOLDERS with the independent owner; retain the local triage taxonomy used
by project skills. Remove Dependabot's upstream private registry and Warp team
reviewers; retain the existing public dependency/security update behavior.
Replace CONTRIBUTING.md's obsolete Oz/team contribution promises and upstream
issue routing with the existing Warpai contribution and validation rules.
Reduce WARP.md to links to AGENTS.md/README.md/MEMORY.md: its private server,
cloud product and nonexistent binary instructions conflict with those sources.
Keep the agent-recognized filename rather than maintaining duplicate guidance.

Development setup still calls `script/install_channel_config`, which contacts
Warp's private channel-config repository. Its callers are `script/run` and
`script/install_cargo_build_deps`; the latter also serves test/release/platform
setup. Remove these automatic upstream installation attempts and make the public
run entrypoint select Warpai's OSS binary consistently. Remove the private
installer helper once its two setup callers and internal error-message reference
are updated; the error should direct standalone users to the public binary.
Keep explicit inherited
channel targets/configuration compatibility until a separate reachability and
cloud-build gate proves they can be removed. Verify argument forwarding with
stubbed commands, without compiling or launching the application locally.

Review obsolete cloud-only specs APP-3801 (upstream account authentication) and
GH1066 (bundled cloud onboarding autonomy) against all source/build/script/docs
consumers before deletion. Retain active terminal/tab/editor specifications and
source-linked engineering records. Prior deep cleanup already removed proven
unreachable cloud sources; do not delete compiled shared modules by their names.
The tracked-reference audit found no external APP-3801 consumers and only a
GH1066 example link in CONTRIBUTING.md; remove that example with the obsolete
specs while retaining GH408/GH1063. Update the macOS run-script setup comment
when removing automatic private installation. Retain external-generator channel
compatibility without a bundled upstream installer or automatic repository fetch.

## Fork relationship

GitHub reports a public fork of terzigolu/warp-lite, administrator access, no
child forks and approximately 169 MiB repository size. The official leave-fork
network feature is eligible on those facts, but the documentation warns about
permanent loss of repository metadata. The public REST/GraphQL tooling exposed
here has no detach mutation; local Orca computer-use CLI is unavailable.
Prepare a complete reviewable metadata/asset backup and inspect the actual
GitHub confirmation before detachment. Do not delete/recreate the repository or
change visibility as a workaround; those actions threaten releases and Pages.

Reference: https://docs.github.com/en/pull-requests/how-tos/work-with-forks/detaching-a-fork

## Acceptance and sequence

1. Verify public 1.6.0, then source-matched 1.6.2 builds, tests, three-language
   native screenshots, tagged installers, checksums and provenance.
2. Fast-forward main to accepted 1.6.2, then apply reviewed cleanup without
   changing published runtime behavior. Validate YAML, links, inventories,
   scripts and affected cloud checks; complete build/native checks if code or
   build inputs change.
3. Record both release receipts and repository decisions in MEMORY.md. Create
   `milestone/1.6.2` at the final verified maintenance commit and push it.
4. Remove merged stale refs and upstream tracking. Verify main/milestone source
   equality, retained tags/assets/Pages and absence of upstream automation.
5. Detach through a supported GitHub operation only after metadata consequences
   are concrete and authorized. Verify fork=false and public delivery afterward.

Status: preparation only; no cleanup, branch deletion, upstream removal or fork
detachment has been executed.

## Current delivery checkpoints — 2026-10-11

1.6.0 source `80e716617bf88a1fd363bb15e4083fec94021469` passes full desktop,
Windows rlto/release_bundle and native acceptance in run 38119276766. Downloaded
macOS 274/Windows 280 PNGs match that source with clean diagnostics. The previous
15a29850 candidate failed the Windows release build through a debug-helper cfg
mismatch. Old assets/tag metadata are backed up in
`/private/tmp/warpai-release-audit-1.6.0`. The owner explicitly authorized
withdrawal of that unpublished draft/tag on 2026-10-11. The replacement tag
points to 80e71661; installer run 38131069169 passes macOS and all three
Companion targets. Windows publication run 38134098256 also passes. v1.6.0 is
public at 2026-10-11T12:15:58Z with five installers and both checksum lists.
All five installer hashes pass; the Windows public installer matches the CI
artifact and its release-source.txt is 80e71661. macOS app version/bundle/icon/
signature and Linux/macOS Companion payload/source/runtime digests also pass.
New-candidate receipts are in `/private/tmp/warpai-release-audit-1.6.0-80e71661`;
the earlier 15a candidate backup remains separate and intact.

1.6.2 source dd55235d includes the original 80e71661 ancestry and the release
compile gate. Native fixed-label corrections also cover shared pane actions,
exported keybinding constants, slash-command prefixes and SSH status wrappers.
Each capture locale now owns its own short SSH master socket. Localized selector
defaults and production update formatting are covered, and the language menu
closes before later full-page captures. Run 38126828908 has passed macOS;
972 source-matched PNGs passed decoding and three-language visual review.
Windows also passed with 990 valid PNGs; settings/update/native-menu visual
review passed. Live WSL Review still showed the English header fallback and
uncommitted target label. Source b7a6a6da fixes those display producers and adds
a localized Head-target assertion to the existing WSL authority gate. Full run
38132366093 finds an assertion caller context mismatch on Windows. Source
a822977c makes the existing read-only diff-target producer accept AppContext;
run 38135111782 passes macOS; its 972 source-matched PNGs confirm the Review
labels, but reveal an English successful-save toast. Source ea9b68a4 localizes
that shared toast and 39 newly classified String::from display literals. Strict
inventory passes with 2704 pairs, 2820 marked sites and zero unclassified direct
display literals. Full run 38139339617 is queued for both desktops and WSL.
Earlier candidates must not be tagged. Complete source-matched artifact review
and publication before main integration, cleanup or milestone creation.

Owner authorized starting release compilation once macOS acceptance passes.
Source 2580c5ef passes macOS in run 38139771778: 972 PNGs, clean diagnostics,
and three-language Review/save-toast visual review pass. Tag v1.6.2 points to
that source. Release build 38144883701 uses auto_publish=false so publication
waits for Windows acceptance and all installer/source/checksum verification.
Windows acceptance remains pending. No main integration or cleanup has begun.

Run 38139771778 also passes Windows, with 990 valid source-matched PNGs and
clean diagnostics. Menu/settings/loaded Review labels pass; the Traditional
Chinese WSL screenshot exposes an English loading Review header. Source
0b109b87 corrects that producer and the additional summary header found by
header_text inventory. Strict coverage passes with 2706 pairs and 2822 sites.
Full run 38147733290 is pending; previous release build 38144883701 was
requested canceled and must not be published. Its Companion artifacts are
retained in /private/tmp/warpai-release-audit-1.6.2-2580c5ef/companions.
The unpublished v1.6.2 candidate still points to 2580c5ef; prepare its metadata
backup before replacing it with the next accepted source.

Monitoring correction: a single delayed tool query does not repeatedly wake
the chat agent to inspect results. Install dev.warpai.ci-monitor as a user
LaunchAgent with StartInterval=1800 and RunAtLoad. The independent read-only
Python monitor uses gh to query selected runs, stores only sanitized summaries
in ~/Library/Application Support/warpai-ci-monitor/latest.json and history.jsonl,
and does not read credential files or raw logs. Self-check and the first
launchd-owned query passed. It records results while this Mac is awake/logged
in; it cannot wake the chat agent or repair source automatically.
MacOS acceptance passes for 0b109b87 in run 38147733290; Windows captures are
in progress. Preserve the old annotated v1.6.2 tag in refs/warpai-release-backups/
v1.6.2-2580c5ef and tag metadata/Companion artifacts in the audit directory.
The old release run is confirmed cancelled and no v1.6.2 release exists.
Replace the unpublished candidate tag with 0b109b87 and start release run
38152318120 with auto_publish=false under the owner early-build instruction.
The background monitor now watches both that build and acceptance run.

Owner correction: remove the macOS background Python monitor entirely.
The launchd job was booted out and the owned plist, Python/config/status/log
files were deleted; both launchd and filesystem absence were verified.
Use active-session half-hour cloud polling only; do not reinstall a local
background monitor. Run 38147733290 now passes both desktops for 0b109b87.
Release 38152318120 passes all three Companion builds; macOS app build is
pending. Retain publication/installer verification gates and post-publication
main integration timing. The feature worktree has a documentation-only MEMORY
change recording the monitoring correction; preserve it for the final merge.

Latest 0b109b87 acceptance artifacts are decoded and source-matched: 972
macOS PNGs (324 per locale), 990 Windows PNGs (330 per locale), clean exit/
assertion/auth/deadline diagnostics. Three-language actual code-pane/remote-file/
terminal/tab menus and loaded SSH/WSL Review labels pass visual review; macOS
save toasts are localized. Loading placeholder translation is checked at its
shared producer/catalog and by strict header inventory. The release remains
gated on installer and payload verification; do not retag published releases.

Prepared cleanup.patch in /private/tmp/warpai-post-release-prepared-1.6.2
against 0b109b87: 12 targeted replacements and 9 audited deletions. It is not
applied to source. YAML/issue-form shape, retained Cargo security-only and
Actions cooldown/group behavior, script syntax, Rust parse and patch
applicability checks pass. Stubbed macOS/Windows script/run tests preserve
argument boundaries/spaces/features/profile forwarding and always choose the
standalone OSS binary even when a private config generator is on PATH.
No application, Cargo build or network install ran locally. Apply only after
public release and main integration, then complete affected cloud checks.

Active-session polls executed at 2026-10-11T16:24:05Z and 16:54:05Z.
At the latter, release run 38152318120 passes all Companion and macOS app
jobs. Automatic Windows release run 38155542555 started at 16:27:33Z and
is packaging the tagged source. v1.6.2 remains a draft with four installers
and the macOS/Companion checksum list. Downloaded assets match all four hashes
and the three Companion CI artifacts. Read-only macOS app inspection verifies
1.6.2, bundle ID, source branding and strict signature; no application or
installer was executed locally. Receipts: /private/tmp/warpai-release-audit-
1.6.2-0b109b87. Windows/publication/main/cleanup gates remain pending.

## Delivery and applied maintenance receipt (2026-10-12)

- Public v1.6.0: source 80e716617bf88a1fd363bb15e4083fec94021469;
  native acceptance 38119276766; packaging 38131069169 and 38134098256.
- Public v1.6.2: source 0b109b87c9a0e19f276ff8c86eb94202b85ee9ff;
  native acceptance 38147733290 (972 macOS and 990 Windows PNGs), packaging
  38152318120 and 38155542555. Published 2026-10-11T17:57:00Z.
- All five installers, both checksum lists, source records, macOS bundle identity,
  branding and signatures were verified. Companion remains 4.0.0/protocol 1.
- Main was fast-forwarded to the public 1.6.2 source after publication.
- Applied the audited twelve replacements and nine deletions above. Script
  argument forwarding passed macOS/Windows stubs without local app execution.
- The owner-requested macOS Python monitor, LaunchAgent and owned support directory
  were removed. Subsequent half-hour checks occur in the active agent session.
- Cloud validation of the maintenance commit, milestone creation, stale branch
  removal and upstream removal remain delivery gates. GitHub fork detachment is
  still unperformed because no supported callable operation is available.
