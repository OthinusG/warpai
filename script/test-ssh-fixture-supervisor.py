#!/usr/bin/env python3
"""Check the owned SSH fixture's timeout and exit propagation without a desktop build."""
import importlib.util
from pathlib import Path
import subprocess
import sys
from unittest.mock import Mock, patch

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

master = Mock()
master.poll.return_value = None
socket = Mock()
socket.exists.return_value = True
with patch.object(fixture.subprocess, 'run', side_effect=[Mock(returncode=255), Mock(returncode=0)]) as probe:
    fixture.wait_for_master(master, 'owned-config', socket)
    assert probe.call_count == 2, 'A socket alone cannot prove multiplex readiness'
master.poll.return_value = 1
try:
    fixture.wait_for_master(master, 'owned-config', socket)
    raise AssertionError('An exited master was accepted')
except RuntimeError:
    pass
master.poll.return_value = None
with patch.object(fixture.time, 'monotonic', side_effect=[0, 11]):
    try:
        fixture.wait_for_master(master, 'owned-config', socket)
        raise AssertionError('Master readiness was unbounded')
    except RuntimeError:
        pass
print('Owned SSH master readiness and failure bounds passed')
