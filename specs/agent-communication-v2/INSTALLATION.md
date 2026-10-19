# Warpai 1.1.0 remote companion delivery

## Product contract

Supersedes the S4 manual target/root/companion form and 1.0.1 raw companion delivery.
Users connect in a terminal with ordinary system OpenSSH (`ssh user@host`, including
existing host nicknames and connection options), then select the project with `cd`.
The collaboration panel follows that confirmed remote session and working directory.
There are no required SSH alias, project-path or companion-path settings fields.

Remote installers contain the source-matched companion and install into the remote
account's stable `~/.config/warpai/bin` directory (`%USERPROFILE%\.config\warpai\bin`
on Windows). Detect the actual remote OS; derive the native executable from the remote
home directory. Never search the local filesystem for a remote companion.
Check installation and protocol compatibility before binding a project. Missing or
incompatible components show a platform-specific install/update guide and retry action.
No automatic download/execution without a user installation action.

Installation owns only Warpai components, permissions and its installation metadata.
Do not edit shell startup files, PATH, SSH settings, credentials or vendor commands.
Do not register a new persistent OS service: the companion retains its existing
on-demand private-service startup. Retain project/account isolation, fences, original
mutation receipts and unknown-work recovery.

## Implementation plan

1. Trace native SSH session/remote cwd/bootstrap and existing OpenSSH multiplexing.
   Reuse trusted connection/session metadata rather than parsing arbitrary output or
   asking users to reproduce connection details. Preserve system host-key review.
2. Build native per-OS installer packages (macOS, Linux, Windows) containing the
   correct release binary, checksums and source/version manifest. Installation must
   be idempotent, enforce Unix execute permissions and report an actionable failure.
3. Discover stable installed components from confirmed remote home/OS and perform
   a bounded compatibility probe. Fail closed on unsupported or unconfirmed sessions.
4. Replace the three-field form with terminal-driven binding, platform-specific
   install guidance and retry. Fence pending replies on cd/tab/host/account changes,
   disconnect and SSH exit; preserve local collaboration and unrelated terminal IO.
5. Update bilingual product/install docs and release pipelines to 1.1.0. Desktop
   delivery remains macOS DMG and Windows EXE only. Remote delivery is installer
   packages rather than raw companion ZIPs. Preserve old source tags and CI history.
6. Run cloud-only Rust checks, focused installer/discovery/isolation regressions,
   controlled OpenSSH tests and native macOS/Windows screenshot QA before packaging.
   Inspect actual release artifacts, checksums and manifests, then publish 1.1.0.

## Acceptance criteria

- Ordinary SSH login and remote cd bind the correct remote project without forms.
- All three remote systems install the matching component to the default location;
  paths with spaces and Unicode work, Unix modes are executable, reinstall is safe.
- No shell/SSH/credential/vendor configuration changes and no new OS service.
- Missing, incompatible, unsupported and failed SSH sessions have useful guidance;
  companion/probe output is bounded, deadlines enforced, and input shell-quoted.
- Same remote account/root shares existing authority; distinct roots/accounts/hosts
  and stale callbacks cannot adopt another project. Local collaboration still passes.
- Native UI has no manual connection form and matches existing theme/components.
- Public 1.1.0 release has desktop installers and all remote installer packages.

## Initial investigation

### Confirmed SSH master selection repair (2026-10-19)

Owner testing on macOS with a Linux zsh remote found working remote directory
completion and Companion 5.4.0/runtime probes, while Explorer and collaboration
both remained at the unconfirmed-project state. A direct version query through
the terminal's existing OpenSSH master also succeeded.

The shared selector unnecessarily required retained launch-command arguments
before considering the confirmed control socket. Select that existing master
without requiring those arguments; retain argument validation for the native
fallback and preserve pending-session, remote home/OS/root and WSL guards.
Do not reconnect to a guessed host or modify SSH/user settings.

Acceptance: a confirmed master without retained arguments selects multiplexed
transport; a session with neither master nor arguments remains unavailable;
Windows direct WSL and nested SSH selection keep their existing isolation.
Regression cases cover the first two conditions. Rust execution and native
acceptance remain pending; the owner's installed desktop has not been replaced,
and the missing launch arguments have not been observed in that running binary.

Owner authorized three-desktop cloud builds and test installers on 2026-10-19.
Use branch `codex/ssh-session-test-20261019` and artifact-only package version
`v1.7.3`; no release tag/publication or personal installation is part of this test.
macOS and Windows packaging run the shared selection regressions; Linux x64 and
ARM64 packaging run them in the native build script. Preserve the existing Windows
WSL isolation test, which correctly checks that nested SSH cannot become direct
WSL. Add a separate regression for selecting a confirmed SSH master nested inside
WSL without launch arguments, retaining its original WSL distribution.
Run existing communication/native UI acceptance and retain all package checks.

