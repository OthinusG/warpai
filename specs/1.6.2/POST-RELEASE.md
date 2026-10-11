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
