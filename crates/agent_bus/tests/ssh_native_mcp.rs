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
            format!("set -m; printf '%s' $$ > .native-shell-pid; {} {}; read done", quote(executable.to_str().unwrap()), args.join(" "))])
    };
    #[cfg(windows)]
    let (shell, arguments) = {
        let shell = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        (shell, vec!["-NoLogo".into(), "-NoProfile".into(), "-Command".into(),
            format!("[IO.File]::WriteAllText('.native-shell-pid', [string]$PID); & '{}' {}; Read-Host", executable.to_string_lossy().replace('\'', "''"), args.join(" "))])
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
    let panel = |client: &mut HostClient| runtime.block_on(client.project_tasks(
        &TaskCommand::Panel(Default::default()), 2)).unwrap();
    let online = |value: &serde_json::Value| value["value"]["agents"].as_array().unwrap().iter()
        .any(|row| row["online"] == true && row["agent"]["program"] == "codex");
    assert_eq!(result, Ok(true), "Native MCP initialize/tools/list/bootstrap did not complete");
    assert!(online(&panel(&mut client)), "The native MCP run must be visible in its project");
    let shell_pid = std::fs::read_to_string(root.path().join(".native-shell-pid")).unwrap().trim().parse().unwrap();
    let native = |client: &mut HostClient, action| {
        let response = runtime.block_on(client.project_tasks(&TaskCommand::GuestNative(
            warp_agent_bus::wsl_setup::NativeRequest { shell_pid, action }), 3)).unwrap();
        serde_json::from_value::<warp_agent_bus::wsl_setup::NativeState>(response["value"].clone()).unwrap()
    };
    let observe = |client: &mut HostClient, run: Option<String>, activity| {
        native(client, warp_agent_bus::wsl_setup::NativeAction::Observe {
                    draft: false, blocked: false, input_epoch: 1, submitted: false,
                    cancelled: false, native_ready: None, screen_run: run, screen_activity: activity,
                })
    };
    let state = observe(&mut client, None, None);
    for activity in [warp_agent_bus::readiness::Activity::Idle,
        warp_agent_bus::readiness::Activity::Working,
        warp_agent_bus::readiness::Activity::WaitingApproval,
        warp_agent_bus::readiness::Activity::Idle] {
        std::thread::sleep(Duration::from_millis(800));
        let observed = observe(&mut client, Some(state.run.clone()), Some(activity));
        assert_eq!(observed.activity, Some(activity), "Native screen evidence must reach Companion before any conversation");
        assert_eq!(observed.run, state.run);
    }
    let value = panel(&mut client);
    let agent = value["value"]["agents"].as_array().unwrap().iter()
        .find(|row| row["online"] == true && row["agent"]["program"] == "codex").unwrap();
    let sent = runtime.block_on(client.project_tasks(&TaskCommand::Operator(
        warp_agent_bus::Operation::AgentSend { to: agent["agent"]["id"].as_str().unwrap().into(),
            body: "Owned native SSH wake fixture".into(), content: None, expects_result: false,
            subject: None, thread_id: None, reply_to: None, task_id: None,
            request_id: uuid::Uuid::new_v4().to_string() }), 4)).unwrap();
    assert!(sent.get("error").is_none());
    let guarded = native(&mut client, warp_agent_bus::wsl_setup::NativeAction::Observe {
        draft: true, blocked: true, input_epoch: 1, submitted: false, cancelled: false,
        native_ready: None, screen_run: Some(state.run.clone()), screen_activity: None });
    assert!(guarded.wake.is_none(), "Drafts and approvals must block wake delivery");
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let wake = loop {
        if let Some(wake) = observe(&mut client, Some(state.run.clone()), Some(warp_agent_bus::readiness::Activity::Idle)).wake { break wake; }
        assert!(std::time::Instant::now() < deadline, "The dormant native Agent never became deliverable");
        std::thread::sleep(Duration::from_millis(100));
    };
    assert!(native(&mut client, warp_agent_bus::wsl_setup::NativeAction::Claim { wake: wake.clone() }).valid);
    assert!(native(&mut client, warp_agent_bus::wsl_setup::NativeAction::Validate { wake: wake.clone() }).valid);
    write!(pty.writer, "Warpai peer work is waiting (message {}). Call warp_agent_inbox.\r", wake.message_id).unwrap();
    pty.writer.flush().unwrap();
    native(&mut client, warp_agent_bus::wsl_setup::NativeAction::Finish { wake, submitted: true });
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !root.path().join(".native-wake-received").exists() {
        assert!(std::time::Instant::now() < deadline, "Native PTY did not receive the guarded wake");
        std::thread::sleep(Duration::from_millis(50));
    }
    let disabled = configure(&mut client, Action::Enable(false));
    assert!(!disabled.preferences.enabled);
    assert!(!online(&panel(&mut client)), "Disabling must revoke MCP before the Agent process exits");
    assert!(pty.is_active().unwrap(), "Revocation must not stop native terminal IO");
    pty.stop().unwrap();
    let text = std::fs::read_to_string(&config).unwrap();
    assert!(text.contains("fixture-unrelated"));
    assert!(!text.contains(warp_agent_bus::setup::SERVER));
    std::fs::remove_file(&config).unwrap();
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
    assert!(line.starts_with("Warpai peer work is waiting (message "));
    std::fs::write(".native-wake-received", b"received").unwrap();
    line.clear();
    std::io::stdin().read_line(&mut line).unwrap();
}
