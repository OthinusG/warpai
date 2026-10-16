# Linux desktop release

Owner authorization: 2026-10-16. Linux desktop release engineering and language
restart support extend the former macOS/Windows-only desktop scope.

## Target and constraints

Match Warp's documented Linux distribution/architecture coverage: x86_64 and
ARM64, glibc >= 2.31, OpenGL ES 3.0+ or Vulkan. Produce Debian/Ubuntu `.deb`,
Fedora/RHEL-compatible and OpenSUSE/SLE-compatible `.rpm`, Arch `.pkg.tar.zst`,
and AppImage for each architecture. This is a package/compatibility target, not
evidence of complete desktop acceptance on every distribution.

Reference (checked 2026-10-16):
https://docs.warp.dev/getting-started/quickstart/installation-and-setup

Keep cloud AI, upstream update repositories, login and telemetry disabled. Reuse
the native terminal, existing Linux packaging structure, pinned Rust toolchain,
shared resource preparation and release asset conventions. No APT/YUM repository
hosting, release-tag creation or publication is requested in this iteration.

## Implementation tasks

1. Add Linux restart handling for language changes. Wait for the old PID to exit;
   preserve executable paths containing spaces/non-ASCII characters. AppImage
   restarts use the original AppImage, not an expiring mounted executable.
   Preserve cancellation and existing macOS/Windows behavior.
2. Repair the OSS Linux desktop identity, resource/Agent-bridge packaging and
   package metadata. Keep original attribution, use Warpai's license, never
   install upstream repositories or signing keys. Package all four formats.
3. Add a standalone Linux release workflow with native x86_64/ARM64 builds, a
   glibc-2.31 build baseline, caches, source provenance and SHA-256 checksums.
   Default to artifact-only validation; tagged uploads may attach new assets to
   an existing release, never overwrite assets or change publication state.
   Keep Linux independent of communication validation and reusable workflow calls.
   Successful Windows packaging hands off version, exact source and test mode via
   an artifact; `workflow_run` starts a separate Linux run. Failures/skips do not
   start builds; manual dispatch works without a Windows run.
4. Add Linux asset selection to the existing public update checker. Document
   package installation, runtime requirements and the remaining desktop feature
   acceptance scope in English and Chinese.

## Acceptance and verification

- Regression checks prove delayed Linux relaunch, safe argv handling and original
  AppImage selection; ordinary binaries keep their own executable path.
- Both architectures compile the desktop and communication bridge. Default and
  `warp_platform` checks and focused language-restart tests pass in cloud CI.
- Packages contain the matching desktop binary, `warpai-agent`, resources,
  desktop entry, icons and notices. Their architecture/version/identity agree;
  package scripts contain no upstream repository or signing-key modifications.
- ELF symbol checks enforce the glibc baseline rather than inferring it from a
  runner label. Headless X11 smoke checks exercise installed/package contents.
- Workflow/script syntax and focused package fixtures run locally. Compilation
  and native acceptance run on GitHub, without changing personal installations.
- Keep awake, Secret Service fallback, native Wayland/global shortcuts and full
  GNOME/KDE acceptance retain the prior audit's explicit outstanding work. Test
  artifacts do not establish a publishable Linux release until those gates are
  resolved or the owner explicitly chooses a reduced feature scope.
