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
ARM64 packaging run them in the native build script. Windows additionally checks
that a confirmed SSH master nested inside WSL selects multiplexed SSH with its
original WSL distribution, rather than merely asserting it is not direct WSL.
Run existing communication/native UI acceptance and retain all package checks.

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
