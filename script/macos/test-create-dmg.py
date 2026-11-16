"""Check DMG detach recovery without mounting disks or building Rust."""

import os
from pathlib import Path
import subprocess
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("create-dmg.sh")


class DetachTests(unittest.TestCase):
    def test_detach_recovery(self):
        for failures in (0, 2, 5, 10):
            with self.subTest(failures=failures), tempfile.TemporaryDirectory() as root:
                root = Path(root)
                source = root / "source"
                source.mkdir()
                (source / ".VolumeIcon.icns").touch()
                commands = {
                    "mktemp": 'mkdir "$CASE_ROOT/image"; echo "$CASE_ROOT/image"',
                    "sleep": 'echo sleep >> "$CASE_ROOT/calls"',
                    "xcrun": '[ "$1" != GetFileInfo ] || echo 1',
                    "hdiutil": '''
echo "$1" >> "$CASE_ROOT/calls"
case "$1" in
    detach)
        count=0
        [ ! -f "$CASE_ROOT/count" ] || count=$(cat "$CASE_ROOT/count")
        count=$((count + 1))
        echo "$count" > "$CASE_ROOT/count"
        [ "$count" -gt "$FAILURES" ] ;;
    convert) touch "$CASE_ROOT/output.dmg" ;;
esac
''',
                }
                for name, body in commands.items():
                    command = root / name
                    command.write_text("#!/bin/sh\n" + body + "\n")
                    command.chmod(0o755)
                result = subprocess.run(
                    ["sh", str(SCRIPT), str(source), str(root / "output.dmg"), "Test"],
                    env={**os.environ, "PATH": f"{root}:{os.environ['PATH']}",
                         "CASE_ROOT": str(root), "FAILURES": str(failures)},
                    capture_output=True, text=True,
                )
                calls = (root / "calls").read_text().splitlines()
                self.assertEqual(result.returncode == 0, failures < 5, result.stderr)
                self.assertEqual("convert" in calls, failures < 5)
                self.assertEqual((root / "output.dmg").exists(), failures < 5)
                self.assertEqual(calls.count("detach"), min(failures + 1, 10))
                self.assertEqual((root / "image").exists(), failures == 10)
                if failures == 10:
                    self.assertEqual(calls.count("sleep"), 8)


if __name__ == "__main__":
    unittest.main()
