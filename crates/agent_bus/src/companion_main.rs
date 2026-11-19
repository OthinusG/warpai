//! Clean stdio endpoint invoked explicitly through verified system SSH.
#[tokio::main]
async fn main() {
    run().await;
}

pub(crate) async fn run() {
    #[cfg(all(target_os = "linux", feature = "wsl_companion"))]
    if std::env::args_os().len() == 2 && std::env::args().nth(1).as_deref() == Some("--guardian-abi") {
        println!("WAG1 ssh+wsl");
        return;
    }
    if std::env::args_os().len() == 2
        && std::env::args().nth(1).as_deref() == Some("--check-runtime")
    {
        let result = warp_agent_bus::installation::git_executable()
            .and_then(|git| {
                let mut command = std::process::Command::new(git);
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    command.creation_flags(windows::Win32::System::Threading::CREATE_NO_WINDOW.0);
                }
                command.arg("--version").output()
            });
        match result {
            Ok(output) if output.status.success() => println!("Companion Git runtime ready"),
            _ => {
                eprintln!("Companion Git runtime unavailable");
                std::process::exit(1);
            }
        }
        return;
    }
    match std::env::args().nth(1).as_deref() {
        Some("mcp") if std::env::args_os().len() == 3 => {
            let program = std::env::args().nth(2).unwrap();
            if warp_agent_bus::companion::serve_guest_mcp(program).await.is_err() {
                eprintln!("Native SSH Agent MCP binding unavailable");
                std::process::exit(1);
            }
            return;
        }
        Some("forward") => {
            let result = match std::env::args().nth(2) {
                Some(endpoint) if std::env::args_os().len() == 3 => {
                    warp_agent_bus::session::forward(&endpoint).await
                }
                _ => Err(anyhow::anyhow!("Invalid relay arguments")),
            };
            if result.is_err() {
                eprintln!("Companion MCP relay unavailable");
                std::process::exit(1);
            }
            return;
        }
        Some("--bound-agent") => {
            let result = async {
                let mut arguments = std::env::args_os().skip(2);
                let program = arguments
                    .next()
                    .and_then(|s| s.into_string().ok())
                    .ok_or_else(|| anyhow::anyhow!("Missing Agent program"))?;
                let executable = std::path::PathBuf::from(
                    arguments
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("Missing native executable"))?,
                );
                anyhow::ensure!(
                    executable.is_absolute(),
                    "Native executable must be absolute"
                );
                warp_agent_bus::mcp::Bridge::from_env()?;
                let options = if program == "codex" {
                    let mut command = tokio::process::Command::new(&executable);
                    #[cfg(windows)]
                    command.creation_flags(windows::Win32::System::Threading::CREATE_NO_WINDOW.0);
                    let help = tokio::time::timeout(
                        std::time::Duration::from_secs(8),
                        command
                            .arg("--help")
                            .kill_on_drop(true)
                            .output(),
                    )
                    .await??;
                    anyhow::ensure!(help.status.success(), "Native CLI capability probe failed");
                    warp_agent_bus::launch::LaunchOptions::from_help(
                        &program,
                        &String::from_utf8_lossy(&help.stdout),
                    )
                } else {
                    Default::default()
                };
                warp_agent_bus::session::launch(
                    &warp_agent_bus::session::NativeLaunch {
                        program,
                        executable,
                        server_name: warp_agent_bus::setup::SSH_SERVER.into(),
                        options,
                    },
                    arguments.collect(),
                )
                .await
            }
            .await;
            match result {
                Ok(status) => std::process::exit(status),
                Err(_) => {
                    eprintln!("Bound Agent launch unavailable");
                    std::process::exit(1);
                }
            }
        }
        Some("agent") => match launch_agent().await {
            Ok(status) => std::process::exit(status),
            Err(_) => {
                eprintln!("SSH Agent connection unavailable");
                std::process::exit(1);
            }
        },
        Some("ctl") => {
            let code =
                warp_agent_bus::companion_ctl::run(&std::env::args().skip(2).collect::<Vec<_>>())
                    .await;
            std::process::exit(code);
        }
        _ => (),
    }
    if std::env::args_os().len() == 2
        && std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("mcp"))
    {
        use rmcp::ServiceExt;
        let result = async {
            warp_agent_bus::mcp::Bridge::from_env()?
                .serve(warp_agent_bus::mcp::legacy_transport(rmcp::transport::stdio()).await?)
                .await?
                .waiting()
                .await?;
            Ok::<_, anyhow::Error>(())
        }
        .await;
        if result.is_err() {
            eprintln!("Companion MCP binding unavailable");
            std::process::exit(1);
        }
        return;
    }
    if std::env::args_os().len() == 2
        && std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--version"))
    {
        println!(
            "warpai-companion {} protocol {}",
            warp_agent_bus::installation::RELEASE_VERSION,
            remote_protocol::managed::PROTOCOL_MAJOR
        );
        return;
    }
    if std::env::args_os().len() == 2
        && std::env::args_os().nth(1).as_deref()
            == Some(std::ffi::OsStr::new("--guardian-service"))
    {
        if warp_agent_bus::companion::serve_guardian().await.is_err() {
            eprintln!("Guardian service unavailable");
            std::process::exit(1);
        }
        return;
    }
    if std::env::args_os().len() == 3
        && std::env::args_os().nth(1).as_deref()
            == Some(std::ffi::OsStr::new("--guardian-stdio"))
    {
        let route = std::env::args().nth(2).unwrap_or_default();
        if let Err(stage) = warp_agent_bus::companion::serve_guardian_stdio(&route).await {
            eprintln!("Guardian bridge unavailable ({stage})");
            std::process::exit(1);
        }
        return;
    }
    let service = std::env::args_os().len() == 2
        && std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--service"));
    if std::env::args_os().count() != 1 && !service {
        eprintln!("Companion accepts managed control on stdio only");
        std::process::exit(2);
    }
    let result = if service {
        warp_agent_bus::companion::serve_account_service().await
    } else {
        warp_agent_bus::companion::serve_stdio().await
    };
    if result.is_err() {
        // Peer bytes, native paths and OS diagnostics never enter protocol stderr.
        eprintln!("Companion control channel closed");
        std::process::exit(1);
    }
}

