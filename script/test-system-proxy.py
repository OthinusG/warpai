#!/usr/bin/env python3
"""Verify OS proxy routing only on disposable GitHub-hosted acceptance runners."""
import http.server
import os
import pathlib
import platform
import subprocess
import sys
import threading
import time


def run(*args, **kwargs):
    return subprocess.run(args, check=True, capture_output=True, **kwargs)


class Proxy(http.server.BaseHTTPRequestHandler):
    observed = []

    def do_GET(self):
        self.observed.append(self.path)
        body = b"native-system-proxy"
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_):
        pass


def main():
    if os.environ.get("GITHUB_ACTIONS") != "true" or os.environ.get("RUNNER_ENVIRONMENT") != "github-hosted":
        raise SystemExit("Requires a disposable GitHub-hosted runner; never changes personal proxy settings.")
    probe = str(pathlib.Path(sys.argv[1]).resolve())
    proxy = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Proxy)
    port = proxy.server_address[1]
    thread = threading.Thread(target=proxy.serve_forever, daemon=True)
    thread.start()
    target = "http://warpai-proxy.invalid/acceptance"
    system = platform.system()
    env = dict(os.environ)
    # A shell proxy must not mask failure to consult native system settings.
    for name in list(env):
        if name.lower() in {"http_proxy", "https_proxy", "all_proxy", "no_proxy"}:
            del env[name]
    try:
        if system == "Windows":
            key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings"
            run("reg", "add", key, "/v", "ProxyServer", "/t", "REG_SZ", "/d", f"127.0.0.1:{port}", "/f")
            run("reg", "add", key, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "1", "/f")
        elif system == "Darwin":
            route = run("route", "-n", "get", "default").stdout.decode()
            device = next(line.split(":", 1)[1].strip() for line in route.splitlines() if "interface:" in line)
            order = run("networksetup", "-listnetworkserviceorder").stdout.decode().splitlines()
            service = next(order[i - 1].split(") ", 1)[1] for i, line in enumerate(order) if f"Device: {device})" in line)
            run("sudo", "networksetup", "-setwebproxy", service, "127.0.0.1", str(port))
            time.sleep(0.5)
        elif system == "Linux":
            env["XDG_CURRENT_DESKTOP"] = "GNOME"
            run("gsettings", "set", "org.gnome.system.proxy.http", "host", "127.0.0.1", env=env)
            run("gsettings", "set", "org.gnome.system.proxy.http", "port", str(port), env=env)
            run("gsettings", "set", "org.gnome.system.proxy", "mode", "manual", env=env)
        else:
            raise SystemExit("Unsupported acceptance OS")
        result = subprocess.run([probe, target], env=env, timeout=15, capture_output=True)
        assert result.returncode == 0, "Native fixture probe failed: " + result.stderr.decode(errors="replace")[-1000:]
        assert result.stdout.strip() == b"200", "Proxy fixture did not receive a successful native request"
        assert Proxy.observed == [target], "Native transport bypassed the configured OS proxy"
        print(f"Verified {system} native system proxy routing without shell proxy variables.")
    finally:
        if system == "Windows":
            run("reg", "add", key, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "0", "/f")
            run("reg", "delete", key, "/v", "ProxyServer", "/f")
        elif system == "Darwin":
            if "service" in locals():
                run("sudo", "networksetup", "-setwebproxystate", service, "off")
        elif system == "Linux":
            run("gsettings", "set", "org.gnome.system.proxy", "mode", "none", env=env)
        proxy.shutdown()
        proxy.server_close()


if __name__ == "__main__":
    main()
