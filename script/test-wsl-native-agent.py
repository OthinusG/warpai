#!/usr/bin/env python3
"""Owned CI fixture: ordinary Linux Agent starts MCP from guest configuration."""
import hashlib
import json
import os
from pathlib import Path
import pty
import pwd
import shlex
import subprocess
import sys
import threading
import time
import uuid


def main():
    mode, companion, directory = sys.argv[1:]
    assert pwd.getpwuid(os.getuid()).pw_name == "warpai-test"
    root = Path(directory).resolve()
    assert root == Path("/home/warpai-test/project spaces 多语言")
    os.chdir(root)
    assert all(name not in os.environ for name in (
        "WARP_AGENT_CAPABILITY", "WARP_AGENT_ENDPOINT", "WARP_TERMINAL_SESSION_UUID"))
    if mode == "--terminal":
        namespace = hashlib.sha256(os.environ["WSL_DISTRO_NAME"].encode()).hexdigest()
        state = Path(os.environ.get("XDG_DATA_HOME", str(Path.home() / ".local/share"))) / "warpai/wsl-remote" / namespace
        state.mkdir(parents=True, exist_ok=True)
        (state / "mcp-settings.json").write_text(json.dumps({"enabled": True, "selected": {
            "fixture": {"active": True, "program": "fixture", "executable": sys.executable,
                        "adapter": {"Codex": str(root / "native-mcp.json")}, "bridge": companion}
        }}))
        (root / "native-mcp.json").write_text(json.dumps({"command": companion, "args": ["mcp", "fixture"]}))
        (root / "native-session.json").unlink(missing_ok=True)
        child, terminal = pty.fork()
        if child == 0:
            command = shlex.join([sys.executable, str(Path(__file__).resolve()), "--agent", companion, str(root)])
            os.execl("/bin/bash", "bash", "--noprofile", "--norc", "-ic", command)
        def output():
            try:
                while data := os.read(terminal, 65536):
                    os.write(1, data)
            except OSError:
                pass
        threading.Thread(target=output, daemon=True).start()
        try:
            while data := os.read(0, 65536):
                os.write(terminal, data)
        finally:
            # Closing a master with a blocked reader does not deliver EOF to the Agent.
            os.write(terminal, b"\x04")
            os.waitpid(child, 0)
            os.close(terminal)
        return
    assert mode == "--agent" and os.isatty(0)
    config = json.loads((root / "native-mcp.json").read_text())
    bridge = subprocess.Popen([config["command"], *config["args"]], stdin=subprocess.PIPE,
                              stdout=subprocess.PIPE, stderr=None, text=True)
    sequence = 0
    def rpc(method, params):
        nonlocal sequence
        sequence += 1
        bridge.stdin.write(json.dumps({"jsonrpc": "2.0", "id": sequence, "method": method, "params": params}) + "\n")
        bridge.stdin.flush()
        while True:
            line = bridge.stdout.readline(1024 * 1024)
            assert line, "Native guest MCP disconnected"
            value = json.loads(line)
            if value.get("id") == sequence:
                assert "error" not in value, value.get("error", {}).get("data", {}).get("code", "MCP request failed")
                return value["result"]
    def tool(name, arguments):
        result = rpc("tools/call", {"name": name, "arguments": arguments})
        assert not result.get("isError", False)
        return result.get("structuredContent") or json.loads(result["content"][0]["text"])
    rpc("initialize", {"protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "owned-native-wsl", "version": "1"}})
    bridge.stdin.write('{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
    bridge.stdin.flush()
    names = {item["name"] for item in rpc("tools/list", {})["tools"]}
    assert {"warp_agent_send", "warp_task_assign", "warp_context_publish", "warp_context_get", "warp_context_list",
            "warp_project_bootstrap", "warp_project_memory_update", "warp_execution_thread_get"} <= names
    assert tool("warp_agent_ready", {})["ready"]
    def publish(wakes):
        temporary = root / "native-session.tmp"
        temporary.write_text(json.dumps({"shell_pid": os.getppid(), "agent_pid": os.getpid(), "wakes": wakes}))
        temporary.replace(root / "native-session.json")
    publish(0)
    wakes = 0
    try:
        for line in sys.stdin:
            if line.strip() == "OWNED_READY":
                assert tool("warp_agent_ready", {})["ready"]
                continue
            assert line.startswith("Warpai peer work is waiting (message ")
            uuid.UUID(line.split("(message ", 1)[1].split(")", 1)[0])
            inbox = tool("warp_agent_inbox", {})
            for message in inbox["messages"]:
                if message["kind"] == "request":
                    bootstrap = tool("warp_project_bootstrap", {})
                    memory = next(entry for entry in bootstrap["memory"] if entry["key"] == "wsl-contract")
                    assert memory["current"] and memory["proposed"] is None
                    body = message["body"]
                    if isinstance(body, dict):
                        assert body["schema_version"] == 1
                        instruction = body["instruction"] if body["type"] == "instruction" else body
                        for reference in instruction.get("context_refs", []):
                            assert memory["current"]["resource"]["reference"] == reference
                            snapshot = tool("warp_context_get", {"reference": reference})
                            assert snapshot["reference"] == reference
                            assert snapshot["content"]["summary"] == "Reusable WSL findings"
                    arguments = {"reply_to": message["id"], "question": "Confirm guest fixture acceptance",
                                 "request_id": str(uuid.uuid4())}
                    question = tool("warp_agent_ask", arguments)
                    assert tool("warp_agent_ask", arguments)["id"] == question["id"]
                    deadline = time.monotonic() + 20
                    while True:
                        answers = tool("warp_agent_inbox", {})["messages"]
                        answer = next((item for item in answers if item["kind"] == "answer"
                                       and item["reply_to"] == question["id"]), None)
                        if answer:
                            assert answer["to"] == message["to"]
                            tool("warp_agent_ack", {"message_id": answer["id"]})
                            break
                        assert time.monotonic() < deadline, "Guest decision answer was not delivered"
                        time.sleep(0.1)
                    returned = tool("warp_agent_reply", {
                        "reply_to": message["id"], "body": "Guest processing finished; native fixture passed",
                        "outcome": "succeeded", "request_id": str(uuid.uuid4()),
                    })
                    assert returned["reply_to"] == message["id"]
                    assert returned["to"] == message["from"]
                    assert returned["kind"] == "result_succeeded"
                else:
                    tool("warp_agent_ack", {"message_id": message["id"]})
            wakes += 1
            assert tool("warp_agent_ready", {})["ready"]
            publish(wakes)
    finally:
        bridge.stdin.close()
        bridge.wait(timeout=10)


if __name__ == "__main__":
    main()
