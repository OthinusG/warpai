#!/usr/bin/env python3
"""Capture a disposable CI desktop session and fail if the installed app exits."""
from pathlib import Path
import subprocess
import sys
import time

executable = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2]).resolve()
output.mkdir(parents=True, exist_ok=True)
with (output / "desktop.log").open("w") as log:
    app = subprocess.Popen([str(executable)], stdout=log, stderr=subprocess.STDOUT)
    try:
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            if app.poll() is not None:
                raise RuntimeError(f"Linux desktop exited with {app.returncode}")
            windows = subprocess.run(["xdotool", "search", "--onlyvisible", "--pid", str(app.pid)], text=True, capture_output=True)
            if windows.returncode == 0 and windows.stdout.strip():
                window = windows.stdout.splitlines()[0]
                time.sleep(2)
                subprocess.run(["xwd", "-silent", "-id", window, "-out", str(output / "desktop.xwd")], check=True, timeout=10)
                subprocess.run(["convert", str(output / "desktop.xwd"), str(output / "desktop.png")], check=True, timeout=10)
                if app.poll() is not None:
                    raise RuntimeError("Linux desktop exited during capture")
                break
            time.sleep(0.5)
        else:
            raise RuntimeError("Linux desktop did not create a visible X11 window")
    finally:
        if app.poll() is None:
            app.terminate()
            try:
                app.wait(timeout=10)
            except subprocess.TimeoutExpired:
                app.kill()
                app.wait(timeout=10)
print("Installed Linux desktop started and produced a screenshot.")
