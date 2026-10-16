#!/usr/bin/env python3
"""Exercise the actual package staging script without building or installing."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

if sys.platform != "linux":
    print("Linux staging fixtures require GNU install; run in Linux CI.")
    sys.exit(0)

repo = Path(__file__).resolve().parent.parent
with tempfile.TemporaryDirectory(prefix="warpai-linux-fixture-") as temporary:
    fixture = Path(temporary) / "source ' 中文"
    output = fixture / "target/release"
    stage = fixture / "stage"
    stage.mkdir(parents=True)
    (output / "resources").mkdir(parents=True)
    for name in ["warpai", "warpai-agent"]:
        (output / name).write_text("fixture executable\n")
    (output / "resources/settings_schema.json").write_text("{}\n")
    for relative in [
        "app/channels/oss/dev.warpai.Warpai.desktop",
        "app/channels/oss/icon/no-padding/512x512.png",
        "FORK_NOTICE.md", "LICENSE-AGPL", "LICENSE-MIT",
    ]:
        destination = fixture / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(repo / relative, destination)
    env = dict(os.environ, WORKSPACE_ROOT_DIR=str(fixture), CARGO_TARGET_OUTPUT_DIR=str(output),
               EXECUTABLE_PATH=str(output / "warpai"), OPT_DIR="/opt/warpai", BINARY_NAME="warpai",
               BUNDLE_ID="dev.warpai.Warpai", RELEASE_CHANNEL="oss", ARTIFACT="app", CARGO_PROFILE="release")
    subprocess.run(["bash", str(repo / "script/linux/bundle_install"), str(stage)], env=env, check=True)
    assert (stage / "opt/warpai/warpai-agent").is_file()
    assert (stage / "opt/warpai/warpai-agent").stat().st_mode & 0o111
    assert (stage / "usr/bin/warpai").readlink() == Path("/opt/warpai/warpai")
    desktop = stage / "usr/share/applications/dev.warpai.Warpai.desktop"
    assert "Exec=warpai %U" in desktop.read_text()
    assert (stage / "opt/warpai/resources/settings_schema.json").is_file()
    assert (stage / "opt/warpai/resources/LICENSE-AGPL").is_file()
    assert not (stage / "etc").exists()
    (output / "warpai-agent").unlink()
    missing = fixture / "missing-bridge"
    missing.mkdir()
    result = subprocess.run(["bash", str(repo / "script/linux/bundle_install"), str(missing)], env=env, capture_output=True)
    assert result.returncode != 0, "Missing bridge must fail packaging"
print("Linux package staging fixtures passed.")
