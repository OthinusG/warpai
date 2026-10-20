use super::*;
use base64::Engine as _;

struct TestAssetProvider;

#[test]
fn native_bootstrap_reports_remote_shell_pid_on_every_desktop() {
    struct Assets;
    impl AssetProvider for Assets {
        fn get(&self, path: &str) -> anyhow::Result<Cow<'_, [u8]>> {
            let script = match path {
                "bundled/bootstrap/bash_init_shell.sh" => include_str!("../../assets/bundled/bootstrap/bash_init_shell.sh"),
                "bundled/bootstrap/bash_init_subshell.sh" => include_str!("../../assets/bundled/bootstrap/bash_init_subshell.sh"),
                "bundled/bootstrap/zsh_init_shell.sh" => include_str!("../../assets/bundled/bootstrap/zsh_init_shell.sh"),
                "bundled/bootstrap/zsh_init_subshell.sh" => include_str!("../../assets/bundled/bootstrap/zsh_init_subshell.sh"),
                "bundled/bootstrap/fish_init_shell.sh" => include_str!("../../assets/bundled/bootstrap/fish_init_shell.sh"),
                "bundled/bootstrap/fish_init_subshell.sh" => include_str!("../../assets/bundled/bootstrap/fish_init_subshell.sh"),
                "bundled/bootstrap/pwsh_init_shell.ps1" => include_str!("../../assets/bundled/bootstrap/pwsh_init_shell.ps1"),
                _ => anyhow::bail!("Unknown fixture asset"),
            };
            Ok(Cow::Borrowed(script.as_bytes()))
        }
    }
    for shell in ["bash", "zsh", "fish"] {
        for kind in ["shell", "subshell"] {
            let script = load_script(&format!("bundled/bootstrap/{shell}_init_{kind}.sh"), &Assets);
            let pid = if shell == "fish" { "$fish_pid" } else { "$$" };
            assert!(script.contains(&format!(r#"\"shell_pid\": {pid}"#)));
        }
    }
    assert!(load_script("bundled/bootstrap/pwsh_init_shell.ps1", &Assets).contains("shell_pid = $PID"));
}

#[cfg(windows)]
#[test]
fn windows_bootstrap_reports_native_shell_pid() {
    struct Assets;
    impl AssetProvider for Assets {
        fn get(&self, path: &str) -> anyhow::Result<Cow<'_, [u8]>> {
            let script = match path {
                "bundled/bootstrap/bash_init_subshell.sh" => include_str!("../../assets/bundled/bootstrap/bash_init_subshell.sh"),
                "bundled/bootstrap/zsh_init_subshell.sh" => include_str!("../../assets/bundled/bootstrap/zsh_init_subshell.sh"),
                "bundled/bootstrap/fish_init_subshell.sh" => include_str!("../../assets/bundled/bootstrap/fish_init_subshell.sh"),
                _ => anyhow::bail!("Unknown fixture asset"),
            };
            Ok(Cow::Borrowed(script.as_bytes()))
        }
    }
    for shell in ["bash", "zsh", "fish"] {
        let script = load_script(&format!("bundled/bootstrap/{shell}_init_subshell.sh"), &Assets);
        let pid = if shell == "fish" { "$fish_pid" } else { "$$" };
        assert!(script.contains(&format!(r#"\"shell_pid\": {pid}"#)));
        assert!(script.contains(r#"\"wsl_name\""#), "Guest identity must precede the full bootstrap for {shell}");
        assert!(!script.contains("export WARP_SESSION_ID"));
    }
}

impl AssetProvider for TestAssetProvider {
    fn get(&self, path: &str) -> anyhow::Result<Cow<'_, [u8]>> {
        let content = match path {
            "bundled/bootstrap/bash.sh" => "#include hello_world",
            "bundled/bootstrap/fish.sh" => "# this is a comment\nthis_is_a_command",
            "bundled/bootstrap/zsh.sh" => {
                "asdf\n#include whitespace\n    prepended whitespace\n\n\n"
            }
            "bundled/bootstrap/pwsh.ps1" => {
                r#"# This is a comment
                Write-Output 'Testing some output'
                function test1 {
                    [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingInvokeExpression', '', Justification = 'We actually need it')]
                    param([string]$command)
                    Invoke-Expression $command
                }"#
            }
            "hello_world" => "hello world!",
            "whitespace" => "no whitespace\n\n\n yes whitespace!",
            _ => anyhow::bail!("path not found in assets"),
        };
        Ok(Cow::Borrowed(content.as_bytes()))
    }
}

#[test]
fn test_include_directive() {
    assert_eq!(
        decode_script(&script_for_shell(ShellType::Bash, &TestAssetProvider)),
        "hello world!\n"
    );
}

#[test]
fn test_trims_comments() {
    assert_eq!(
        decode_script(&script_for_shell(ShellType::Fish, &TestAssetProvider)),
        "this_is_a_command\n"
    );
}

#[test]
fn test_trims_whitespace() {
    assert_eq!(
        decode_script(&script_for_shell(ShellType::Zsh, &TestAssetProvider)),
        "asdf\nno whitespace\n yes whitespace!\n prepended whitespace\n"
    );
}

#[test]
fn subshell_init_scripts_enable_raw_input_for_injected_bootstrap() {
    // The full bootstrap arrives as multi-line PTY input. In canonical mode the line discipline
    // drops bytes past MAX_CANON, which corrupts long lines like the remote PowerShell seed.
    for script in [
        include_str!("../../assets/bundled/bootstrap/zsh_init_subshell.sh"),
        include_str!("../../assets/bundled/bootstrap/bash_init_subshell.sh"),
    ] {
        assert!(script.contains("command -p stty raw"));
    }
}

#[test]
fn windows_ssh_powershell_seed_is_minified() {
    let script = decode_script(&script_for_shell(ShellType::Zsh, &crate::ASSETS));
    let marker = "-EncodedCommand ";
    let start = script
        .find(marker)
        .expect("SSH helper embeds the PowerShell seed")
        + marker.len();
    let end = start
        + script[start..]
            .find('\'')
            .expect("PowerShell seed must be single-quoted");
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&script[start..end])
        .expect("PowerShell seed must be valid base64");
    let seed = String::from_utf16(
        &decoded
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>(),
    )
    .expect("PowerShell seed must be UTF-16LE");
    assert!(seed.contains("Remove-Module -Name PSReadline"));
    assert!(seed.contains("function prompt"));
    assert!(!seed.lines().any(|line| line.trim_start().starts_with('#')));
    assert!(
        decoded.len() < 3600,
        "PowerShell seed contains unused bytes"
    );
    assert!(
        script.lines().all(|line| line.len() < 5000),
        "Injected bootstrap lines must stay bounded"
    );
}

#[test]
fn zsh_bootstrap_restores_terminal_after_reading_hidden_input() {
    let script_bytes = script_for_shell(ShellType::Zsh, &crate::ASSETS);
    let script = decode_script(&script_bytes);
    let read = script
        .find("read -r -d '' WARP_BOOTSTRAP_VAR")
        .expect("Zsh bootstrap must read the complete injected script");
    let restore = script[read..]
        .find("command -p stty sane")
        .map(|offset| read + offset)
        .expect("Zsh bootstrap must restore terminal mode");
    let evaluate = script[read..]
        .find("eval \"$WARP_BOOTSTRAP_VAR\"")
        .map(|offset| read + offset)
        .expect("Zsh bootstrap must evaluate after restoring the terminal");

    assert!(read < restore && restore < evaluate);
    assert!(script.contains("command -p stty raw; unsetopt ZLE RCS GLOBAL_RCS"));
    let restore_before_user_rc = script
        .find("command -p stty sane\n setopt ZLE")
        .expect("Zsh bootstrap must restore terminal mode before user rc files");
    let user_rc = script
        .find("source \"${ZDOTDIR:-$HOME}/.zshenv\";")
        .expect("Zsh bootstrap must source the user's startup files");
    assert!(restore_before_user_rc < user_rc);
}

#[test]
fn test_trims_powershell_specifics() {
    assert_eq!(
        decode_script(&script_for_shell(ShellType::PowerShell, &TestAssetProvider)),
        " Write-Output 'Testing some output'\n function test1 {\n param([string]$command)\n Invoke-Expression $command\n }\n"
    );
}

fn decode_script(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("should not fail to decode")
}

#[test]
fn powershell_subshell_initialization_emits_native_session_hook() {
    let command = powershell_subshell_command(&crate::ASSETS);
    assert!(!command.contains("Invoke-Expression"));
    assert!(!command.contains("-Scope CurrentUser"));
    assert!(!command.contains("-Scope LocalMachine"));
    #[cfg(windows)]
    {
        let output = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command"])
            .arg(format!("{command}; prompt"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "Native PowerShell bootstrap failed"
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        let hex = stdout
            .split("]9278;d;")
            .nth(1)
            .unwrap()
            .split('\u{7}')
            .next()
            .unwrap();
        let hook: serde_json::Value = serde_json::from_slice(&hex::decode(hex).unwrap()).unwrap();
        assert_eq!(hook["hook"], "InitShell");
        assert_eq!(hook["value"]["shell"], "pwsh");
    }
}

#[test]
fn remote_powershell_bootstrap_uses_bounded_native_commands() {
    let source = "Write-Output 'Remote 多语言'\n".repeat(100);
    let commands = String::from_utf8(remote_powershell_bootstrap(source.as_bytes())).unwrap();
    assert!(commands.contains("[IO.FileMode]::CreateNew"));
    assert!(commands.contains("finally { Remove-Item -LiteralPath"));
    assert!(!commands.contains("Invoke-Expression"));
    let mut bytes = Vec::new();
    for line in commands.split('\r').filter(|line| !line.is_empty()) {
        assert!(line.len() < 800, "Remote bootstrap commands must stay short");
        if let Some((_, encoded)) = line.split_once("FromBase64String('") {
            let encoded = encoded.split_once("')").unwrap().0;
            bytes.extend(base64::engine::general_purpose::STANDARD.decode(encoded).unwrap());
        }
    }
    assert_eq!(&bytes[..3], &[0xef, 0xbb, 0xbf]);
    assert_eq!(&bytes[3..], source.as_bytes());
}

#[cfg(windows)]
#[test]
fn remote_powershell_bootstrap_executes_on_windows_powershell_5() {
    use std::{io::Write, process::{Command, Stdio}};
    let fixture = tempfile::tempdir().unwrap();
    let marker = fixture.path().join("remote-unicode.txt");
    let path = marker.to_str().unwrap().replace('\'', "''");
    let source = format!("[IO.File]::WriteAllText('{path}','Remote 多语言')");
    let mut commands = remote_powershell_bootstrap(source.as_bytes());
    commands.extend_from_slice(b" if (Test-Path variable:_warpBootstrapPath) { exit 3 }\r");
    let mut child = Command::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", "-"])
        .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped())
        .spawn().unwrap();
    child.stdin.take().unwrap().write_all(&commands.iter().map(|byte| if *byte == b'\r' { b'\n' } else { *byte }).collect::<Vec<_>>()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "Owned PowerShell remote bootstrap failed");
    assert!(output.stderr.is_empty(), "Owned PowerShell remote bootstrap emitted errors");
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "Remote 多语言");
}

#[cfg(windows)]
#[test]
fn remote_powershell_bootstrap_runs_through_native_terminal() {
    use std::{io::{Read, Write}, sync::mpsc, time::{Duration, Instant}};
    use warp_agent_bus::native_pty::{NativePty, Size};
    let fixture = tempfile::tempdir().unwrap();
    let marker = fixture.path().join("native-bootstrap.txt");
    let seed = include_str!("../../assets/bundled/bootstrap/pwsh_init_shell.ps1")
        .trim_start_matches(BYTE_ORDER_MARK);
    let encoded = base64::engine::general_purpose::STANDARD.encode(
        seed.encode_utf16().flat_map(u16::to_le_bytes).collect::<Vec<_>>());
    let executable = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap())
        .join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut pty = NativePty::spawn(&executable,
        &["-NoLogo".into(), "-NoProfile".into(), "-NoExit".into(), "-EncodedCommand".into(), encoded],
        fixture.path(), Size { columns: 100, rows: 24 }).unwrap();
    let mut reader = pty.reader.try_clone().unwrap();
    let (ready, receive) = mpsc::sync_channel(1);
    let thread = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut buffer = [0; 4096];
        let mut notified = false;
        while let Ok(count) = reader.read(&mut buffer) {
            if count == 0 { break; }
            if !notified {
                bytes.extend_from_slice(&buffer[..count]);
                if bytes.windows(b"]9278;d;".len()).any(|part| part == b"]9278;d;") {
                    let _ = ready.send(());
                    notified = true;
                    bytes.clear();
                }
                if bytes.len() > 65536 { break; }
            }
        }
    });
    receive.recv_timeout(Duration::from_secs(15)).expect("PowerShell InitShell was not emitted");
    let path = marker.to_str().unwrap().replace('\'', "''");
    let source = format!("{}\n[IO.File]::WriteAllText('{path}','Remote 多语言')",
        "# Owned long bootstrap fixture\n".repeat(2000));
    pty.writer.write_all(&remote_powershell_bootstrap(source.as_bytes())).unwrap();
    pty.writer.flush().unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    while !marker.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
    }
    let result = std::fs::read_to_string(marker);
    let _ = pty.stop();
    let _ = pty.exit_code();
    drop(pty);
    thread.join().unwrap();
    assert_eq!(result.unwrap(), "Remote 多语言");
}
