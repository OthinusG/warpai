#!/usr/bin/env python3
"""Exercise fixed WSL diagnostic payloads without a personal guest or configuration."""
import json
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent


class DiagnosticTests(unittest.TestCase):
    def test_setup_payloads_and_postinstall_command(self):
        source = (ROOT / 'diagnose-wsl-companion.ps1').read_text()
        request_literals = re.findall(r"\$(?:emptyRequest|agentRequest) = '(.*)'", source)
        self.assertEqual(len(request_literals), 2)
        requests = [json.loads(text) for text in request_literals]
        self.assertEqual(requests[0]['commands'], [])
        self.assertEqual(len(requests[1]['commands']), 4)
        for request in requests:
            self.assertEqual(request['action'], {'action': 'rescan'})
        script = re.search(r"\$appScript = '(.*)'", source)[1]
        subprocess.run(['sh', '-n'], input=script, text=True, check=True)
        with tempfile.TemporaryDirectory() as directory:
            companion = Path(directory) / 'fixture companion'
            companion.write_text('#!/bin/sh\ncat\n')
            companion.chmod(0o700)
            user = subprocess.check_output(['id', '-un'], text=True).strip()
            result = subprocess.run(['sh', '-c', script, 'warpai-wsl-setup', str(companion), user],
                                    input=request_literals[1], text=True, capture_output=True, timeout=5)
            self.assertEqual(result.returncode, 0)
            self.assertEqual(json.loads(result.stdout), requests[1])
            missing = subprocess.run(['sh', '-c', script, 'warpai-wsl-setup', str(companion) + '-missing', user],
                                     input=request_literals[0], text=True, capture_output=True, timeout=5)
            self.assertEqual(missing.returncode, 127)
        self.assertIn("if ($Value.Length -gt 0 -and $Value -notmatch", source)
        self.assertIn("@('/usr/bin/id', '-un')", source)
        self.assertIn("@('/usr/bin/printenv', 'HOME')", source)
        self.assertIn("$process.StandardInput.BaseStream.Write", source)
        self.assertIn("Write-Report $stderr", source)
        self.assertIn("Setup response was not valid JSON; raw metadata omitted", source)
        self.assertNotIn('base64', source)
        self.assertNotIn('StandardInputEncoding', source)
        self.assertNotIn('Unrecognized stderr omitted', source)
        self.assertIn('"%~dp0diagnose-wsl-companion.ps1"', (ROOT / 'diagnose-wsl-companion.bat').read_text())


if __name__ == '__main__':
    unittest.main()
