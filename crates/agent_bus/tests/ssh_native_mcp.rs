//! Disposable-account acceptance for ordinary SSH Agent MCP, without a vendor model.
use std::{io::{BufRead, Read, Write}, path::Path, process::Stdio, time::Duration};
use warp_agent_bus::{companion::TaskCommand, native_pty::{NativePty, Size}, ssh_remote::HostClient,
    ssh_setup::{Request, Response}, wsl_setup::Action};

#[test]
#[ignore = "runs only in an explicitly disposable cloud account"]
fn installed_native_mcp_initializes_from_an_ordinary_terminal() {
    assert_eq!(std::env::var("WARPAI_DISPOSABLE_MCP_ACCOUNT").as_deref(), Ok("yes"));
    if std::env::var_os("WARPAI_NATIVE_MCP_HOME").is_none() {
        let directory = tempfile::tempdir().unwrap();
        let home = directory.path().join("home");
        std::fs::create_dir(&home).unwrap();
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command.args(["--exact", "installed_native_mcp_initializes_from_an_ordinary_terminal", "--ignored", "--nocapture"])
            .env("WARPAI_NATIVE_MCP_HOME", &home);
        // Windows KnownFolder resolution needs the real disposable account's
        // profile; substituting USERPROFILE makes LocalAppData unavailable.
        #[cfg(unix)]
        command.env("HOME", &home).env("USERPROFILE", &home)
            .env("XDG_DATA_HOME", home.join("data"));
        let status = command.status().unwrap();
        assert!(status.success(), "Disposable native MCP acceptance failed");
        return;
    }
    // Windows KnownFolder APIs intentionally ignore HOME; only an empty disposable
    // account may run this fixture. Never open or overwrite an existing vendor file.
    let home = dirs::home_dir().unwrap();
    let config = home.join(".codex/config.toml");
    assert!(!config.exists(), "A pre-existing Codex configuration cannot be used by this fixture");
    std::fs::create_dir_all(config.parent().unwrap()).unwrap();
    std::fs::write(&config, "# Owned fixture\n[mcp_servers.unrelated]\ncommand = 'fixture-unrelated'\n").unwrap();
    let root = tempfile::tempdir().unwrap();
    let bin = root.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let executable = bin.join(if cfg!(windows) { "codex.exe" } else { "codex" });
    std::fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let companion = Path::new(env!("CARGO_BIN_EXE_warpai-companion"));
    let version = std::process::Command::new(companion).arg("--version").output().unwrap();
    assert!(version.status.success(), "Owned Companion binary could not execute");
    let state = dirs::data_local_dir().unwrap().join("warpai/remote");
    if let Err(error) = warp_agent_bus::companion::Companion::new(&state) {
        panic!("Owned native service identity unavailable: {:?}", error.kind());
    }
    let mut client = runtime.block_on(HostClient::connect_companion(companion, root.path().to_str().unwrap())).unwrap();
    let commands = vec![("codex".into(), "codex".into())];
    let configure = |client: &mut HostClient, action| runtime.block_on(async {
        let value = client.project_tasks(&TaskCommand::SshSetup(Request {
            action, commands: commands.clone(), search_paths: vec![bin.to_string_lossy().into_owned()],
        }), 1).await.unwrap();
        assert!(value.get("error").is_none());
        serde_json::from_value::<Response>(value["value"].clone()).unwrap()
    });
    let scan = configure(&mut client, Action::Rescan);
    assert!(scan.available.iter().any(|row| row.command == "codex" && row.installed.is_some()));
    assert!(!scan.preferences.enabled);
    configure(&mut client, Action::Enable(true));
    let enabled = configure(&mut client, Action::Select("codex".into()));
    assert!(enabled.preferences.programs().contains("codex"));
    let text = std::fs::read_to_string(&config).unwrap();
    assert!(text.contains("fixture-unrelated"));
    let value: toml::Value = toml::from_str(&text).unwrap();
    assert!(value["mcp_servers"][warp_agent_bus::setup::SERVER].get("env").is_none());
    let args = ["--exact", "native_agent_child", "--ignored", "--nocapture"];
    #[cfg(unix)]
    let (shell, arguments) = {
        let quote = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));
        (std::path::PathBuf::from("/bin/sh"), vec!["-i".into(), "-c".into(),
            format!("set -m; {} {}; read done", quote(executable.to_str().unwrap()), args.join(" "))])
    };
    #[cfg(windows)]
    let (shell, arguments) = {
        let shell = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        (shell, vec!["-NoLogo".into(), "-NoProfile".into(), "-Command".into(),
            format!("& '{}' {}; Read-Host", executable.to_string_lossy().replace('\'', "''"), args.join(" "))])
    };
    let mut pty = NativePty::spawn(&shell, &arguments, root.path(), Size {columns:160, rows:30}).unwrap();
    let mut reader = pty.reader.try_clone().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = [0;4096];
        let mut output = String::new();
        while let Ok(count) = reader.read(&mut bytes) {
            if count == 0 { break; }
            output.push_str(&String::from_utf8_lossy(&bytes[..count]));
            if output.contains("NATIVE_MCP_ROUNDTRIP_OK") { let _ = sender.send(true); return; }
            if output.len() > 65536 { break; }
        }
        let _ = sender.send(false);
    });
    let result = receiver.recv_timeout(Duration::from_secs(60));
    pty.stop().unwrap();
    let disabled = configure(&mut client, Action::Enable(false));
    assert!(!disabled.preferences.enabled);
    let text = std::fs::read_to_string(&config).unwrap();
    assert!(text.contains("fixture-unrelated"));
    assert!(!text.contains(warp_agent_bus::setup::SERVER));
    std::fs::remove_file(&config).unwrap();
    assert_eq!(result, Ok(true), "Native MCP initialize/tools/list/bootstrap did not complete");
}

#[test]
#[ignore = "fixture Agent launched only by the owned native terminal"]
fn native_agent_child() {
    assert_eq!(std::env::var("WARPAI_DISPOSABLE_MCP_ACCOUNT").as_deref(), Ok("yes"));
    assert!(std::env::var_os(warp_agent_bus::transport::CAPABILITY).is_none());
    let config: toml::Value = toml::from_str(&std::fs::read_to_string(dirs::home_dir().unwrap().join(".codex/config.toml")).unwrap()).unwrap();
    let server = &config["mcp_servers"][warp_agent_bus::setup::SERVER];
    let mut bridge = std::process::Command::new(server["command"].as_str().unwrap())
        .args(server["args"].as_array().unwrap().iter().map(|arg| arg.as_str().unwrap()))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
    let mut input = bridge.stdin.take().unwrap();
    let mut output = std::io::BufReader::new(bridge.stdout.take().unwrap());
    for request in [
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"owned-native-ssh-fixture","version":"1"}}}),
        serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"warp_project_bootstrap","arguments":{}}}),
    ] {
        writeln!(input, "{request}").unwrap();
        input.flush().unwrap();
        if let Some(id) = request.get("id") {
            let mut line = String::new();
            output.read_line(&mut line).unwrap();
            let response: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(&response["id"], id);
            assert!(response.get("error").is_none());
            assert_ne!(response["result"]["isError"], true);
            if id == 2 {
                assert!(response["result"]["tools"].as_array().unwrap().iter().any(|tool| tool["name"] == "warp_agent_send"));
            }
        }
    }
    drop(input);
    assert!(bridge.wait().unwrap().success());
    println!("NATIVE_MCP_ROUNDTRIP_OK");
    std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
}
