#!/usr/bin/env python3
"""Exercise the actual SSH shell wrappers with an owned command transport."""
import json
import os
from pathlib import Path
import shutil
import shlex
import sys
import subprocess
import tempfile

repo = Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory(prefix='warpai-shell-routing-') as directory:
    root = Path(directory)
    ssh = root / 'ssh'
    transport = root / 'ssh.py'
    ssh.write_text('#!/bin/sh\nexec ' + shlex.quote(Path(sys.executable).as_posix()) + ' ' + shlex.quote(transport.as_posix()) + ' \"$@\"\n')
    transport.write_text('''import json, os, sys
with open(os.environ['OWNED_SSH_CALLS'], 'a') as calls:
    calls.write(json.dumps(sys.argv[1:]) + '\\n')
if '-G' in sys.argv:
    print('remotecommand ' + os.environ['OWNED_REMOTE_COMMAND'])
elif sys.argv[-1] == 'echo $env:OS':
    print(os.environ['OWNED_REMOTE_OS'])
''')
    ssh.chmod(0o700)
    shells = [('bash', 'bash_body.sh'), ('zsh', 'zsh_body.sh'), ('fish', 'fish.sh')]
    for shell, filename in shells:
        executable = shutil.which(shell)
        if os.name == 'nt' and shell == 'bash':
            git_bash = Path(os.environ.get('ProgramFiles', 'C:/Program Files')) / 'Git/bin/bash.exe'
            executable = str(git_bash) if git_bash.is_file() else None
        if not executable:
            source = (repo / 'app/assets/bundled/bootstrap' / filename).read_text(encoding='utf-8')
            assert 'echo $env:OS' in source and '-EncodedCommand @@WARP_SSH_POWERSHELL_INIT@@' in source
            print(f'{shell} routing source checked; runtime unavailable')
            continue
        source = (repo / 'app/assets/bundled/bootstrap' / filename).read_text(encoding='utf-8')
        start = source.index('    function warp_ssh_helper') if shell == 'fish' else source.index('function warp_ssh_helper()')
        if shell == 'fish':
            end = source.index('\n    function ssh', start)
            prelude = 'function warp_init_shell; end\nfunction warp_send_json_message; printf "%s\\n" $argv; end\n'
        else:
            end = source.index('function ssh()', start)
            # The top-level enclosing bootstrap block ends after this helper on some shells.
            helper = source[start:end].rstrip()
            helper = helper[:helper.rfind('}') + 1]
            prelude = 'init_shell_hook() { :; }\nwarp_send_json_message() { printf "%s\\n" "$1"; }\n'
        helper = source[start:end] if shell == 'fish' else helper
        helper = helper.replace('command ssh', 'command ' + chr(34) + '$OWNED_SSH_WRAPPER' + chr(34))
        script = root / f'{shell}-fixture'
        script.write_text(prelude + helper.replace('@@USING_CON_PTY_BOOLEAN@@', 'false') + '\nwarp_ssh_helper owned-host\n', encoding='utf-8')
        for remote_os, remote_command in [('Windows_NT', 'none'), (':OS', 'none'), ('Windows_NT', 'owned-command')]:
            calls = root / 'calls.jsonl'
            calls.write_text('')
            ssh_wrapper = str(ssh)
            if os.name == 'nt':
                cygpath = shutil.which('cygpath')
                if cygpath:
                    ssh_wrapper = subprocess.check_output([cygpath, '-u', str(ssh)], text=True).strip()
                else:
                    raise RuntimeError('Git Bash cygpath is required for the Windows SSH fixture')
            env = dict(os.environ, OWNED_SSH_WRAPPER=ssh_wrapper,
                OWNED_SSH_CALLS=str(calls), OWNED_REMOTE_OS=remote_os,
                OWNED_REMOTE_COMMAND=remote_command, SSH_SOCKET_DIR=str(root), WARP_SESSION_ID='123')
            result = subprocess.run([executable, str(script)], env=env, capture_output=True, text=True, timeout=10)
            assert result.returncode == 0, (shell, result.stderr)
            observed = [json.loads(line) for line in calls.read_text(encoding='utf-8').splitlines()]
            if remote_command != 'none':
                assert len(observed) == 2 and observed[-1] == ['owned-host'], (shell, observed)
            elif remote_os == 'Windows_NT':
                assert len(observed) == 3, (shell, observed)
                assert observed[-1][-1].startswith('powershell.exe -NoLogo -NoProfile -NoExit -EncodedCommand ')
                assert 'export ' not in observed[-1][-1] and 'test -n' not in observed[-1][-1]
                assert json.loads(result.stdout)['value']['remote_shell'] == 'pwsh'
            else:
                assert len(observed) == 3, (shell, observed)
                assert 'export TERM_PROGRAM=' in observed[-1][-1]
                assert r'\"shell_pid\": $$' in observed[-1][-1], (shell, 'Remote shell PID missing')
        print(f'{shell} Windows/POSIX/RemoteCommand SSH routing passed')