Cloud source: `ca6ed733ad90d402dc752a055f35114186a2c3f2`.
Runs: [communication/native UI/WSL](https://github.com/OthinusG/warpai/actions/runs/39195493324),
[macOS test installers](https://github.com/OthinusG/warpai/actions/runs/39195496962),
[Windows test installer](https://github.com/OthinusG/warpai/actions/runs/39195603252).
Windows hands the same SHA and artifact-only flag to Linux x64/ARM64 packaging.
All runs and owner reproduction remain pending at dispatch.
Earlier `fe91eb5a` runs were canceled after restoring the original WSL isolation
test and moving the confirmed-master assertion into a separate regression.

Linux x64/ARM64 artifact-only packaging also runs independently in
[39196005293](https://github.com/OthinusG/warpai/actions/runs/39196005293) from
the same `ca6ed733` source. Avoid treating a subsequent Windows-triggered duplicate
Linux build as a new source acceptance requirement.

Owner test application downloaded from run 39195493324 to
`~/Downloads/Warpai-SSH-test-ca6ed733/WarpaiReview.app`. Ad-hoc signature
verification passed; architecture is arm64, bundle ID `dev.warpai.WarpaiReview`.
The review workflow pins its display version to 1.6.5; this is not the package
release version. ZIP SHA-256:
`a03870c3af8e1407c8de6cf8782b6d84d2070d30e28f6c6d34320c6a38ee3297`.
No local installation or launch was performed. Await the owner's real Linux
SSH Explorer/collaboration result before claiming that reproduction is resolved.

Owner screenshots at 12:13 confirm that `ca6ed733` now selects the correct remote
host/root but fails at connection with `InvalidProfile`; the first test package
does not resolve the full reproduction. The native shell sets `SSH_SOCKET_DIR`
to literal `~/.ssh`. OpenSSH expands it for directory completion, but the managed
control validator previously required an absolute path. A local `ssh -S` check
confirmed that OpenSSH itself expands the literal home prefix.

Accept only a bounded `~/.ssh/<single ASCII filename>` form in addition to the
existing absolute paths, sharing validation between control and SFTP. Preserve
control-master-only attachment and disabled reconnect fallback. Do not expand a
WSL guest path using the Windows host home. Reject other relative paths, named
user homes, traversal, control characters and expansion tokens in this new form.
Native macOS SSH/SFTP UI acceptance must now use a literal home-relative socket,
and unit regressions must exercise both native and WSL command construction.
This client transport repair does not change the remote Companion protocol or
service behavior; Companion remains 5.4.0.

The owner confirmed both Explorer and collaboration recovered in the same
`ca6ed733` test app after setting `SSH_SOCKET_DIR="$HOME/.ssh"` in a fresh local
Mac terminal before connecting. This isolates the second failure to native
home-relative control socket validation, rather than Companion installation,
SFTP availability or the Agent communication setting.

Combined runtime source: `73f4fae3cae877276b59e64c5926ce46946b1816`.
Current runs: [communication/native UI/WSL](https://github.com/OthinusG/warpai/actions/runs/39198653585),
[macOS test package](https://github.com/OthinusG/warpai/actions/runs/39198656314),
[Windows test package](https://github.com/OthinusG/warpai/actions/runs/39198659143),
[Linux x64/ARM64 packages](https://github.com/OthinusG/warpai/actions/runs/39198662224).
The Windows run was rerun after canceled macOS completion briefly displaced it
in the existing shared release queue. The older Linux build may finish to retain
its compilation cache, but its `ca6ed733` packages are not the repaired deliverable.

Existing POSIX SSH bootstrap already creates a system OpenSSH control socket and
reports remote shell, native home/OS, session identity and cwd events. The existing
RemoteCommandExecutor reuses that socket, including WSL routing. Prefer this route
for established native sessions; do not replace user SSH aliases or configuration.
Windows remote/local PowerShell coverage requires explicit investigation rather
than assuming the POSIX shell wrapper handles it.

## Verification status

Full macOS/Windows application regressions passed at source `6ec6755` in
[run 37340966290](https://github.com/OthinusG/warpai/actions/runs/37340966290).
Final source `7ebbd3e` changes only the SSH guidance caption and documentation
link. Three-platform installer/transport checks passed in
[run 37348866696](https://github.com/OthinusG/warpai/actions/runs/37348866696).
Final native screenshot review is recorded in the UI contract and screenshot
provenance ledger. The previously rejected `770741b` PowerShell frames were
corrected by simulating a running echoed command and reserving banner height;
the capture never executes SSH.

The owner changed the release scope
while 1.0.1 Windows packaging was running; cancel that publication and preserve the
private 1.0.1 draft/source tag. Prior 1.0.1 acceptance is historical evidence only.

## Native Windows OpenSSH route

Microsoft's native client does not support ControlMaster. Preserve the original
interactive SSH destination and supported transport/authentication arguments for
an independent clean control connection when no master exists. System OpenSSH
owns authentication (including its configured key agent); Warpai never saves a
password or rewrites SSH configuration. An unavailable noninteractive credential
must produce an actionable authentication state rather than credential copying
or an alias form. Attach only after the native remote shell confirms home/OS/cwd;
fence the control connection when the originating terminal exits or changes.

Primary reference: https://github.com/PowerShell/Win32-OpenSSH/wiki/Project-Scope.
Reuse the existing transient shell integration for remote PowerShell; extend its
missing init branch without eval/Invoke-Expression or rc-file writes. Do not invoke
tmux installation or automatic shell-profile modification for this feature.


## SSH Remote settings checkpoint (2026-10-19)

Owner subsequently requested the independent SSH Remote master switch, using WSL
as the native settings reference, plus per-account native MCP installation.
Contract: [SSH-MCP-SETTINGS.md](SSH-MCP-SETTINGS.md). This expands the release to
Companion 5.5.0; the earlier Companion 5.4.0 compatibility statements describe the
connection-only repairs, not the native MCP setup feature.

Source `e36593aa` adds the master switch, live account selector, Agent checkboxes,
rescan/remove-all controls, locale coverage and six fixed screenshot states.
Source `2e4dbb04` adds verified service/account fencing and a disposable-account
ordinary native CLI MCP initialize/tools/list/bootstrap fixture on all three
remote operating systems. No paid model calls or owner installation changes.

Initial cloud checks caught missing native-input conditional compilation and a
Windows fixture's nonprivate ACL; `aced4898` fixes both without weakening runtime
permissions. A second check found another exact capability-list expectation;
`e36593aa` updates the expectation to include the new advertised capability.
These were failed intermediate checks, not acceptance receipts.

Current checks: [desktop builds/tests](https://github.com/OthinusG/warpai/actions/runs/39201409860),
[native UI/WSL](https://github.com/OthinusG/warpai/actions/runs/39201024382),
[remote native MCP/installers](https://github.com/OthinusG/warpai/actions/runs/39201364472).
The desktop/protocol run completed successfully at `2e4dbb04`. Native MCP plus
all three remote installers passed at `c554f201` in
[run 39202558424](https://github.com/OthinusG/warpai/actions/runs/39202558424).
The original WSL regression remains intact. The
1.7.2 draft stays unpublished; final 1.7.3 release still requires full checks and
owner Mac-to-Linux settings/ordinary-CLI acceptance.


Latest runtime checkpoint: `796396b38680b2977c6e14a69809a39ac92f759c`.
The `5e159524` full desktop run passed application checks/tests on both desktops,
but native capture stopped because the pre-existing synthetic remote panel
fixture had not opted into the newly default-off master. `796396b3` adds that
explicit opt-in only to the owned debug fixture. Production defaults stay off.

The Windows ordinary MCP fixture initially replaced USERPROFILE, which made
KnownFolder LocalAppData unavailable. Preserve the real disposable CI account's
Windows profile instead; do not relax runtime ACLs or change owner settings.
Native MCP initialize/tools/list/bootstrap then passed on all three systems.
Setup discovery deadlines are bounded at 180 seconds through both the desktop
control channel and the private account proxy; normal operations retain 10 seconds.

Current complete UI/WSL rerun: [39210678296](https://github.com/OthinusG/warpai/actions/runs/39210678296).
Current source-matched remote rerun: [39210766219](https://github.com/OthinusG/warpai/actions/runs/39210766219).
Artifact-only packages: [macOS](https://github.com/OthinusG/warpai/actions/runs/39211044793),
[Windows](https://github.com/OthinusG/warpai/actions/runs/39211048507),
[Linux x64/ARM64](https://github.com/OthinusG/warpai/actions/runs/39211040394).
No release publication is authorized before acceptance; the v1.7.2 draft remains private.

## Complete SSH Companion switch checkpoint (2026-10-19)

Owner confirmed that the SSH Remote master covers every Companion-backed SSH
operation, including Explorer, Review, editors, file transfers and Agent/MCP
communication. Ordinary SSH terminal connections remain independent; local and
direct WSL settings retain their own admission policies.

Runtime source `ac2d29a1` passed the complete Linux, macOS and Windows remote
matrix in [39212814355](https://github.com/OthinusG/warpai/actions/runs/39212814355).
The real SSH fixture proves that disabling revokes existing file/task clients,
blocks new clients and reconnects, and still permits native SSH commands.
Re-enabling requires a verified file reconnect rather than reviving stale access.
The native MCP fixture proves account registration is revoked before the disable
response while the ordinary Agent process remains alive.

The source-matched Linux x64 Companion 5.5.0 installer was downloaded without
installation. SHA-256:
`163dae32ce02c53a042a33e8f3a026ebb0f4da2902d9e3fabc4dc88fbe3d90cc`.
Desktop UI, WSL and build acceptance remains pending in
[39212818285](https://github.com/OthinusG/warpai/actions/runs/39212818285);
the protocol jobs have passed. Owner Mac-to-Linux ordinary CLI acceptance and
final UI review still gate delivery and publication.
