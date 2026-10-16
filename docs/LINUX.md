# Warpai on Linux

**English** | [简体中文](LINUX.zh-CN.md)

Linux desktop release engineering targets x86_64 and ARM64, matching
[Warp's documented Linux coverage](https://docs.warp.dev/getting-started/quickstart/installation-and-setup).
The minimum compatibility target is glibc 2.31 with OpenGL ES 3.0+ or Vulkan:
Ubuntu 20.04+, Debian 11+, Fedora 32+, Arch, and compatible RPM distributions
including OpenSUSE/SLE. Older glibc and musl-based systems are outside this target.

Linux packages are currently validation artifacts, not a published supported
desktop release. Full native Wayland, desktop-environment and credential/power
acceptance remains tracked in [the Linux contract](../specs/linux-desktop/TECH.md).
Matching a package format does not establish acceptance on every distribution.

## Packages

The Linux release workflow produces all four formats for both architectures:

| Distribution family | Package | Install |
| --- | --- | --- |
| Debian / Ubuntu | `Warpai-VERSION-linux-ARCH.deb` | `sudo apt install ./Warpai-VERSION-linux-ARCH.deb` |
| Fedora / compatible RPM systems | `Warpai-VERSION-linux-ARCH.rpm` | `sudo dnf install ./Warpai-VERSION-linux-ARCH.rpm` |
| OpenSUSE / SLE | The same RPM | `sudo zypper install ./Warpai-VERSION-linux-ARCH.rpm` |
| Arch Linux | `Warpai-VERSION-linux-ARCH.pkg.tar.zst` | `sudo pacman -U ./Warpai-VERSION-linux-ARCH.pkg.tar.zst` |
| Other compatible desktops | `Warpai-VERSION-linux-ARCH.AppImage` | Mark executable, then run the AppImage |

`ARCH` is `x64` or `arm64`. Native packages install `/usr/bin/warpai`, a matching
desktop entry and icons; their binaries, Agent bridge and resources live under
`/opt/warpai`. They never add APT/YUM/pacman repositories or upstream signing keys.
Install system Git for local Worktree and Review features; vendor CLI Agents keep
their own dependencies and authentication. Minimal desktops also need working
graphics drivers, fonts, `xdg-utils`, file-dialog support and desktop D-Bus services.

Language changes wait for the old application to exit before restarting. AppImage
restarts use the original AppImage file, which must remain at its original path.
The update checker selects the matching published Linux AppImage; native package
users can download their preferred format from the same release page.

## Build and release

Dispatch **Release Warpai Linux** with a semantic `release_tag`. Artifact-only is
enabled by default and builds the selected source revision. Both native runners
use a Debian 11 build environment; package verification checks actual ELF/glibc
requirements, architecture, version, bridge/resources and desktop identity.
The workflow runs language-restart regressions, default/`warp_platform` checks,
and an installed X11 desktop smoke capture. It emits source manifests and SHA-256
checksums alongside all eight packages. Optional Linux checks in the communication
workflow call the same artifact-only build rather than duplicate its implementation.

To attach packages, first create a reviewed immutable tag and an existing draft
release, then disable artifact-only. The workflow requires both architectures and
verifies their source/checksums; existing assets and published releases are refused.
It never publishes a release. Artifact-only runs do not alter GitHub Releases.
