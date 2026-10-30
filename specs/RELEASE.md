# Warpai releases

## 1.8.1 delivery contract

Owner authorization on 2026-10-30 resumes release work as 1.8.1 after macOS
acceptance of the pane-icon correction. Merge the accepted branch and begin
formal packaging; preserve the unpublished immutable v1.8.0 tag at 56493761.
Do not resume its canceled release job or move its tag.

Runtime/test source 693559e1 passed macOS and Windows default/platform builds,
focused regressions and native captures plus both protocol jobs in run
41053972438. Both native archives contain 1008 PNGs with valid structure/CRCs;
representative icon/menu frames were reviewed. Subsequent changes affect memory
and release documentation only. Immutable v1.8.1 points to
8ff0c00f41a096fc032e5caef6fe204773a4fbbf; formal release run 41060413768 passed
all three Companion jobs. Retain every tagged desktop/Companion installer,
provenance and checksum gate. Keep the release private until all 13 installers
and four checksum inventories are verified.

Release notes describe pane-owned icons, builtin/local SVG/PNG choices, existing
tab config TOML save/reopen, SSH badges, PAC and Qoder usage queries. Companion
remains 5.5.0; SSH protocol major 1 and Agent Bus major 2 remain unchanged.
After public assets resolve, update both READMEs, current source guides,
bilingual gh-pages home/docs/strings/footer and Homebrew. Product documentation
describes current usage, without changelog or migration narratives. Verify the
production Jekyll build, internal links, native captures, live deployment and
independently downloaded installer hashes. No local Rust builds or personal
application/configuration changes.

### GitHub release-service recovery

The first attempt of macOS run 41060413768 passed all compilation, test,
packaging, signing and DMG gates, but GitHub returned HTTP 500 while creating
the draft. Both tag lookup and release-list inspection confirmed that no
v1.8.1 draft or assets were created. Formal macOS installers are not retained
as standalone workflow artifacts, so rerun only its failed desktop job; reuse
the three successful Companion jobs. Do not move the tag or bypass gates.
Windows run 41072512977 and Linux run 41067669163 build the same tag in parallel
with automatic publication disabled. Verify all assets before publication;
avoid duplicate automatic platform builds after recovery.

## 1.8.0 delivery contract

Publication is on hold per the owner's subsequent 2026-10-30 instruction.
Preserve immutable v1.8.0 at 56493761; release run 41037908298 was canceled.
The pane-icon/tab-config correction must be verified before any new release
authorization. Prepared 1.8.0 download pages remain local and unpublished.

Owner authorization on 2026-10-30: merge PR #8 after cloud acceptance and publish
1.8.0. Synchronize both READMEs, current source guides and bilingual website/docs
in place. Keep change history in the Release notes, outside README, the website
and user guides.

