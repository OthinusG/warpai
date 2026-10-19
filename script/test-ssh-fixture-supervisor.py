#!/usr/bin/env python3
"""Check the owned SSH fixture's timeout and exit propagation without a desktop build."""
import importlib.util
from pathlib import Path
import subprocess
import sys

spec = importlib.util.spec_from_file_location('ssh_fixture', Path(__file__).with_name('verify-ssh-file-tools.py'))
fixture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixture)
fixture.checked_run([sys.executable, '-c', 'pass'], timeout=5)
try:
    fixture.checked_run([sys.executable, '-c', 'raise SystemExit(7)'], timeout=5)
    raise AssertionError('Nonzero fixture exit was lost')
except subprocess.CalledProcessError as error:
    assert error.returncode == 7
try:
    fixture.checked_run([sys.executable, '-c', 'import time; time.sleep(60)'], timeout=0.1)
    raise AssertionError('Fixture timeout was lost')
except subprocess.TimeoutExpired:
    pass
print('Owned SSH fixture timeout and exit propagation passed')
