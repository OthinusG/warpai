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
        if self.path == "/proxy.pac":
            body = (f'function FindProxyForURL(url, host) {{ '
                    f'if (host == "127.0.0.1" || host == "localhost") return "DIRECT"; '
                    f'return "PROXY 127.0.0.1:{self.server.server_port}"; }}').encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/x-ns-proxy-autoconfig")
            self.send_header("Cache-Control", "no-store")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        self.observed.append(self.path)
        body = b"native-system-proxy"
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_CONNECT(self):
        self.observed.append(self.path)
        self.send_error(403, "Synthetic tunnel rejection")

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
            run("reg", "add", key, "/v", "AutoDetect", "/t", "REG_DWORD", "/d", "0", "/f")
        elif system == "Darwin":
            route = run("route", "-n", "get", "default").stdout.decode()
            device = next(line.split(":", 1)[1].strip() for line in route.splitlines() if "interface:" in line)
            order = run("networksetup", "-listnetworkserviceorder").stdout.decode().splitlines()
            service = next(order[i - 1].split(") ", 1)[1] for i, line in enumerate(order) if f"Device: {device})" in line)
            run("sudo", "networksetup", "-setwebproxy", service, "127.0.0.1", str(port))
            run("sudo", "networksetup", "-setsecurewebproxy", service, "127.0.0.1", str(port))
            time.sleep(0.5)
        elif system == "Linux":
            env["XDG_CURRENT_DESKTOP"] = "GNOME"
            run("gsettings", "set", "org.gnome.system.proxy.http", "host", "127.0.0.1", env=env)
            run("gsettings", "set", "org.gnome.system.proxy.http", "port", str(port), env=env)
            run("gsettings", "set", "org.gnome.system.proxy.https", "host", "127.0.0.1", env=env)
            run("gsettings", "set", "org.gnome.system.proxy.https", "port", str(port), env=env)
            run("gsettings", "set", "org.gnome.system.proxy", "mode", "manual", env=env)
        else:
            raise SystemExit("Unsupported acceptance OS")
        result = subprocess.run([probe, target], env=env, timeout=15, capture_output=True)
        assert result.returncode == 0, "Native fixture probe failed: " + result.stderr.decode(errors="replace")[-1000:]
        assert result.stdout.strip() == b"200", "Proxy fixture did not receive a successful native request"
        assert target in Proxy.observed, f"Native proxy fixture did not receive target: {Proxy.observed}"

        def child(url, expected, observed):
            Proxy.observed.clear()
            result = subprocess.run([probe, url, "--subprocess"], env=env, timeout=20, capture_output=True)
            assert result.returncode == 0, "Subprocess fixture failed: " + result.stderr.decode(errors="replace")[-1000:]
            assert result.stdout.strip() == expected
            assert observed in Proxy.observed, f"Child bypassed the selected system proxy: {Proxy.observed}"

        child(target, b"200", target)
        child("https://warpai-proxy.invalid/acceptance", b"connect-rejected", "warpai-proxy.invalid:443")

        pac_url = f"http://127.0.0.1:{port}/proxy.pac"
        if system == "Windows":
            run("reg", "add", key, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "0", "/f")
            run("reg", "add", key, "/v", "AutoConfigURL", "/t", "REG_SZ", "/d", pac_url, "/f")
        elif system == "Darwin":
            run("sudo", "networksetup", "-setwebproxystate", service, "off")
            run("sudo", "networksetup", "-setsecurewebproxystate", service, "off")
            run("sudo", "networksetup", "-setautoproxyurl", service, pac_url)
            run("sudo", "networksetup", "-setautoproxystate", service, "on")
        else:
            run("gsettings", "set", "org.gnome.system.proxy", "autoconfig-url", pac_url, env=env)
            run("gsettings", "set", "org.gnome.system.proxy", "mode", "auto", env=env)
        time.sleep(0.5)
        child(target, b"200", target)
        child("https://warpai-proxy.invalid/acceptance", b"connect-rejected", "warpai-proxy.invalid:443")
        child(f"http://127.0.0.1:{port}/acceptance", b"200", "/acceptance")
        print(f"Verified {system} native and child static/PAC routes, HTTPS CONNECT and loopback bypass.")
    finally:
        if system == "Windows":
            run("reg", "add", key, "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "0", "/f")
            run("reg", "delete", key, "/v", "ProxyServer", "/f")
            subprocess.run(["reg", "delete", key, "/v", "AutoConfigURL", "/f"], capture_output=True)
        elif system == "Darwin":
            if "service" in locals():
                run("sudo", "networksetup", "-setwebproxystate", service, "off")
                run("sudo", "networksetup", "-setsecurewebproxystate", service, "off")
                run("sudo", "networksetup", "-setautoproxystate", service, "off")
        elif system == "Linux":
            run("gsettings", "set", "org.gnome.system.proxy", "mode", "none", env=env)
        proxy.shutdown()
        proxy.server_close()


if __name__ == "__main__":
    main()
