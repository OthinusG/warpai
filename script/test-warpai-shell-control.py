#!/usr/bin/env python3
"""Verify the bundled control function in real Bash and Zsh TTY sessions."""
import hashlib
import json
import os
from pathlib import Path
import pty
import shlex
import shutil
import subprocess
import tempfile


def run(shell, account, flags, commands, *, interactive=True):
    asset = Path("app/assets/bundled/bootstrap/warpai_control.sh").resolve()
    environment = {**os.environ, "HOME": str(account), "ZDOTDIR": str(account)}
    environment.pop("WARP_IS_SSH", None)
    environment.pop("WSL_DISTRO_NAME", None)
    environment.update(flags)
    shell_flags = ["--noprofile", "--norc"] if shell == "bash" else ["-f"]
    master, slave = pty.openpty()
    try:
        return subprocess.run(
            [shell, *shell_flags, "-ic" if interactive else "-c", commands.replace("SOURCE", f". {shlex.quote(str(asset))}")],
            stdin=slave, capture_output=True, text=True, env=environment, timeout=15,
        )
    finally:
        os.close(slave)
        os.close(master)


def main():
    shells = [shell for shell in ("bash", "zsh") if shutil.which(shell)]
    assert shells, "Bash or Zsh is required"
    with tempfile.TemporaryDirectory(prefix="warpai-shell-control-") as temporary:
        account = Path(temporary) / "account with spaces 多语言"
        binary = account / ".config/warpai/bin/warpai-companion"
        binary.parent.mkdir(parents=True)
        original = b'#!/bin/sh\nprintf "<%s>\\n" "$@"\n'
        binary.write_bytes(original)
        binary.chmod(0o700)
        binary.with_name("companion-manifest.json").write_text(json.dumps({
            "sha256": hashlib.sha256(original).hexdigest(),
        }))
        user_bin = account / "user-bin"
        user_bin.mkdir()
        user_command = user_bin / "warpai"
        user_command.write_text("#!/bin/sh\necho original\n")
        user_command.chmod(0o700)
        wslpath = Path("/usr/bin/wslpath")
        distro = None
        if wslpath.is_file():
            native = subprocess.check_output([str(wslpath), "-w", "/"], text=True).strip().rstrip("\\")
            for prefix in ("\\\\wsl.localhost\\", "\\\\wsl$\\"):
                if native.startswith(prefix):
                    distro = native[len(prefix):]
                    break
            assert distro, "A native WSL root is required"
            guest = account / ".config/warpai/wsl/bin/warpai-wsl-companion"
            guest.parent.mkdir(parents=True)
            guest.write_bytes(original)
            guest.chmod(0o700)
            shutil.copyfile(binary.with_name("companion-manifest.json"), guest.with_name("companion-manifest.json"))
        for shell in shells:
            remote = {"WARP_IS_SSH": "1"}
            result = run(shell, account, remote, "before=$PATH; SOURCE; [ \"$PATH\" = \"$before\" ] && warpai resume '含 空格' '$(echo ignored)'")
            assert result.returncode == 0, result.stderr
            assert result.stdout.splitlines() == ["<ctl>", "<resume>", "<含 空格>", "<$(echo ignored)>"], result.stdout
            result = run(shell, account, {}, "SOURCE; ! command -v warpai")
            assert result.returncode == 0, result.stderr
            result = run(shell, account, remote, "SOURCE; ! command -v warpai", interactive=False)
            assert result.returncode == 0, result.stderr
            result = run(shell, account, remote, "alias warpai='echo original'; SOURCE; eval warpai")
            assert result.returncode == 0 and result.stdout == "original\n", result.stderr
            assert f"{binary} ctl" in result.stderr
            result = run(shell, account, remote, "warpai() { echo original; }; SOURCE; warpai")
            assert result.returncode == 0 and result.stdout == "original\n", result.stderr
            result = run(shell, account, {**remote, "PATH": str(user_bin) + os.pathsep + os.environ["PATH"]}, "SOURCE; warpai")
            assert result.returncode == 0 and result.stdout == "original\n", result.stderr
            if distro:
                result = run(shell, account, {"WSL_DISTRO_NAME": distro}, "SOURCE; warpai list")
                assert result.returncode == 0 and result.stdout == "<ctl>\n<list>\n", result.stderr
                result = run(shell, account, {"WSL_DISTRO_NAME": distro + "-wrong"}, "SOURCE; ! command -v warpai")
                assert result.returncode == 0, result.stderr
            binary.write_bytes(b'#!/bin/sh\necho unexpected\n')
            result = run(shell, account, remote, "SOURCE; warpai list")
            assert result.returncode != 0 and not result.stdout, result.stderr
            assert "checksum does not match" in result.stderr
            binary.write_bytes(original)
    print("Bundled shell control, collision handling, literal argv and checksum rejection passed")


if __name__ == "__main__":
    main()