/// Explicit launch inside an existing remote terminal; no shell command evaluation.
async fn launch_agent() -> anyhow::Result<i32> {
    use remote_protocol::proto::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut arguments = std::env::args().skip(2);
    let root = arguments
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing project root"))?;
    let program = arguments
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing Agent program"))?;
    let executable = arguments
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing executable"))?;
    let arguments = arguments.collect::<Vec<_>>();
    let _terminal_mode = TerminalMode::raw()?;
    let mut dimensions = terminal_size();
    let mut client = warp_agent_bus::ssh_remote::HostClient::connect_local(&root)
        .await
        .map_err(|_| anyhow::anyhow!("Private project service unavailable"))?;
    let mut state = client
        .terminal_launch(TerminalLaunch {
            fence: client.fence().cloned(),
            session_id: uuid::Uuid::new_v4().to_string(),
            executable,
            arguments,
            columns: dimensions.0,
            rows: dimensions.1,
            agent_program: Some(program), working_directory: None,
            guarded: false,
            label: None,
            session_kind: None,
        })
        .await
        .map_err(|_| anyhow::anyhow!("Agent launch unavailable"))?;
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut buffer = [0; 4096];
    let mut offset = 0;
    let mut sequence = 0;
    let mut input_open = true;
    loop {
        let mut control = TerminalControl {
            fence: client.fence().cloned(),
            session_id: state.session_id.clone(),
            run_id: state.run_id.clone(),
            action: TerminalAction::TerminalRead as i32,
            output_offset: offset,
            ..Default::default()
        };
        state = client
            .terminal_control(control.clone())
            .await
            .map_err(|_| anyhow::anyhow!("Agent control unavailable"))?;
        anyhow::ensure!(!state.output_truncated, "Agent output exceeded its buffer");
        stdout.write_all(&state.output).await?;
        stdout.flush().await?;
        offset = state.output_offset + state.output.len() as u64;
        if state.processes_active == Some(false)
            && state.output_closed
            && offset == state.output_end
        {
            control.action = TerminalAction::TerminalRelease as i32;
            control.output_offset = 0;
            client
                .terminal_control(control)
                .await
                .map_err(|_| anyhow::anyhow!("Agent release unavailable"))?;
            return Ok(state.exit_code.unwrap_or(1));
        }
        if offset < state.output_end {
            continue;
        }
        let resized = terminal_size();
        if resized != dimensions {
            let mut resize = control.clone();
            resize.action = TerminalAction::TerminalResize as i32;
            resize.output_offset = 0;
            resize.columns = resized.0;
            resize.rows = resized.1;
            client
                .terminal_control(resize)
                .await
                .map_err(|_| anyhow::anyhow!("Agent resize unavailable"))?;
            dimensions = resized;
        }
        tokio::select! {
            count = stdin.read(&mut buffer), if input_open => {
                let count = count?;
                control.output_offset = 0;
                if count == 0 {
                    input_open = false;
                    control.action = TerminalAction::TerminalStop as i32;
                } else {
                    sequence += 1;
                    control.action = TerminalAction::TerminalInput as i32;
                    control.input = buffer[..count].to_vec();
                    control.input_sequence = sequence;
                }
                client.terminal_control(control).await.map_err(|_| anyhow::anyhow!("Agent input unavailable"))?;
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(20)) => ()
        }
    }
}

#[cfg(unix)]
struct TerminalMode(Option<libc::termios>);
#[cfg(unix)]
impl TerminalMode {
    fn raw() -> anyhow::Result<Self> {
        use std::io::IsTerminal;
        if !std::io::stdin().is_terminal() {
            return Ok(Self(None));
        }
        unsafe {
            let mut original: libc::termios = std::mem::zeroed();
            anyhow::ensure!(
                libc::tcgetattr(0, &mut original) == 0,
                "Terminal mode unavailable"
            );
            let mut raw = original;
            libc::cfmakeraw(&mut raw);
            anyhow::ensure!(
                libc::tcsetattr(0, libc::TCSANOW, &raw) == 0,
                "Terminal mode unavailable"
            );
            Ok(Self(Some(original)))
        }
    }
}
#[cfg(unix)]
impl Drop for TerminalMode {
    fn drop(&mut self) {
        if let Some(original) = &self.0 {
            unsafe {
                libc::tcsetattr(0, libc::TCSANOW, original);
            }
        }
    }
}
#[cfg(windows)]
use warp_agent_bus::companion_ctl::{terminal_size, TerminalMode};

#[cfg(unix)]
fn terminal_size() -> (u32, u32) {
    #[cfg(unix)]
    unsafe {
        let mut size: libc::winsize = std::mem::zeroed();
        if libc::ioctl(0, libc::TIOCGWINSZ, &mut size) == 0
            && (1..=1000).contains(&size.ws_col)
            && (1..=1000).contains(&size.ws_row)
        {
            return (size.ws_col as u32, size.ws_row as u32);
        }
    }
    (80, 24)
}