Accepted runtime source `28342ace849d577053179a1d44efc49e68bcef3c` passed
[communication validation](https://github.com/OthinusG/warpai/actions/runs/41025311966)
on macOS and Windows, including both protocol jobs. The PR head differs only in
MEMORY.md. Companion remains 5.5.0, SSH protocol major 1 and Agent Bus major 2.

Reuse the immutable-tag macOS, Windows and Linux release workflows and the
existing release-notes source convention. Keep all native build, installer,
source, version and checksum gates. Publish only after all platform packages
are attached and verified, then update public download links and Homebrew.
Describe vertical tab icon selection, local SVG/PNG discovery and automatic SSH
badges as current usage. Preserve layouts, screenshots and prior release assets.

Acceptance: all 13 desktop/Companion installers, checksum inventories and source
manifests match v1.8.0; bilingual download URLs resolve; production Jekyll build
and internal links pass; GitHub Pages deployment succeeds; the Homebrew cask
uses an independently verified macOS DMG hash. No local Rust builds or changes
to personal application installations, CLI configuration or credentials.

## 2026-10-21 MCP adapter migration and 1.7.8.2 bugfix

The owner authorized fixing persisted MCP argv migration and publishing 1.7.8.2
as a bugfix. Migrate local, SSH and WSL records so each adapter's `server_name`
matches its `add`, `get`, and `remove` arguments; let inactive cleanup records
be selected again. Preserve unrelated MCP entries and activation preferences.
The local QoderCN installation was separately repaired by registering only
`warpai-local-com`; its CLI confirmed the bridge command. Companion remains
5.5.0; SSH protocol major 1 and Agent Bus major 2 remain unchanged.

Validate the focused Agent Bus regression and all existing cloud release gates
against immutable tag `v1.7.8.2` before publication. After public assets and
checksums exist, complete the README, bilingual site, and Homebrew updates below.

## 2026-10-21 MCP scope names and 1.7.8.1 publication

The owner authorized publication of Warpai 1.7.8.1 after cloud acceptance.
Publish the accepted MCP scope-name change: local `warpai-local-com`, SSH
`warpai-ssh-com`, and direct WSL `warpai-wsl-com`; preserve unrelated MCP
registrations and remove only recognized legacy Warpai entries. WSL does not
inject a host MCP configuration. Companion remains 5.5.0; SSH protocol major 1
and Agent Bus major 2 are unchanged.

This four-part patch version must work in release tag validation, all platform
packagers, Linux manifests, and stable update ordering. Existing three-part
versions remain valid and compare as revision zero. Publish from the accepted
source through `release-macos.yml` with Windows and Linux packaging follow-ups.
After all platform assets and checksums are public, complete the README, bilingual
site, and Homebrew updates in “Mandatory final step after public release assets
are available.”

## 2026-10-20 WSL recovery and 1.7.8 publication

The owner confirmed real-machine acceptance covering SSH connections to Linux,
macOS and Windows hosts and local WSL, then authorized merging
`fix/windows-bootstrap-size` and publishing Warpai 1.7.8. Scope: shorten the
Windows PowerShell SSH seed below the console input limit to remove the
"Starting shell..." hang, restore WSL auto-warpify when entering `wsl`, keep Zsh
bootstrap input hidden in raw mode, and carry the container `safe.directory`
packaging fix already on `main`. Companion remains 5.5.0; SSH protocol major 1
and Agent Bus major 2 are unchanged. The release notes must state that SSH to
Linux, macOS, Windows and local WSL was tested on real machines.

Acceptance: branch source 67d3e83b passed
[Validate Warpai communication](https://github.com/OthinusG/warpai/actions/runs/39367252504)
and produced the Windows test installer used for owner testing in
[Release Windows x64](https://github.com/OthinusG/warpai/actions/runs/39374934247).
The Linux packaging correction was verified in container run 39383298426.
Publish through `release-macos.yml` with `release_tag=v1.7.8` and
`auto_publish=true`; Windows and Linux follow from the immutable tag. Retain all
installer, provenance, checksum and smoke gates, verify public assets and
checksums, then complete the README, bilingual site and Homebrew steps below.

Published 2026-10-20 from tag `1747f21a` (`b050715e` source): macOS run 39384574699,
Windows run 39393646830 and Linux run 39405040826 all completed successfully. The
release became public at 10:02:08Z and the Linux packages attached at 11:29:22Z.
All 19 assets matched their published SHA-256 digests, every download URL returned
HTTP 200, and an independently downloaded macOS DMG matched
`d9f5ca03e7539cb6cfb1b6eaf55d68354f67d35a35f6092d660e0cf6d2e60a83`. Public
updates: README commit 1cd02900, site commit dc555f41 on `gh-pages`, and Homebrew
tap commit 3bc0340 (cask 1.7.8, `brew style` and online `brew audit` clean).

## Corrected 1.7.7 publication

Owner authorization on 2026-10-20: stop the earlier cloud runs, fix the Zsh
bootstrap input echo issue on `main`, validate the correction, and publish the
corrected release. The immutable `v1.7.6` tag and draft point to the original
source and must not be moved or have their assets replaced. Publish the combined
1.7.6 feature set plus the Zsh fix as immutable `v1.7.7` after source-matched
cloud validation passes. Check cloud run status with `gh run view` no more often
than once every 30 minutes; fix failures and rerun the affected validation.

The Zsh acceptance contract is in “Zsh bootstrap visibility” below. Retain the
existing macOS, Windows, Companion, and Linux package gates. After publication,
verify all assets and checksums, then complete the public download and Homebrew
updates described in “Mandatory final step after public release assets are
available.”

## 2026-10-19 combined 1.7.6 acceptance (original draft retained)

The original owner instruction authorized the formal 1.7.6 release after
acceptance, and its source/tag/draft are preserved unchanged. The Oct20 Zsh
bootstrap correction is being released separately as v1.7.7 per the contract
above. Cloud acceptance passed on source `c6c2967f`: full macOS/Windows communication,
Windows SSH and WSL, Companion macOS/Linux/Windows, and Linux x64/ARM64 packages.
Receipts: [communication](https://github.com/OthinusG/warpai/actions/runs/39259499684),
[Companion](https://github.com/OthinusG/warpai/actions/runs/39254214995),
[Linux packages](https://github.com/OthinusG/warpai/actions/runs/39249589503).

### Windows protobuf compiler provisioning

Windows CI and release builds must not depend on the Chocolatey `protoc` package:
its NuGet feed returned HTTP 503/504 during the accepted source's main-branch
validation and first release attempt. Download the official Protobuf v36.2
`protoc-36.2-win64.zip` asset from GitHub Releases, verify SHA-256
`f0c128dc0d8492eceece83bb459a4c0e316764b929ffbf1aa416357fd644edd3`, then set
`PROTOC` and `PATH`. Keep Inno Setup provisioning independent. Acceptance requires
all Windows protocol, desktop, Companion, and release build workflows to use the
verified compiler and rerun successfully.

### Zsh bootstrap visibility

Keep local and SSH Zsh PTYs in raw mode with ZLE disabled while Warpai injects
bootstrap input. Restore sane terminal settings after the heredoc is read and
before evaluating the bootstrap, so users do not see initialization source and
their prompt and rc files receive normal terminal behavior.

Owner authorized canceling the pending 1.7.2 publication and publishing its Qoder
changes together with the three-desktop SSH selection repair after tests pass.
Scope additionally includes the SSH Remote master switch and per-account native
MCP setup in agent-communication-v2/SSH-MCP-SETTINGS.md. Connection-only packages
are interim tests. The completed feature requires Companion 5.5.0.
Retain the immutable v1.7.2 tag and its unpublished draft; never replace its
already built assets with different source. Use a new immutable v1.7.6 tag.

Require macOS/Windows application and native UI/WSL acceptance, focused shared
SSH selection regressions on macOS/Windows/Linux, and Linux x64/ARM64 package
verification/smoke tests. Preserve the existing WSL isolation regression and
add the confirmed-master regression separately. Source/run receipts live in
agent-communication-v2/INSTALLATION.md. Test builds use artifact-only mode.

Earlier instruction: cancel obsolete artifact runs 39211040394,
39211044793 and 39211048507. After validation, build only macOS and Windows test
installers in artifact-only mode. Wait for the owner's test result and explicit
instruction before starting formal release workflows, main integration or tagging.
Owner subsequently requested prioritizing a macOS test package despite the
Windows capture failure. macOS acceptance passed in run 39212818285; artifact-only
macOS packaging was dispatched as 39224657289 at source 657db07c. The Windows
capture wrapper exited 1 without a classified diagnostic; this is not full
cross-platform acceptance. Formal publication remains on hold.

After full cloud acceptance, integrate the reviewed changes into main and publish
v1.7.6 through the existing workflows. Companion advances to 5.5.0 for SSH native MCP setup; the connection repairs also
apply to all desktop targets. Retain all source/version/checksum/native installer gates and
verify both Linux architectures. Complete README, bilingual website and Homebrew
updates using the mandatory final steps below after public assets are available.
Do not change the owner's installed desktop, credentials or personal settings.

## Mandatory final step after public release assets are available

After the release workflow has published the desktop and Companion assets, and
the expected download URLs resolve, synchronize the public download surfaces:

1. Update `README.md` and `README.zh-CN.md` with the release version, feature
   summary, desktop and Companion asset names, and direct download links.
2. Update the bilingual gh-pages homepages, shared header/footer release copy,
   `_data/strings.yml`, and English/Chinese installation docs with the new
   version and links. Preserve Jekyll `site.baseurl` for internal documentation
   links.
3. Verify every referenced public asset exists, review the changes for stale
   previous-version download URLs, and wait for the gh-pages deployment to
   succeed before marking release delivery complete.
4. As the final release action, update `OthinusG/homebrew-tap`'s
   `Casks/warpai.rb` to the published macOS DMG version and independently
   verified SHA-256. Submit the tap change, run the available Homebrew parsing,
   style/audit and download checksum checks, then merge and record the tap
   commit.

Do not point public pages to a new release before its download assets exist.

## 2026-10-15 1.6.5 / Companion 5.3.0 delivery contract

Owner authorized all-Agent Herdr state adaptation, MCP repair across local/WSL/SSH,
latency repair preserving communication, copyable names and removal of tab dots.
Shared Companion code changed, requiring 5.3.0. SSH protocol major 1 and Agent Bus
major 2 remain unchanged. This contract supersedes earlier monitoring/validation
exceptions for this release.

Require source-matched protocol, both desktop builds/native UI/real WSL acceptance,
and all three Companion platforms before immutable v1.6.5 publication. Reuse the
existing release workflows, preserving installer/provenance/runtime/checksum gates.
Monitor hourly; resolve actual failures before verifying all five public installers,
both checksum files and website download links. No personal installation/configuration
changes or local Rust builds.

Acceptance: source `9cc8d0b25baa9f6258fa8b3098a09767b8dadc41` passed complete
macOS/Windows protocol, application builds, native UI and real WSL gates in
[38667669349](https://github.com/OthinusG/warpai/actions/runs/38667669349).
Native capture diagnostics report exit 0 with no failed steps/assertions: 984
macOS PNGs and 1011 Windows PNGs. Reviewed representative participant/worktree
views in English, Simplified Chinese and Traditional Chinese, plus live WSL.
All three Companion platforms passed
[38667672552](https://github.com/OthinusG/warpai/actions/runs/38667672552).
The owner explicitly requested triggering the formal 1.6.5 publication workflow
on 2026-10-15 after these gates passed. Immutable `v1.6.5` points to `4b6aa95e`
(accepted runtime source plus documentation receipts). Formal release started in
[38682643407](https://github.com/OthinusG/warpai/actions/runs/38682643407) with
`auto_publish=true`; Windows follows the same tag after macOS succeeds.
Installer checks and public publication remain pending.

## 2026-10-14 direct 1.6.4 / Companion 5.2.0 publication

The owner explicitly requested publishing Warpai 1.6.4 with Companion 5.2.0
without further validation runs. This supersedes the standalone validation
prerequisite below. Cancel the pending Windows follow-up; preserve the existing
release workflow's source/version/installer/checksum gates. Do not move tags.

Desktop and native UI source ed9d9dd1 passed macOS and Windows native captures
in run 38551750559, including real WSL workspace and installation isolation.
Its only failure was the Windows PNG count (1008 expected, 1011 produced), fixed
in 8e38b352. All three Companion platforms passed run 38547390880; subsequent
runtime changes are limited to the shared desktop context selector and translations.

Update all active runtime/WSL package expectations and public Companion links
to 5.2.0; include native task threads, bootstrap resources and reviewed project
memory in the notes. Integrate source into main and create immutable v1.6.4,
then dispatch release-macos.yml with auto_publish=true. Windows follows from
the same tag and completes publication. Monitor every 30 minutes, resolve actual
packaging failures and verify all five public installers/checksums before delivery.

## 1.6.4 delivery contract

Owner authorization 2026-10-13: package and publish Warpai 1.6.4, updating
Companion to 5.1.0. Retain SSH protocol major 1 and Agent Bus protocol major 2.

Owner authorization 2026-10-13: create Windows and macOS test installer artifacts
from the maintenance branch without publishing them. The release remains subject
to all gates.

1. Finish source-matched desktop protocol/build/application/native UI/WSL
   acceptance and all three remote Companion jobs. Preserve every existing gate.
2. Update the Companion constant, WSL package checks and fixtures to 5.1.0;
   desktop versions derive from immutable v1.6.4. Prepare bilingual README links
   and docs/releases/v1.6.4.md. No personal installation or local Rust builds.
3. Integrate the accepted source into main, preserving unrelated worktree changes.
   Create immutable v1.6.4 and dispatch release-macos.yml with release_tag=v1.6.4
   and auto_publish=true. Windows follows from the same tagged source.
4. Retain all five installer/source/version/runtime/reinstall/checksum gates.
   Monitor at the owner's 30-minute cadence and resolve failures before delivery.
5. Verify public assets, checksums, tagged provenance and download links; update
   website downloads and record completed delivery here and in MEMORY.md.

## 1.6.3 delivery contract

Owner authorization 2026-10-13: check cloud results every 30 minutes, finish all
1–17 fixes and additional communication/WSL/SSH requirements, then publish Warpai
1.6.3 with Companion 5.0.0. This supersedes older pending release instructions.

1. Resolve both desktop native capture failures without bypassing assertions.
   Require one complete source-matched macOS/Windows protocol/build/application/
   native UI/real WSL acceptance run, including installation/removal recovery,
   plus all three Companion platform jobs. Review all three UI languages.
2. Set Companion RELEASE_VERSION and WSL package/test expectations to 5.0.0;
   preserve SSH protocol major 1 and Agent Bus major 2. Desktop derives 1.6.3 from
   the immutable tag. Record feature dependencies in public documentation.
3. Prepare bilingual README/download updates and v1.6.3 notes. Integrate accepted
   source into main without losing unrelated changes; create immutable v1.6.3.
   Preserve historical tags, assets and the gh-pages website.
4. Use release-macos.yml with release_tag=v1.6.3 and auto_publish=true; the existing
   Windows release workflow follows. Retain all desktop/Companion installer,
   source/version/runtime/reinstall/signature/checksum gates. Poll every 30 minutes
   and fix failures; dispatch alone is not acceptance or publication.
5. Verify all five public installers, both SHA-256 lists, exact tagged provenance
   and public links, then update website download/dependency notes and delivery
   receipts. No personal installation or local Rust compilation.

Acceptance receipt: immutable v1.6.3 source 67057c0a2de9de96ec1d781661551a07d437215a.
Run 38323386419 passed both desktop protocol, build and native UI gates, including
real Windows WSL installation/reinstallation/uninstallation with SSH isolation.
Run 38323390269 passed all three Companion platform gates. Capture diagnostics
reported exit 0: macOS 981 PNGs (327 per language), Windows 1005 (335 per language),
covering English, Simplified Chinese and Traditional Chinese. Formal packaging
started in run 38333613421; publication and website receipts remain pending.
Main includes the accepted source. Maintenance branch is milestone/1.6.3; the
merged milestone/1.6.2 and former centipede-tepui remote names were removed.

## 1.6.0 delivery contract

Owner authorization 2026-10-11: after acceptance passes, trigger both desktop
release workflows for Warpai 1.6.0 with Companion 4.0.0. Add no telemetry.

1. Finish the Windows WSL capture correction and one source-matched complete
   macOS/Windows acceptance run. Inspect diagnostics and the 274 macOS / 280
   Windows PNGs, including all six WSL views and installation/removal isolation.
2. Prepare v1.6.0 notes and README downloads. Companion RELEASE_VERSION remains
   4.0.0 and protocol major remains 1; desktop versions derive from the tag.
3. Fast-forward main to the accepted source and release documentation, retaining
   unrelated worktree changes and historical tags/assets. This also installs the
   existing WSL-aware Windows release workflow on the default branch, required
   by its workflow_run trigger. Create immutable v1.6.0; never move older tags.
4. Dispatch release-macos.yml with release_tag=v1.6.0 and auto_publish=true.
   Windows follows from the same tag and publishes after the native package gates.
   Preserve all three Companion native installer/runtime gates and both desktop
   version/source/package checks. Monitor at the owner's 30-minute cadence.
5. Verify all five public installers, both SHA-256 lists, source provenance and
   public links before reporting delivery. Record receipts here and in MEMORY.md.
   No personal installation, local Rust compilation or credential changes.

## Sequential patch delivery: 1.5.1, then 1.5.5

Owner authorization 2026-10-10 supersedes the earlier single-publication limit.
Publish 1.5.1 first, then 1.5.5; both use unchanged Companion 4.0.0 and protocol 1.
Add no telemetry. Do not begin 1.6.0 until the owner supplies its scope.

- 1.5.1 source: immutable tag v1.5.1 at 30ad76bb. Formal macOS/Companion run
  38022801330 uses auto_publish=true; Windows follows and publishes after gates.
- 1.5.5 source is prepared on release/1.5.5, retaining 1.5.1 ancestry and accepted
  native provider/UI implementation. Usage fixture visibility correction needs
  capture acceptance in run 38022957210 before tagging/publication.
- Retain native source/version/installer/reinstall/runtime checks and all five
  installers/checksum receipts. Confirm 1.5.1 public delivery before dispatching
  the 1.5.5 release; only the second release becomes latest afterward.

## 1.5.1 preparation contract (superseded publication limit)

Owner authorization 2026-10-10: complete both patch scopes, validate combined
1.5.5 directly, then package/publish the independent 1.5.1 source checkpoint.
Data usage belongs to 1.5.5 and must remain absent from 1.5.1. Retain historical
releases and the active workflows; do not resume the stopped 1.5.0 audit.

1. Keep 1.5.1 UI source on `patch/1.5.1`, with shared fixes backported from the
   combined implementation. Verify the checkpoint has no usage module, settings,
   footer or provider/credential dependencies and no Companion source changes.
2. Accept combined macOS/Windows builds, focused tests and native walkthroughs;
   record the immutable source/run and visual results in the patch contract.
   Do not repeat an intermediate 1.5.1 walkthrough. Apply shared fixes to both.
3. Integrate accepted 1.5.1 and release preparation into main. Keep Companion
   RELEASE_VERSION 4.0.0 and protocol major 1. Publish immutable `v1.5.1` through
   release-macos.yml with auto_publish=true; Windows follows existing gates.
4. Retain native source/version/installer/reinstall/checksum/runtime acceptance.
   Monitor cloud jobs at the owner's 30-minute cadence. Verify five versioned
   installers and both checksum lists before recording public delivery. No local
   Rust build, personal installation or automatic 1.5.5 publication is authorized.

## Historical delivery: 1.5.0 with Companion 4.0.0

Owner authorization 2026-10-10: merge all accepted fixes into main, clean unused
branches/workflows, and package/publish desktop 1.5.0 with Companion 4.0.0.

1. Merge the accepted correction branch into main, preserving local website
   memory commits as well as accepted code. Preserve historical tags,
   release assets, screenshots and the active gh-pages website. Delete completed
   development refs only after main contains them; remove only clean, unused
   native worktrees. The two release/two validation workflows remain necessary;
   keep Pages deployment and acceptance/release evidence.
2. Set Companion RELEASE_VERSION to 4.0.0, retaining protocol major 1. Desktop
   version derives from the immutable v1.5.0 tag. Update both README download and
   About/update sections, and write docs/releases/v1.5.0.md against public 1.3.1.
   No unrelated refactor, new packaging flow, local Rust build or installation.
3. Verify focused version/packaging checks and scripts, commit main, and create
   v1.5.0 without altering older tags. Dispatch release-macos.yml with
   release_tag=v1.5.0 and auto_publish=true. Existing native Companion install/
   reinstall/version/source checks precede the macOS draft; Windows follows and
   publishes only after its installer/version/icon/Agent-bridge checks pass.
4. Monitor cloud jobs at the owner's 30-minute cadence. Diagnose and fix failed
   gates without bypassing checks. Verify the five versioned public installers,
   both SHA-256 lists, exact source provenance, signed macOS bundles/DMG contents
   and public download links. Record delivery in MEMORY.md and this contract.

Acceptance baseline: source 46248e08 passes both desktop protocol/build/native
walkthrough jobs in run 37943970413, with 214 valid PNGs per OS (24 About states).
Application unit suites pass at production-equivalent 27a9f992 in run 37939850502.
Remote backend is unchanged from the three-platform accepted 15927aa3 source,
run 37920112411. The new component version requires fresh tagged package gates.

## Historical delivery: 1.3.1 with Companion 3.1.0

Owner authorization 2026-10-09: merge accepted Worktree collaboration into main,
remove completed branches and obsolete diagnostic workflow history, integrate
the feature into existing bilingual README sections, update repository About,
and publish App 1.3.1 with Companion 3.1.0. Preserve historical tags/assets.

1. Fast-forward main to the accepted feature source and release preparation.
   Keep the two release and two validation workflows; remove the retired Codex
   diagnostic workflow's runs. Delete completed branch refs only after main
   contains their commits. No history rewrite, local Rust build or installation.
2. Set the independent Companion component version to 3.1.0; protocol major
   remains 1. Update existing download/install guidance and add release notes
   describing changes from 1.3.0. Tag immutable main source as v1.3.1.
3. Dispatch release-macos.yml with release_tag=v1.3.1 and auto_publish=true.
   Reuse all native installer/source/version/runtime checks. Windows follows
   automatically and publishes only after its package checks and upload pass.
4. Check cloud runs every 30 minutes. Verify five versioned public installers,
   both checksum lists, tagged source provenance and About/download links.
   Record completed delivery in MEMORY.md; fix failed gates without bypassing them.

Source acceptance before version bump: Companion run 37830659603 passes all
three remote platforms. Desktop run 37830653334 passes macOS and Windows
protocol/default/platform/application regressions; Windows runtime preparation
fails on an occupied identical DLL. Workflow-only fix 48f1ca4a passes focused
Windows native run 37841105097. Each desktop has 191 verified captures; the
overall full-run failure is retained in the Worktree contract.

### 1.3.1 public delivery receipt

Published [Warpai 1.3.1 with Companion 3.1.0](https://github.com/OthinusG/warpai/releases/tag/v1.3.1)
as latest on 2026-10-09. Immutable tagged source:
`0312ac0f9ed6d9781fcd0b52c9f8d0b4d1c0e4c9`.

- [macOS and three-platform Companion release](https://github.com/OthinusG/warpai/actions/runs/37881956251)
  succeeds on attempt 2, preserving successful first-attempt Companion installers
  while retrying the desktop job after dependency executable/disk I/O failures.
  No source tag was changed and no check was bypassed.
- [Windows desktop release](https://github.com/OthinusG/warpai/actions/runs/37888146809)
  passes application/installer version, icon, Agent bridge and package checks,
  then uploads and publishes the complete release.
- All five downloaded installers and two SHA-256 lists match GitHub digests.
  Windows public bytes match the native CI artifact and immutable source receipt.
  The checksum lists cover exactly all five installers. Public notes match the
  committed changelog; all five unauthenticated download URLs return HTTP 200.
- Read-only macOS DMGs pass identity, version, deep signature, icons, launcher,
  source/target and payload checks; Linux source/target/version and all payload
  hashes pass. macOS App, bridge, Companion and private Git binaries declare
  minimum macOS 11.0. Bilingual README requirements specify Big Sur or newer on
  Apple silicon; this deployment minimum is not an older-OS execution receipt.
- Completed branches and the obsolete Codex diagnostic workflow were removed.
  The four current release/validation workflows remain; historical releases,
  tags, screenshots and branding are preserved. About and bilingual README
  describe both modes within the existing product sections. No personal-machine
  installation, local Rust compilation or credential/configuration edit occurred.

## Historical delivery: 1.3.0 with Companion 3.0.0

The owner authorizes packaging and public release of App 1.3.0 and Companion
3.0.0. Summarize the net changes from the v1.2.0 source baseline in
docs/releases/v1.3.0.md. Preserve all earlier tags, drafts and release assets.

1. Integrate the accepted self-contained remote tools into main, update the
   component version and bilingual download links, and tag committed source
   as v1.3.0. Desktop bundle/PE versions derive from this tag; protocol major
   remains 1. No local Rust compilation or personal-machine installation.
2. Dispatch release-macos.yml with release_tag=v1.3.0 and auto_publish=true.
   Reuse native packaging, checksums, three-platform install/reinstall and
   private Git checks. Windows follows automatically from the same tag and
   publishes only after its package identity checks and uploads pass.
3. Verify completed jobs, the public five-installer inventory, matching versions,
   source provenance and checksum files. Record delivery in MEMORY.md. Fix any
   packaging failure before publishing; do not bypass existing release gates.

Existing desktop builds/tests and native SSH Review/Code View assertions passed
on macOS/Windows. The corrected 170-image count passes against actual captures.
Three-platform self-contained Companion acceptance passed before this version
bump. Tagged release checks verify the new component version and formal packages.

### 1.3.0 public delivery receipt

Published [Warpai 1.3.0 with Companion 3.0.0](https://github.com/OthinusG/warpai/releases/tag/v1.3.0)
as the latest formal release on 2026-10-07. Immutable tagged source:
`e2964f3b49923ab44e219b7ece0ae6071f9fe3a6`.

- [macOS and three-platform Companion release](https://github.com/OthinusG/warpai/actions/runs/37606929158):
  all four jobs pass, including source-matched native installers, install/reinstall,
  private Git without system Git on PATH and Windows installed release protocol.
- [Windows desktop release](https://github.com/OthinusG/warpai/actions/runs/37613962878):
  application numeric/string versions, installer ProductVersion 1.3.0, icon,
  Agent bridge and package contents pass before automatic public delivery.
- Read-only inspection of downloaded macOS DMGs passes App version/identity,
  strict deep signature, Agent bridge, canonical artwork and Applications link;
  Companion source/target/version, every payload checksum and Finder launcher
  icon pass. Linux source/target/version and complete payload checksums pass.
- Public delivery contains five versioned installers and two SHA-256 lists.
  All public GitHub installer digests agree with those lists; Windows's public
  digest matches the downloaded verified CI installer and its tagged-source
  receipt. The public notes match the committed changelog exactly.
  Release notes cover the v1.2.0 baseline through v1.3.0; earlier tags and assets
  are preserved. No local Rust build or personal-machine installation occurred.

## Historical delivery: 1.2.2

The owner authorizes pushing the Review baseline correction and triggering
v1.2.2, without monitoring afterward. Retain Companion 2.0.0 and protocol major
1. Use docs/releases/v1.2.2.md and versioned desktop installer names.

Dispatch release-macos.yml from main with release_tag=v1.2.2 and
auto_publish=true. The option defaults to false for other releases. The macOS
workflow creates a draft only after its desktop and three-platform Companion
checks pass; its release-target artifact carries the explicit publication
choice. Windows follows automatically, verifies and uploads its installer,
then publishes the complete draft as latest when requested. Failed builds leave
the release unpublished. This owner instruction supersedes manual monitoring
and downloaded-package inspection for this delivery. Preserve historical tags.

Local focused checks passed; desktop checks were blocked by the missing Metal
compiler. Triggering automation does not constitute completed native acceptance.

## Historical delivery: 1.2.1

Publish `v1.2.1` only after the tagged desktop/agent regressions, native UI
review and three-platform remote installation/transport checks pass. Build
committed tagged source with the existing release workflows; do not compile
Rust locally.
The formal companion jobs reuse disposable-runner native install/reinstall checks
and exercise Windows binary stdio against the installed optimized executable.

Deliver `Warpai-1.2.1-macos-arm64.dmg`, `WarpaiSetup-1.2.1-windows-x64.exe`,
`WarpaiCompanion-2.0.0-linux-x64.run`, `WarpaiCompanion-2.0.0-macos-arm64.dmg` and
`WarpaiCompanion-2.0.0-windows-x64-setup.exe`, with SHA-256 checksum files. No desktop
or raw companion ZIP is a public release asset. Remote installers own only the
account's Warpai component and metadata; follow the
[installation contract](agent-communication-v2/INSTALLATION.md).

The macOS workflow creates a draft. Windows attaches its version/icon-checked
installer and leaves that draft private. Inspect downloaded formal packages,
source manifests, checksums, native identities and artwork before publishing
the complete release. Preserve all prior release tags and assets.

README images must be unmodified native macOS captures with vertical tabs and
Claude Warm Light, with exact source/run provenance. Native Windows independent
SSH authentication uses system keys/agent; Warpai does not store passwords or
modify vendor commands, shell profiles or SSH configuration. macOS ad-hoc
signing and unsigned Windows installation remain documented.
The desktop DMG contains the signed app, an Applications shortcut and the
canonical Warpai Finder volume icon. Verify these in the mounted formal image.

The 1.2.1 delivery completes the existing SSH Explorer, in-app editor/preview
and original Git Review workflow, simplifies daily Agent collaboration, and
includes the legacy MCP cleanup and actionable setup errors prepared after 1.1.
Ordinary Codex communication remains session-only. The explicit cleanup preserves
unrelated MCP entries and reports unsupported or failed cleanup.

The public baseline is 1.1.0; the 1.1.5 preparation was not published. Explain
changes from 1.1 to 1.2 in docs/releases/v1.2.1.md. Update the existing README
feature/setup/remote/download sections in place, retaining the existing native
screenshots; do not add a README changelog. The banner identifies the delivered
version, both desktop targets and all three remote targets. Remote copy describes
local/remote workflow parity, without upgrade history or transport internals.

### 1.2.1 delivery tasks and acceptance

1. Preserve the unpublished v1.2.0 tag at 210ebb4a as requested. Release Warpai
   1.2.1 with Companion 2.0.0; protocol major remains 1. Retain accepted runtime
   logic from 60ea2810 (desktop/native run 37569663988) and the accepted Companion
   backend (three-platform run 37498107083). Verify the component-version change
   and versioned packaging, then create the immutable v1.2.1 tag.
2. Dispatch release-macos.yml from main with release_tag=v1.2.1. Its three native
   Companion jobs verify tagged builds and install/reinstall before the macOS
   desktop build creates a private draft. The existing Windows workflow follows
   automatically and attaches its version/icon-checked installer.
3. Download all five installers and both checksum files. Check download hashes,
   exact source manifests, target/version identity, bundled Agent bridge, license
   notices and existing artwork. Mount both macOS DMGs read-only to check their
   Finder icons and contents. Retain native Windows installation/version receipts.
4. Publish the complete draft as latest only after these gates pass. Verify the
   public asset inventory and README download links, and record the immutable
   tag, jobs and delivery receipt in MEMORY.md. Preserve older releases.

## 1.2.1 public delivery receipt

Published [Warpai 1.2.1 with Companion 2.0.0](https://github.com/OthinusG/warpai/releases/tag/v1.2.1)
as latest on 2026-10-07. Immutable tagged source:
`4788cbde9a74987b53a3bd032b340cc38bd09fb7`.
The unpublished v1.2.0 tag remains at 210ebb4a as requested.

- [Formal macOS and three-platform Companion release](https://github.com/OthinusG/warpai/actions/runs/37577830960):
  all four jobs pass, including component-version probes, native installation/
  reinstallation, Windows Companion installer ProductVersion 2.0.0 and installed
  optimized Windows binary stdio. macOS desktop default/platform checks and the
  tagged release build pass.
- [Formal Windows desktop release](https://github.com/OthinusG/warpai/actions/runs/37581995715):
  native PE numeric/string versions, installer ProductVersion 1.2.1, icon, Agent
  bridge and archive contents pass. The downloaded public installer is byte-for-byte
  identical to its CI artifact; the source receipt matches the immutable tag.
- All five installers and both checksum files match GitHub SHA-256 digests.
  Both checksum files cover exactly the public installer inventory. Parse standard
  SHA-256 text/binary mode markers, including Windows's `*` marker.
- Both DMGs pass verification and read-only inspection. Desktop bundle identity,
  1.2.1 version, strict deep code signature, Agent bridge, canonical icon,
  Applications shortcut and Finder volume icon flag pass. macOS Companion's
  executable reports 2.0.0 protocol 1; its manifest, source/target/hash, launcher
  resource fork and artwork pass. Linux payload source/target/version, binary
  checksum, license notices and artwork pass.
- Release notes describe changes from 1.1 to 1.2. Bilingual READMEs update existing
  feature sections, identify local/remote platform support, explain remote file
  workflow parity and retain the original screenshots. All five download links
  match the public versioned installer names. No raw ZIP is a public asset.

| Public installer | SHA-256 |
| --- | --- |
| `Warpai-1.2.1-macos-arm64.dmg` | `84838a232ada10551ca24568be684f5b517f6e26772e9865ce0ab2605ed9afc9` |
| `WarpaiCompanion-2.0.0-linux-x64.run` | `460ae0c1a9f8fddf28179c4a339cbd7f08a1819a1e85c86e7483e672d9658fe0` |
| `WarpaiCompanion-2.0.0-macos-arm64.dmg` | `831a44fc5f1247c3a9e7a7e69978af8af53ec8b1a9b6de8828fcda07ad9fac8b` |
| `WarpaiCompanion-2.0.0-windows-x64-setup.exe` | `7099a0f2846a520e755e8f143b48ff993089ec5781c51bebd41c09f550743d36` |
| `WarpaiSetup-1.2.1-windows-x64.exe` | `bcf911bf21222995d74a50da7920434f91441a7b70ccb2dfa2ac89f06dc2f78c` |

## Historical releases

## Historical 1.0.1 preparation (not published)

> Superseded before publication by the owner's [1.1.0 iteration](agent-communication-v2/INSTALLATION.md). Keep the 1.0.1 draft private and preserve its source tag.

## Scope and acceptance

Publish the first renamed Warpai release as `Warpai 1.0.1`, tag `v1.0.1`.
The owner authorized publication after desktop/native and remote acceptance.
Rewrite both READMEs as complete product introductions with native screenshots,
features, installation, agent setup, SSH setup and stable release download links.
Preserve terminal guardrails, licenses and historical tags.

## Delivery plan

1. Accept the current icon source with full macOS/Windows checks and native
   screenshot review, plus Linux/macOS/Windows companion verification.
2. Reuse the two release workflows; build only tagged repository source. Derive
   bundle, installer and PE versions from the same release tag. No local Rust
   compilation and no changes to user CLI installations or configuration.
3. Build source-matched release companions for Linux x64, macOS ARM64 and Windows
   x64. Each archive includes its target, commit, binary version and checksums.
   Companion protocol versions remain independent of the product release version.
4. Create a draft after macOS packages and companions pass. Windows attaches its
   validated EXE installer and publishes the complete draft.
5. Verify public release assets, checksums, identities and documentation links;
   record acceptance and delivery in MEMORY.md.

## Packaging checks and risks

- macOS DMG must contain Warpai.app, `dev.warpai.Warpai`, version 1.0.1,
  the reviewed ICNS and the local MCP bridge. Verify ad-hoc signing and DMG.
- Windows installer must contain Warpai.exe, required native
  resources and the local MCP bridge. Application PE numeric/string versions and installer product
  version must be 1.0.1; the packaged icon must match the reviewed ICO.
- Remote packages must identify actual Rust target and tagged source, pass their
  native version invocation, and have verified SHA-256 manifests.
- Ad-hoc macOS signing and unsigned Windows installers can require OS approval;
  describe actual installation behavior without promising notarization.
- A failed packaging job leaves a draft rather than a partially public release.
  Preserve existing assets/history and fix failures before publication.

## Verification receipt

### Windows gate correction and installer-only delivery

The owner requested desktop installer images only: macOS DMG and Windows EXE,
plus all three companion archives. Remove desktop app/portable ZIP release assets
and update both READMEs, notes and checksum lists; retain the immutable source tag.

Run 37282889244 built the Windows packages but failed an incorrect installer
numeric file-version assertion. Inno's `VersionInfoVersion` defaults to 0.0.0.0;
its product text version defaults to `AppVersion`. See the official
[file version](https://jrsoftware.org/ishelp/topic_setup_versioninfoversion.htm)
and [product version](https://jrsoftware.org/ishelp/topic_setup_versioninfoproducttextversion.htm)
contracts. The application PE string-version assertions already passed.

Fix plan (diagnose-ci-failures / fix-errors):

1. Keep application PE numeric/string versions strict; verify the installer
   product version against the release version. Log actual version fields.
2. Preserve source-matched installer/checksum artifacts before the gate, allowing
   diagnosis without losing successfully compiled packages.
3. Publish only the EXE from Windows; remove the existing private desktop ZIP,
   correct checksums and notes, and keep DMG/companions already validated.
4. Validate workflow syntax, rerun native Windows checks, then inspect the public
   asset list/checksums and publish a complete release. No CLI installation edits.

- Icon/runtime source `5d2588a86bbdc05cc994ad816561c4626be9aac4` passed
  [desktop/native run 37268540992](https://github.com/OthinusG/warpai/actions/runs/37268540992):
  both application configurations, focused regressions, 161 images per OS,
  source-matched diagnostics and all native action assertions. Archive SHA-256,
  ZIP CRC and every PNG decode passed locally; selected native frames were reviewed.
- [Remote run 37268449581](https://github.com/OthinusG/warpai/actions/runs/37268449581)
  passed Linux/macOS/Windows. Packaging preparation source `ade8308` also passed
  [remote run 37270362606](https://github.com/OthinusG/warpai/actions/runs/37270362606).
- Release workflow actionlint, YAML, Bash and embedded Python syntax checks passed.
  Temporary macOS packaging smoke checked version 1.0.1, bundle identity, canonical
  ICNS, signing and invalid-version rejection using copied system placeholders.
  This smoke is a packaging check, not application/runtime acceptance.
- Publication additionally requires the cloud tagged-package checks above.
- macOS distribution uses `release_bundle,extern_plist`, matching the existing
  `script/macos/bundle` OSS path. This preserves release IME marked-text behavior
  and uses the actual installed Info.plist rather than the development embedded
  plist. Native UI acceptance already uses `release_bundle`. The release workflow
  executes from main while checking out the immutable tagged application source.
- Packaging-version source `ade83082b101ac624b62d90ea2c25d1a46b4fc65` passed
  [desktop run 37270362536](https://github.com/OthinusG/warpai/actions/runs/37270362536)
  on macOS and Windows, including both application configurations and all retained
  focused regressions. Subsequent pre-tag changes affect docs/screenshots and
  release-only archive checks, not application or companion runtime inputs.

- Tagged macOS/companion packaging passed [run 37277364402](https://github.com/OthinusG/warpai/actions/runs/37277364402). Downloaded macOS app inspection verified version 1.0.1, bundle identity, canonical ICNS, bridge and strict signing; the retained DMG passed SHA-256 and native image verification. All three companion archive manifests match immutable tag source `2bb47910ceadcf4ea47b4370166a6df7a34d2b96`, with valid binary/archive checksums.
- Installer-only workflow correction `594f015` passed actionlint, YAML, shell syntax and bilingual download-link checks. The private draft now contains only DMG, three companions and their checksum file; Windows rerun [37290225101](https://github.com/OthinusG/warpai/actions/runs/37290225101) must pass before publication.

### 1.1.0 Windows installer string normalization

Run 37362390293 built the tagged Windows application and installer with product
version 1.1.0, but Inno Setup pads its ProductVersion resource with spaces.
Normalize only that installer string with Trim before exact comparison; keep
application string/numeric checks and native resource checks strict. Dispatch
the corrected workflow from main while checking out the unchanged immutable
v1.1.0 source tag. Rerun the Windows release gate before uploading or publishing.

## 1.1.0 public delivery receipt

Published [Warpai 1.1.0](https://github.com/OthinusG/warpai/releases/tag/v1.1.0)
as the latest non-prerelease. Immutable tag source:
`574b2f97772cbd0f932128662445a686dd196a5e`.

- Full application regressions: run 37340966290; final native UI/actions:
  run 37349315123, 163 frames per platform with archive integrity and visual QA.
- Tagged macOS desktop and three native companion installers: run 37355772889.
  All remote install/reinstall checks passed; installed optimized Windows
  companion binary stdio tests passed.
- Corrected Windows desktop release: run 37411727676. Application string/numeric
  versions, normalized installer version, icon, bridge and archive contents passed.
- Downloaded all five formal installers and both checksum files. All hashes
  match GitHub asset digests; Windows installer is byte-identical to its native
  CI artifact and release-source receipt matches the immutable tag.
- Both DMGs passed native checksum verification and read-only mounted inspection.
  Desktop bundle identity/version, executable, strict deep code signature,
  reviewed ICNS, local MCP bridge, Applications shortcut and actual Finder custom
  volume-icon flag passed. Companion source/target/version manifests, payload
  hashes, artwork, launcher resource fork and actual volume-icon flag passed.
- Linux payload licenses, target/source/version manifest, executable checksum
  and companion artwork matched the tagged source. Windows setup PE/version
  resources passed; native CI performed the platform-specific installation checks.

| Public installer | SHA-256 |
| --- | --- |
| `Warpai.dmg` | `0ae508be02dc210b30d0c86dc2667b1518cc1afd5287cd45eda011573aa11f2a` |
| `WarpaiCompanion-linux-x64.run` | `85e4ba3b7d68195b1b9e6d1b631717c0eabe741183e5dec538f4f3dead08954e` |
| `WarpaiCompanion-macos-arm64.dmg` | `32b7c5cddadafbcdffd42d883b8ec7a288de776df1e47b1965077e5ca2da7d27` |
| `WarpaiCompanion-windows-x64-setup.exe` | `4ef9e059c87b8c1d4331135f35d21dc3b291907208720e874a336db22dc32148` |
| `WarpaiSetup-x64.exe` | `3893afe0e9285a24f0147391df035dbe5a8cd68eaa60e68361e32e31861546fb` |

No desktop or raw companion ZIP is published. Ad-hoc macOS signing and unsigned
Windows installation remain the documented first-launch constraints.
## Linux desktop packages (2026-10-16)

Owner-authorized Linux release engineering is specified in
[Linux scope and acceptance](linux-desktop/TECH.md). The standalone
`release-linux.yml` produces x86_64/ARM64 `.deb`, `.rpm`, Arch and AppImage
packages with a verified glibc-2.31 baseline. Native installers include the
communication bridge and Warpai desktop identity, never upstream update sources.
Source manifests and architecture-specific SHA-256 files accompany the packages.
Artifact-only is the default. Tagged attachment requires an existing release,
matching source, both architectures and no asset collisions; it cannot create
tags or change publication state. Successful Windows packaging triggers this
independent workflow with the same tag/source/test mode; communication validation
does not invoke Linux. Existing macOS/Windows/Companion release rules continue
to apply. Linux test builds are not full distribution/desktop acceptance.
