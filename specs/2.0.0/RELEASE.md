# Warpai 2.0.0 optional guardian release plan

Status: proposed. Published versions remain Warpai 1.9.0 / Companion 6.1.0. This
planning task creates no runtime change, version bump, tag, build or publication.
Use the existing [release contract](../RELEASE.md) when release work begins.

## Version, compatibility and supported scope

Desktop target 2.0.0; proposed Companion 6.2.0 if additive negotiation preserves
current clients. Freeze version/protocol decisions against actual changes at S0/S9.
Do not bump protocol major just to match desktop version or require new helpers
for working unguarded features. SSH and dedicated WSL guest must each advertise
proven guardian capability; WSL is required, not an optional later milestone.

No native local guardian/helper distribution, local Broker migration or forced
workflow switch. Existing Windows-bundled WSL guest remains source-matched. New
SSH hosts use normal verified Companion installation. Guarding defaults off;
preferences apply to future launches and require an explicit new session policy.

## Bilingual product content framework

| Surface | Content integrated in the existing location |
| --- | --- |
| EN/ZH README workflow | Optional SSH/WSL guardian alongside current workflow; no local persistence claim |
| README remote sections | Opt in, leave, reconnect same session; multiple Agents/commands and host boundaries |
| Source Agent guides | Headless task/readiness/permission behavior, serial per-Agent queue and review safeguards |
| Source SSH/WSL guides | Separate target identity, CLI attach, defaults, capability and distro/host shutdown behavior |
| Website overview/features | Optional low-overhead remote continuity with measured claims only |
| Website installation | New remote/guest capability requirement, legacy compatibility, no local daemon requirement |
| Website sessions/workflow | Actual per-session policy, multi-session catalog, writer takeover and explicit stop-many |
| Website recovery/files | Lazy Explorer/editor/Review restore, output expiry and uncertain file/task reconciliation |
| Release notes | Migration/version details, benchmark conditions and accepted limitations |

Keep evergreen docs descriptive rather than changelogs. Never claim every UI
feature runs without a client, all existing processes are adopted, or work survives
host/distro shutdown. Use the PRD feature matrix. Update EN/ZH together and preserve
screenshot provenance, the accepted mobile header and gh-pages `/warpai` base URL.

## Private CLI and reconnect delivery

CLI commands are part of existing Companion/guest executables, not a new PATH
installation. Test session-only `warpai` function injection/collision handling and
absolute-path invocation from independent SSH/WSL shells. Document list/resume/stop,
unique labels/IDs, current/unique/MRU resume and immediate bare stop-all from
CLI.md. Document only list/resume/stop and standard help/version, with no resume
picker or required stop confirmation. Existing UI launch and automatic guardian
lifecycle cover the rest; do not promise withdrawn tools.
One guardian serves all Agent kinds in the verified execution account; logical
SSH/WSL/Store scopes remain isolated. Automatic reconnect restores information
and only safely reacquires a free previous writer; no control theft or relaunch.

## Packaging/update acceptance

- [ ] Source-matched desktop clients, SSH Companion and dedicated WSL guest on each
  advertised OS/architecture; correct capability and namespace in actual binaries.
- [ ] Existing local and guardian-off installations/workflows need no new daemon.
- [ ] Guest payload checksum/provenance and disposable install/reinstall/corruption
  tests pass with spaces/Unicode and exact distro/user boundaries.
- [ ] Remote Unix/Windows and WSL native services survive client/bridge exit.
- [ ] Compatible live runtimes are reused. Side-by-side helper/dependency versions
  retain files needed by active runs; updates never kill work for version mismatch.
- [ ] Incompatible live runtime upgrade/uninstall reports exact active sessions
  and requires explicit safe stop; no wildcard process kill or private-config loss.
- [ ] Downgrade does not overwrite newer catalog/task state or launch a second
  Broker on the same Store. No automatic reboot-resume plans are stored.

## Release gates

- [ ] S0-S8 required tasks verified with detailed method/source/evidence records.
- [ ] V01-V30 required native coverage passed, including mandatory real WSL.
- [ ] Default-off, mixed-policy and native local regressions passed.
- [ ] Unified account daemon, CLI selection/stop semantics, PATH/profile isolation
  and automatic information restore passed V25-V30.
- [ ] Multi-session/project fairness, serial per-Agent semantics and queue caps passed.
- [ ] Idle/flood/reconnect measurements freeze CPU/memory and concurrency budgets;
  no advertised overhead number lacks native benchmark conditions.
- [ ] EN/ZH static/live native screenshots and focus/draft/permission QA passed.
- [ ] Default/warp_platform checks and task/local/SSH/WSL/file regressions passed.
- [ ] Packaging/update/downgrade/provenance checks passed against exact source.
- [ ] Public docs describe accepted optional scope and actual feature coverage.
- [ ] Immutable source selected; drafts built with publication disabled; downloads,
  hashes, signatures and installed helper/app versions independently checked.
- [ ] Existing publication review gate satisfied, then docs/downloads/Homebrew
  synchronized and public links/assets/mobile layout verified.
- [ ] Final ledger/MEMORY records release source, measured limits and receipts.

## Rollback and operational boundaries

Preserve published 1.9.0 artifacts and immutable release provenance. Correct a bad
release with a new version. Never fix client downgrade by stopping live guardians,
rewriting an active Store to an older schema or relaunching a vendor conversation.
Guarding off keeps the ordinary workflow available for explicit future launches;
it does not terminate existing guarded sessions. Stop or reconnect only exact
verified sessions. Host/runtime death remains interruption, not transparent resume.
