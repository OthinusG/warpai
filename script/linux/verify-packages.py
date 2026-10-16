#!/usr/bin/env python3
"""Verify native Linux package identities, payloads and the glibc baseline."""
import hashlib
import json
import os
import re
import struct
import subprocess
import sys
import tempfile
from pathlib import Path


def run(*args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True, timeout=120)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def check_elf(path, architecture):
    with path.open("rb") as source:
        header = source.read(20)
    require(header[:6] == b"\x7fELF\x02\x01", f"Not a little-endian ELF64: {path}")
    require(struct.unpack_from("<H", header, 18)[0] == {"x64": 62, "arm64": 183}[architecture], f"Wrong ELF architecture: {path}")
    versions = [tuple(map(int, version.split("."))) for version in re.findall(r"GLIBC_([0-9.]+)", run("readelf", "--version-info", str(path)))]
    require(all(version <= (2, 31) for version in versions), f"glibc > 2.31 required: {path}")


def check_payload(root, architecture, appimage=False):
    required = [
        "opt/warpai/warpai", "opt/warpai/warpai-agent",
        "opt/warpai/resources/settings_schema.json",
        "opt/warpai/resources/THIRD_PARTY_LICENSES.txt",
        "opt/warpai/resources/FORK_NOTICE.md", "opt/warpai/resources/LICENSE-AGPL",
        "opt/warpai/resources/LICENSE-MIT",
        "usr/share/applications/dev.warpai.Warpai.desktop",
        "usr/share/icons/hicolor/512x512/apps/dev.warpai.Warpai.png",
    ]
    for relative in required:
        require((root / relative).is_file(), f"Missing package file: {relative}")
    desktop = (root / required[-2]).read_text()
    for entry in ["Exec=warpai %U", "Icon=dev.warpai.Warpai", "StartupWMClass=dev.warpai.Warpai", "MimeType=x-scheme-handler/warpai;"]:
        require(entry in desktop.splitlines(), f"Incorrect desktop identity: {entry}")
    link = root / "usr/bin/warpai"
    require(link.is_symlink(), "Missing launcher symlink")
    expected = "../../opt/warpai/warpai" if appimage else "/opt/warpai/warpai"
    require(str(link.readlink()) == expected, "Launcher does not point at the packaged binary")
    for binary in [root / "opt/warpai/warpai", root / "opt/warpai/warpai-agent"]:
        require(binary.stat().st_mode & 0o111, f"Executable permission missing: {binary}")
        check_elf(binary, architecture)
    if appimage:
        for library in (root / "usr/lib").rglob("*.so*"):
            if library.is_file() and library.read_bytes()[:4] == b"\x7fELF":
                check_elf(library, architecture)


def main():
    tag, architecture = sys.argv[1:]
    require(re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", tag), "Invalid release tag")
    require(architecture in {"x64", "arm64"}, "Unsupported architecture")
    version = tag[1:]
    output = (Path(os.environ.get("CARGO_TARGET_DIR", "target")) / "release/bundle/linux").resolve()
    prefix = f"Warpai-{version}-linux-{architecture}"
    native_arch = {"x64": "x86_64", "arm64": "aarch64"}[architecture]
    deb_arch = {"x64": "amd64", "arm64": "arm64"}[architecture]
    packages = [output / f"{prefix}.{suffix}" for suffix in ["deb", "rpm", "pkg.tar.zst", "AppImage"]]
    for package in packages:
        require(package.is_file() and package.stat().st_size > 0, f"Missing package: {package}")
        with tempfile.TemporaryDirectory(prefix="warpai-package-") as temporary:
            stage = Path(temporary)
            if package.suffix == ".deb":
                for field, expected in [("Package", "warpai"), ("Version", version), ("Architecture", deb_arch)]:
                    require(run("dpkg-deb", "-f", str(package), field).strip() == expected, f"Incorrect Debian {field}")
                run("dpkg-deb", "-x", str(package), str(stage))
                run("dpkg-deb", "-e", str(package), str(stage / "control"))
                require(not any((stage / "control" / name).exists() for name in ["preinst", "postinst", "prerm", "postrm"]), "Unexpected Debian install script")
            elif package.suffix == ".rpm":
                identity = run("rpm", "-qp", "--queryformat", "%{NAME}\n%{VERSION}\n%{ARCH}", str(package)).splitlines()
                require(identity == ["warpai", version, native_arch], "Incorrect RPM identity")
                require(not run("rpm", "-qp", "--scripts", str(package)).strip(), "Unexpected RPM install script")
                run("bsdtar", "-xf", str(package), "-C", str(stage))
            elif package.suffix == ".zst":
                info = run("bsdtar", "-xOf", str(package), ".PKGINFO").splitlines()
                for field in ["pkgname = warpai", f"pkgver = {version}-1", f"arch = {native_arch}"]:
                    require(field in info, f"Incorrect Arch identity: {field}")
                run("bsdtar", "-xf", str(package), "-C", str(stage))
            else:
                check_elf(package, architecture)
                run(str(package), "--appimage-extract", cwd=stage)
                stage /= "squashfs-root"
            check_payload(stage, architecture, package.suffix == ".AppImage")
    manifest = {
        "source": run("git", "rev-parse", "HEAD").strip(), "release": tag,
        "target": native_arch + "-unknown-linux-gnu", "glibc_minimum": "2.31",
        "packages": {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in packages},
    }
    (output / f"manifest-linux-{architecture}.json").write_text(json.dumps(manifest, indent=2) + "\n")
    (output / f"SHA256SUMS-linux-{architecture}.txt").write_text("".join(f"{digest}  {name}\n" for name, digest in manifest["packages"].items()))
    print(f"Verified all four Linux {architecture} packages, ELF payloads and glibc baseline.")


if __name__ == "__main__":
    main()
