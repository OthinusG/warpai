#!/usr/bin/env python3
"""Assert that diagnostic output never copies runtime payload or unknown names."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile

module_spec = importlib.util.spec_from_file_location(
    "capture_diagnostic", Path(__file__).with_name("native-capture-diagnostic.py")
)
module = importlib.util.module_from_spec(module_spec)
module_spec.loader.exec_module(module)
source = 'TestStep::new("owned checkpoint")'
log = "\n".join([
    "Test step 'owned checkpoint' failed: unrelated runtime payload",
    "Test step 'runtime payload' failed",
    "Test step 'owned checkpoint' failed",
])
assert module.failed_steps(source, log) == ["owned checkpoint"]
assert module.failed_steps(source, "unrelated runtime payload") == []
assert module.failed_assertions('add_named_assertion("owned assertion", callback)',
    "Native checkpoint failed: owned assertion\nNative checkpoint failed: runtime payload") == ["owned assertion"]
assert module.failed_assertions(source, "Native checkpoint failed: runtime payload") == []
with tempfile.TemporaryDirectory() as directory:
    root = Path(directory)
    (root / "capture").mkdir()
    (root / "capture/checkpoint-assertion.txt").write_text("owned remote tree populated")
    (root / "capture/checkpoint-tree.txt").write_text("Native SSH tree: active=true, current=false, selected=true, attached=false, error=false, roots=0, entries=0\nunrelated runtime payload")
    (root / "capture/checkpoint-panic.txt").write_text("app/src/agent_communication/panel.rs:4485:1\nunrelated runtime payload")
    (root / "capture/checkpoint-singleton.txt").write_text("Native missing singleton: warpai::terminal::cli_agent_sessions::CLIAgentSessionsModel\nNative missing singleton: settings::Preferences\nNative missing singleton: runtime payload")
    (root / "capture/checkpoint-agent-projection.txt").write_text("Native Agent projection: connected=true, remote=true, project_uuid=true, fixture=true, online=false, renamed=true, run_uuid=true\nunrelated runtime payload")
    (root / "capture/checkpoint-worktree.txt").write_text("Native Worktree: connected=false, form=true, joined=false, team=false, main=true, submitting=false, query_error=3, git_rev_parse=0, git_registry=1\nunrelated runtime payload")
    (root / "capture/checkpoint-worktree-error.txt").write_text("Native Worktree read error: category=3\nunrelated runtime payload")
    (root / "capture/checkpoint-worktree-creation.txt").write_text("Native Worktree creation: snapshot=true, child=false, coordinator=true, roles=1, candidates=1, tabs=2, modal=false\nunrelated runtime payload")
    (root / "capture/checkpoint-private-git.txt").write_text("Native private Git error: code=7\nunrelated runtime payload")
    (root / "capture/checkpoint-private-git-runner.txt").write_text("Native private Git runner: stage=2, exit=128\nunrelated runtime payload")
    (root / "capture/checkpoint-deadline.txt").write_text("Native collaboration capture exceeded its 420-second deadline\nunrelated runtime payload")
    (root / "log").write_text("Native SSH file failure: ConnectionLost\nNative SSH file failure: runtime payload\nwarning: crates/agent_bus/src/companion_tasks.rs:637:37\n")
    subprocess.run([sys.executable, str(Path(__file__).with_name("native-capture-diagnostic.py")), "--log", str(root / "log"), "--output", str(root / "diagnostic.json"), "--exit-code", "1"], check=True, stdout=subprocess.DEVNULL)
    diagnostic = json.loads((root / "diagnostic.json").read_text())
    assert diagnostic["failed_assertions"] == ["owned remote tree populated"]
    assert diagnostic["connection_errors"] == ["ConnectionLost"]
    assert diagnostic["remote_trees"] == [dict(active=True, current=False, selected=True, attached=False, error=False, roots=0, entries=0)]
    assert diagnostic["panic_locations"] == ["app/src/agent_communication/panel.rs:4485:1"]
    assert diagnostic["missing_singletons"] == ["settings::Preferences", "warpai::terminal::cli_agent_sessions::CLIAgentSessionsModel"]
    assert diagnostic["agent_projection"] == [dict(connected=True, remote=True, project_uuid=True, fixture=True, online=False, renamed=True, run_uuid=True)]
    assert diagnostic["worktree_states"] == [dict(connected=False, form=True, joined=False, team=False, main=True, submitting=False, query_error=3, git_rev_parse=0, git_registry=1)]
    assert diagnostic["worktree_read_errors"] == [3]
    assert diagnostic["worktree_creation"] == [dict(snapshot=True, child=False, coordinator=True, roles=1, candidates=1, tabs=2, modal=False)]
    assert diagnostic["private_git_errors"] == [7]
    assert diagnostic["private_git_runner"] == [dict(stage=2, exit=128)]
    assert diagnostic["deadline_exceeded"] is True
    assert "runtime payload" not in json.dumps(diagnostic)
print("Native capture diagnostic redaction passed")
