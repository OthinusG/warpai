//! Actual guest entrypoint must scan installed CLIs without nesting Tokio runtimes.
#![cfg(all(target_os = "linux", feature = "wsl_companion"))]

use std::{os::unix::fs::PermissionsExt, process::Stdio, time::Duration};
use tokio::io::AsyncWriteExt;
use warp_agent_bus::wsl_setup::{Action, Request, Response};

#[tokio::test]
async fn installed_agent_scan_runs_through_the_wsl_companion_entrypoint() {
    let temporary = tempfile::tempdir().unwrap();
    let home = temporary.path();
    let bin = home.join("bin");
    std::fs::create_dir(&bin).unwrap();
    let fixture = bin.join("warpai-fixture");
    std::fs::write(&fixture, "#!/bin/sh\nprintf 'fixture help\\n'\n: > \"$HOME/probed\"\n").unwrap();
    std::fs::set_permissions(&fixture, std::fs::Permissions::from_mode(0o700)).unwrap();
    let commands = vec![("custom".to_owned(), "warpai-fixture".to_owned())];
    for (mode, action, commands) in [
        ("setup", Action::Rescan, vec![]),
        ("setup", Action::Rescan, commands.clone()),
        ("recover-uninstall", Action::RemoveAll, commands),
    ] {
        let count = commands.len();
        let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_warpai-wsl-companion"))
            .arg(mode)
            .env("HOME", home)
            .env("XDG_DATA_HOME", home.join("data"))
            .env("WSL_DISTRO_NAME", "WarpaiOwnedFixture")
            .env("PATH", std::env::join_paths([bin.clone(), "/usr/bin".into(), "/bin".into()]).unwrap())
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .kill_on_drop(true).spawn().unwrap();
        let mut input = child.stdin.take().unwrap();
        input.write_all(&serde_json::to_vec(&Request { action, commands }).unwrap()).await.unwrap();
        drop(input);
        let output = tokio::time::timeout(Duration::from_secs(20), child.wait_with_output()).await.unwrap().unwrap();
        assert!(output.status.success(), "Owned fixture stderr: {}", String::from_utf8_lossy(&output.stderr));
        if mode == "setup" {
            let response: Response = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(response.available.len(), count);
        } else {
            assert!(String::from_utf8_lossy(&output.stdout).contains("warpai-install:cleanup"));
        }
    }
    assert!(home.join("probed").is_file(), "The installed executable must actually be queried");
}
