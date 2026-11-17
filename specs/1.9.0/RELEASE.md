# Warpai 1.9.0 release ledger

Date: 2026-11-17. Contract: [release preparation](../RELEASE.md).

## Scope

Desktop 1.9.0 / Companion 6.1.0. Preserve SSH major 1, Agent Bus major 2,
existing identity/recovery safeguards and published artifacts. All Rust/native
acceptance runs in GitHub Actions. No personal installation/configuration changes.

## Gates

- [x] Website mobile header: bilingual controls, themes, drawer and desktop QA.
- [x] Companion version/WSL packaging checks and disposable installer tests (local).
- [x] Source-matched cloud protocol, application, native captures and real WSL.
- [x] Three-platform Companion wire validation.
- [x] Native screenshot and diagnostic review.
- [x] Immutable v1.9.0 tag and draft macOS/Companion build, auto_publish=false.
- [x] Windows x64 and Linux x64/ARM64 packages attached and verified.
- [x] Thirteen installers, checksum/provenance inventories and public-release gate.
- [x] Public download surfaces, production Pages and final Homebrew synchronization.

## Existing evidence

Main `1699828636cf125b43ecd46413918e5630f29984` passed communication
[44008543133](https://github.com/OthinusG/warpai/actions/runs/44008543133) and
Companion wire [44008543111](https://github.com/OthinusG/warpai/actions/runs/44008543111).
The push-triggered communication run did not request native captures or real WSL;
it cannot close those release gates or validate the pending 6.1.0 version change.

## Candidate and active acceptance

Runtime candidate: `f25421630ad3fb6dfce93de6325a9841e4afd1eb`.
Local WSL packaging/disposable installer provenance, repeat install, SSH isolation
and corrupt-payload rejection passed; shell syntax and diff checks passed.

- Full native/application/protocol/WSL:
  [44030913883](https://github.com/OthinusG/warpai/actions/runs/44030913883),
  capture_ui=true, check_app=true, wsl_acceptance=true, protocol_only=false.
- Companion wire:
  [44030884183](https://github.com/OthinusG/warpai/actions/runs/44030884183).
- Routine push validation:
  [44030884261](https://github.com/OthinusG/warpai/actions/runs/44030884261).

Submitted workflows are pending evidence, not passed gates. No v1.9.0 tag or
public release has been created. Companion 6.1.0 was explicitly confirmed by the
owner after the version decision was explained.

## Website acceptance

Website source `8eb59796a0868ba5e40886bb5b640cb46b3c2287` is deployed.
The header retains all controls in two rows at <=700 px, preserves desktop layout
and shares its height with the drawer offset. Local assertions passed 112
bilingual/page/width/theme combinations at 320/375/390/700/701/768/1440 px,
including control bounds/non-overlap, language targets and real-pointer drawer
open/close. Eight public-site 320/390-px checks passed after deployment. English
ink/Chinese paper 320-px screenshots were visually reviewed. Long inline commands
wrap without changing preformatted-block scrolling. Website specification:
`gh-pages:docs/MOBILE-HEADER.md`.

## Additional packaging preparation

Linux x64/ARM64 artifact validation was started on the exact runtime candidate:
[44031746164](https://github.com/OthinusG/warpai/actions/runs/44031746164),
release_tag=v1.9.0, artifact_only=true. No public release/asset mutation.
Local Unix staged-installation/reinstall/corruption tests and mocked macOS DMG
detach retry tests passed. Native Linux staging tests require GNU install and
were skipped locally; Linux CI remains their evidence source.

## Deferred public documentation

Prepared 24 source/website document drafts outside the repositories at
`/tmp/warpai-1.9.0-publication`. A manifest records each original content SHA-256
so applying the drafts cannot overwrite intervening edits. Both README drafts
cover exactly the 13 planned installer names. The isolated website draft builds
with Jekyll: 34 pages, 1,096 internal links/assets and anchors passed; all 13
planned installer names are covered and no previous current-release strings
remain in rendered HTML. These are unpublished drafts, not working download URLs.
Apply only after public asset checks, then repeat production-site acceptance.

Strict locale validation passed: 3,003 translation pairs, 2,971 marked display
sites, zero unclassified direct display literals. This does not replace native
screenshot acceptance.

The website's 320-px follow-up check additionally requires all controls to stay
inside the header content box, preserving its full 16-px padding. The final
4-px gap/2-px label-padding refinement passed the same 112 combinations.
Desktop header bounds, typography and padding were compared directly to the
baseline in both languages and are exactly equal.

Final website source `d825383615f656ba014d1f05e794adb50d9bfdd4` is public.
Its production CSS bytes match the local source; the eight live 320/390-px
checks were repeated with the stronger content-box constraint and passed.

## Scheduled cloud check: 2026-11-17 06:54 UTC

- Companion wire 44030884183 passed all Linux/macOS/Windows jobs on f2542163.
- Full acceptance 44030913883: macOS (including native capture) and both protocol
  jobs passed; Windows was still running at legacy cleanup verification. Real
  WSL/native Windows gates remain pending until its completed result is checked.
- Routine push 44030884261: both protocol jobs passed; application jobs running.
- Linux artifact 44031746164: both architectures building/verifying packages.

No immutable tag or formal draft build yet. Next status review follows the
30-minute cadence; completed macOS artifacts can be inspected independently.

## macOS native artifact acceptance

Downloaded the completed macOS capture artifact from full run 44030913883:
artifact 14306661025, 186,297,826 bytes. Its SHA-256 matches GitHub's recorded
`7a2d4354a8d9404cdf04924a105a732e0ba1574c020f9b9c8482b4999c0877bf`.
ZIP CRC and all 1,161 PNG chunk-CRC/decompression checks passed (17 fixtures,
251 fixed frames, three locales). Diagnostic source matches f2542163, exit is
zero, no failed steps/assertions/fixture tests/exceptions/connection errors/panic
locations/missing singletons/deadline, and all three disposable SSH masters lived
through capture.

Visually inspected English light 320-px follow-ups, Simplified Chinese dark
third-page follow-ups, Traditional Chinese handoff detail, English light 320-px
reconcile at 1.25 scale, and English handoff evidence. Revision arrows, full-list
navigation, reconciliation explanation/actions, task/attempt provenance and
reported-versus-reviewed evidence remain readable. Terminal drafts persist;
scrollable content and fixed usage footers retain their intended boundaries.
Windows/real-WSL acceptance remains pending, so no overall native gate is closed.

## Accepted source and formal packaging

On resumption at 2026-11-17 08:17 UTC, full run 44030913883 and routine push
44030884261 both reported success on f2542163. Windows completed real WSL
provisioning, native Windows-to-guest acceptance, bundled installation/reinstallation/
uninstallation isolation and all three-language capture stages. The explicit
bundled-WSL regression executed once and passed (140.85 s).

Windows artifact 14310366110 (153,730,409 B) matches SHA-256
`8660231e95bf186c2cc8e58c1ba8361fb99d57c5f7ec1c72c7a56cb246a98ac1`.
ZIP CRC and all 1,188 PNG chunk/decompression checks passed; diagnostic source is
f2542163, exit zero, with no failure/exception/panic/deadline. Windows captures do
not report macOS's owned-SSH-master observations; three-platform SSH wire results
are separate evidence. Inspected English third-page navigation, Simplified Chinese
WSL collaboration and Traditional Chinese handoff detail; no clipped primary
controls or overlapping text observed.

Linux artifact run 44031746164 succeeded on x64 and ARM64, including DEB/RPM/Arch/
AppImage checks and installed X11 startup. Downloaded both package archives and
smoke artifacts; archive SHA-256, ZIP CRC, exact source/release and all eight
package digests passed. Both manifests retain glibc >=2.31. Inspected both startup
PNGs. Package archives: x64 14308098226, ARM64 14308979481; smoke archives:
x64 14308223138, ARM64 14308999389. Retain these exact-source validated packages as recovery artifacts. Prefer the
existing Windows-to-Linux formal release handoff; do not dispatch a second Linux
attachment while that automatic path is active.

Created and pushed annotated immutable v1.9.0 ->
`f25421630ad3fb6dfce93de6325a9841e4afd1eb` after acceptance. No previous tags moved.
Formal macOS/Companion workflow is dispatched with artifact_only=false,
auto_publish=false. Windows follows its existing source/tag handoff. The release
stays a draft until all package/checksum/publication gates pass.

Formal draft run:
[44046076338](https://github.com/OthinusG/warpai/actions/runs/44046076338),
release_tag=v1.9.0, artifact_only=false, auto_publish=false, dispatched at
2026-11-17 08:21 UTC. Preserve its existing automatic Windows/Linux handoffs.

## Formal package acceptance

Formal macOS/Companion run 44046076338, automatic Windows 44059180202 and
automatic Linux 44070174083 all succeeded at the immutable f2542163 source.
The existing sequential handoffs completed without a duplicate manual attachment.
All three Companion native installer jobs passed.

Draft release 448421156 contained exactly 13 installers and six metadata assets.
Downloaded all 19 complete files; every size and SHA-256 matches GitHub's asset
inventory. All four checksum inventories cover the 13 installers; both Linux
manifests match the tag/source, exact architecture targets, package sets and
glibc >=2.31. A pre-publication recheck confirmed unchanged asset IDs/digests and
release notes byte-equivalent (trimmed) to docs/releases/v1.9.0.md.

Read-only macOS inspection passed both DMG integrity checks, desktop bundle
version/build 1.9.0 and dev.warpai.Warpai identity, ARM64 desktop and bridge,
strict deep ad-hoc signature, canonical app/volume icons, Applications symlink and
real Finder custom-volume-icon flags. Companion 6.1.0 source/version/ARM64,
launcher resource icon and all 15 payload checksums passed. The Linux Companion
self-extractor was parsed without execution: source/release/version/executable
and all 15 payload checksums passed. Both inspection mounts detached normally;
no app or installer was launched locally.

Windows native logs confirm checkout f25421630, application ProductVersion and
FileVersion 1.9.0, and installer ProductVersion (trimmed) 1.9.0. Windows PE version/icon/archive and
installer provenance acceptance comes from its native CI, not a local installation;
the Windows installer remains unsigned.

Verified desktop macOS DMG SHA-256:
`28cc631b10b0fdb31038541bc2568853102deed1a486645c9f617b399d53de92`.

## Public release

Published latest stable v1.9.0 at 2026-11-17T13:05:23Z:
https://github.com/OthinusG/warpai/releases/tag/v1.9.0 . All 19 canonical public
asset URLs returned HTTP 200. A separate unauthenticated canonical-URL DMG
download matches the independently verified SHA-256 above. Default latest points
to v1.9.0; the source tag remains f2542163. No existing assets/tags were replaced.

Applied the 24 deferred source/website documentation drafts only after public
URL acceptance and a before-content SHA-256 check for every target file. Current
Companion instructions describe matching 6.1.0 features with unchanged protocol
majors rather than inventing a new compatibility break. Production Pages and
Homebrew delivery are the remaining steps.

## Completed public surfaces and Homebrew

Default-source download synchronization: da89a7e5. Website source:
`f615f39ac760f09f1ea8667fd07bfdc63ea85265`; Pages reports built at that source.
The final Jekyll build passed all 34 pages and 1,096 internal links/assets/anchors;
all 13 installer names match the public inventory. All 34 production pages return
HTTP 200; pages carrying versions match 1.9.0/6.1.0, and none retain stale current
1.8.6.1 strings. The verified mobile header CSS is retained.

Homebrew PR https://github.com/OthinusG/homebrew-tap/pull/3 merged at
2026-11-17T13:17:04Z as `3599d7f73f63453d30d4625e083aaaacd34b1db9`.
The cask version and public DMG SHA-256 match the independently downloaded file.
Ruby syntax, metadata parsing, Homebrew style (only the existing
Homebrew/OSDependsOn exception for the macOS11 floor), strict online audit,
fetch/checksum and install dry-run passed. ARM64 and quarantine behavior remain.
The temporary tap and isolated trust configuration were removed; the existing
personal tap and original developer setting were preserved. No application was
installed or launched, and no personal installation/configuration was changed.

All release delivery gates above are complete. Runtime and native acceptance
refer to the immutable tagged f2542163 source; subsequent main changes are
release documentation/receipts only. Live paid vendor model calls were not part
of deterministic fixture/native acceptance.
