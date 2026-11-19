//! Private `ctl` dispatcher for tasks owned by the remote Companion.
use crate::{
    companion::{guardian, Route},
    ssh_remote::{ConnectionError, HostClient},
};
use remote_protocol::proto::*;
use std::{path::{Path, PathBuf}, time::Duration};

const EXIT_OK: i32 = 0;
const EXIT_DOMAIN: i32 = 1;
const EXIT_SYNTAX: i32 = 2;
const PAGE_LIMIT: u32 = 32;
/// UTF-8 survives the Windows console and direct Unix PTY paths.
pub const DETACH_INPUT: &[u8] = "\u{effe}warpai:detach".as_bytes();
const HELP: &str = "\
warpai run <program> [args...] [--name <name>]
    Start a guarded task in this remote directory and attach to it.
    The name defaults to the executable's filename.
    Use run -- <program> [args...] to forward every program argument literally.

warpai list [--project <path>] [--id <session>]
    List guardian sessions for this account.

warpai resume [<name>] [--id <session>] [--project <path>] [--takeover]
    Attach the current, unique or last-used live session in this helper's scope.
    Use Warpai's Detach guarded task shortcut, or EOF, to leave the task running.

warpai stop [<name>] [--id <session>] [--project <path>]
    Stop every guarded run in this account, or one exact resolved run.

Exit codes: 0 success, 1 domain failure or unconfirmed stop, 2 invalid syntax
or an ambiguous named target. Bare stop covers the account. Use --project to
target a specific project root.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    All,
    Id(String),
    Name(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Version,
    Launch {
        name: Option<String>,
        program: String,
        arguments: Vec<String>,
    },
    List {
        project: Option<String>,
        id: Option<String>,
    },
    Resume {
        project: Option<String>,
        selection: Selection,
        takeover: bool,
    },
    Stop {
        project: Option<String>,
        selection: Selection,
    },
}

/// Parse a control command without evaluating any shell input.
pub fn parse(arguments: &[String]) -> Result<Command, String> {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Err("Missing command. Use run, list, resume or stop.".into());
    };
    let rest = &arguments[1..];
    match command {
        "-h" | "--help" | "help" => Ok(Command::Help),
        "-V" | "--version" | "version" => Ok(Command::Version),
        "run" => parse_launch(rest),
        "list" => parse_list(rest),
        "resume" => parse_resume(rest),
        "stop" => parse_stop(rest),
        other => Err(format!("Unknown command: {other}")),
    }
}

fn parse_launch(arguments: &[String]) -> Result<Command, String> {
    if matches!(arguments, [flag] if flag == "--help" || flag == "-h") {
        return Ok(Command::Help);
    }
    let (name, command) = match arguments {
        [separator, command @ ..] if separator == "--" => (None, command),
        [flag, name, separator, command @ ..] if flag == "--name" && separator == "--" => {
            (Some(name), command)
        }
        [program, rest @ ..] if !program.starts_with('-') => match rest {
            [.., flag, name] if flag == "--name" => {
                (Some(name), &arguments[..arguments.len() - 2])
            }
            [.., flag] if flag == "--name" => return Err("Missing value for --name.".into()),
            _ => (None, arguments),
        },
        _ => return Err("Usage: warpai run <program> [args...] [--name <name>]".into()),
    };
    if let Some(name) = name {
        validate_name(name)?;
    }
    let (program, arguments) = command.split_first().ok_or("Missing executable.")?;
    if program.is_empty() || program.len() > 4096 || program.chars().any(char::is_control)
        || arguments.len() > 64
        || arguments.iter().any(|argument| argument.len() > 8192 || argument.contains('\0'))
        || arguments.iter().map(String::len).sum::<usize>() > 65536
    {
        return Err("Invalid native command or arguments.".into());
    }
    Ok(Command::Launch { name: name.cloned(), program: program.clone(), arguments: arguments.to_vec() })
}

struct Flags {
    project: Option<String>,
    id: Option<String>,
    name: Option<String>,
    takeover: bool,
    help: bool,
}

fn parse_flags(
    arguments: &[String],
    allow_name: bool,
    allow_takeover: bool,
) -> Result<Flags, String> {
    let mut flags = Flags {
        project: None,
        id: None,
        name: None,
        takeover: false,
        help: false,
    };
    let mut literal = false;
    let mut index = 0;
    while index < arguments.len() {
        let argument = arguments[index].as_str();
        index += 1;
        if !literal {
            match argument {
                "--" => {
                    literal = true;
                    continue;
                }
                "-h" | "--help" => {
                    flags.help = true;
                    continue;
                }
                "--takeover" if allow_takeover => {
                    if flags.takeover {
                        return Err("Duplicate --takeover".into());
                    }
                    flags.takeover = true;
                    continue;
                }
                _ => {}
            }
            if let Some(value) = argument.strip_prefix("--project=") {
                set_once(&mut flags.project, value, "--project")?;
                continue;
            }
            if let Some(value) = argument.strip_prefix("--id=") {
                set_once(&mut flags.id, value, "--id")?;
                continue;
            }
            match argument {
                "--project" | "--id" => {
                    let value = arguments
                        .get(index)
                        .ok_or_else(|| format!("Missing value for {argument}"))?;
                    index += 1;
                    set_once(
                        if argument == "--project" {
                            &mut flags.project
                        } else {
                            &mut flags.id
                        },
                        value,
                        argument,
                    )?;
                    continue;
                }
                value if value.starts_with('-') => {
                    return Err(format!("Unknown option: {value}"));
                }
                _ => {}
            }
        }
        if !allow_name {
            return Err("Unexpected argument. list accepts only --project and --id.".into());
        }
        if flags.name.replace(argument.to_owned()).is_some() {
            return Err("Only one session name is accepted".into());
        }
    }
    if flags.help {
        return Ok(flags);
    }
    if let Some(project) = &flags.project {
        validate_path(project)?;
    }
    if let Some(id) = &flags.id {
        validate_session_id(id)?;
    }
    if let Some(name) = &flags.name {
        validate_name(name)?;
    }
    if flags.id.is_some() && flags.name.is_some() {
        return Err("Use a name or --id, not both".into());
    }
    Ok(flags)
}

fn set_once(slot: &mut Option<String>, value: &str, flag: &str) -> Result<(), String> {
    if slot.replace(value.to_owned()).is_some() {
        return Err(format!("Duplicate {flag}"));
    }
    Ok(())
}

fn validate_path(value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
        return Err("Invalid project path".into());
    }
    Ok(())
}

fn validate_session_id(value: &str) -> Result<(), String> {
    if uuid::Uuid::parse_str(value).is_err() {
        return Err("Invalid session ID".into());
    }
    Ok(())
}

fn validate_name(value: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err("Invalid session name".into());
    }
    Ok(())
}

fn parse_list(arguments: &[String]) -> Result<Command, String> {
    let flags = parse_flags(arguments, false, false)?;
    if flags.help {
        return Ok(Command::Help);
    }
    Ok(Command::List {
        project: flags.project,
        id: flags.id,
    })
}

fn parse_resume(arguments: &[String]) -> Result<Command, String> {
    let flags = parse_flags(arguments, true, true)?;
    if flags.help {
        return Ok(Command::Help);
    }
    let selection = match (flags.id, flags.name) {
        (Some(id), None) => Selection::Id(id),
        (None, Some(name)) => Selection::Name(name),
        (None, None) => Selection::All,
        (Some(_), Some(_)) => return Err("Use a name or --id, not both".into()),
    };
    Ok(Command::Resume {
        project: flags.project,
        selection,
        takeover: flags.takeover,
    })
}

fn parse_stop(arguments: &[String]) -> Result<Command, String> {
    let flags = parse_flags(arguments, true, false)?;
    if flags.help {
        return Ok(Command::Help);
    }
    let selection = match (flags.id, flags.name) {
        (Some(id), None) => Selection::Id(id),
        (None, Some(name)) => Selection::Name(name),
        (None, None) => Selection::All,
        (Some(_), Some(_)) => return Err("Use a name or --id, not both".into()),
    };
    Ok(Command::Stop {
        project: flags.project,
        selection,
    })
}

fn route() -> Route {
    #[cfg(all(target_os = "linux", feature = "wsl_companion"))]
    {
        Route::Wsl
    }
    #[cfg(not(all(target_os = "linux", feature = "wsl_companion")))]
    {
        Route::Ssh
    }
}

/// Entry point; returns the process exit code.
pub async fn run(arguments: &[String]) -> i32 {
    let command = match parse(arguments) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("{message}");
            return EXIT_SYNTAX;
        }
    };
    match command {
        Command::Help => {
            println!("{HELP}");
            EXIT_OK
        }
        Command::Version => {
            println!(
                "warpai-companion {} protocol {}",
                crate::installation::RELEASE_VERSION,
                remote_protocol::managed::PROTOCOL_MAJOR
            );
            EXIT_OK
        }
        Command::Launch { name, program, arguments } => launch(name, &program, arguments).await,
        Command::List { project, id } => list(project, id).await,
        Command::Resume {
            project,
            selection,
            takeover,
        } => resume(project, selection, takeover).await,
        Command::Stop { project, selection } => stop(project, selection).await,
    }
}

fn resolve_executable(program: &str, root: &Path) -> Option<PathBuf> {
    fn usable(path: PathBuf) -> Option<PathBuf> {
        let metadata = path.metadata().ok()?;
        if !metadata.is_file() {
            return None;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o111 == 0 {
                return None;
            }
        }
        #[cfg(windows)]
        if !path.extension().and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("exe") || value.eq_ignore_ascii_case("com"))
        {
            return None;
        }
        #[cfg(all(feature = "wsl_companion", target_os = "linux"))]
        if !crate::wsl_setup::native_executable(&path) {
            return None;
        }
        // Interpreters such as virtualenv Python use the invoked path to find their environment.
        Some(path)
    }

    let program = Path::new(program);
    if program.is_absolute() || program.components().count() != 1 {
        return usable(root.join(program));
    }
    for entry in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let candidate = root.join(entry).join(program);
        if let Some(executable) = usable(candidate.clone()) {
            return Some(executable);
        }
        #[cfg(windows)]
        if candidate.extension().is_none() {
            if let Some(executable) = usable(candidate.with_extension("exe"))
                .or_else(|| usable(candidate.with_extension("com")))
            {
                return Some(executable);
            }
        }
    }
    None
}

async fn launch(name: Option<String>, program: &str, arguments: Vec<String>) -> i32 {
    let Ok(root) = std::env::current_dir() else {
        eprintln!("Remote working directory is unavailable.");
        return EXIT_DOMAIN;
    };
    let Some(project) = root.to_str().filter(|path| validate_path(path).is_ok()) else {
        eprintln!("Remote working directory is invalid.");
        return EXIT_DOMAIN;
    };
    let Some(executable) = resolve_executable(program, &root)
        .and_then(|file| file.to_str().map(str::to_owned)) else {
        eprintln!("Native executable was not found for this remote account.");
        return EXIT_DOMAIN;
    };
    if guardian::connect(route()).await.is_err() {
        eprintln!("Could not start the native account guardian.");
        return EXIT_DOMAIN;
    }
    let Ok((mut client, _)) = connect(&Some(project.into()), true).await else {
        return EXIT_DOMAIN;
    };
    let (columns, rows) = terminal_size();
    let label = name.or_else(|| Path::new(program).file_name().and_then(|value| value.to_str())
        .filter(|value| validate_name(value).is_ok()).map(str::to_owned));
    let kind = if arguments.is_empty() && matches!(
        Path::new(program).file_stem().and_then(|value| value.to_str()),
        Some("bash" | "zsh" | "sh" | "fish" | "powershell" | "pwsh" | "cmd")
    ) { TerminalSessionKind::Shell } else { TerminalSessionKind::Command };
    match client.terminal_launch(TerminalLaunch {
        fence: client.fence().cloned(),
        session_id: uuid::Uuid::new_v4().to_string(),
        executable,
        arguments,
        columns,
        rows,
        guarded: true,
        label,
        session_kind: Some(kind as i32),
        ..Default::default()
    }).await {
        Ok(mut state) => attach_loop(&mut client, &mut state, false).await,
        Err(error) => {
            eprintln!("Guarded task could not start ({error}).");
            EXIT_DOMAIN
        }
    }
}

fn encode_text(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_control() { '?' } else { c })
        .collect()
}

fn row_line(row: &AccountCatalogRow) -> String {
    let state = if row.state.is_empty() {
        "unknown"
    } else {
        row.state.as_str()
    };
    let label = row
        .label
        .as_deref()
        .map(encode_text)
        .unwrap_or_else(|| "-".into());
    let exit = row
        .exit_status
        .map(|code| format!(" exit={code}"))
        .unwrap_or_default();
    let writer = match row.writer_attached {
        Some(true) => "attached",
        Some(false) => "free",
        None => "unknown",
    };
    format!(
        "{}  {}  {}  {}  project={}  mru={}  gen={}  writer={}{}",
        row.session_id,
        label,
        encode_text(&row.kind),
        state,
        encode_text(&row.project_id),
        row.mru_seq,
        row.attachment_generation,
        writer,
        exit
    )
}

fn is_live(row: &AccountCatalogRow) -> bool {
    row.state == "live"
}

fn route_name(route: Route) -> &'static str {
    match route {
        Route::Ssh => "ssh",
        Route::Wsl => "wsl",
    }
}

/// Account discovery and bare stop do not require a project admission.
async fn connect(
    project: &Option<String>,
    attach: bool,
) -> Result<(HostClient, String), i32> {
    let active = route();
    let (stream, guardian, boot) = match guardian::probe(active, attach).await {
        Ok(connected) => connected,
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
            eprintln!("Guardian connection capacity is busy. Try again after another attachment closes.");
            return Err(EXIT_DOMAIN);
        }
        Err(_) => {
            eprintln!("Guardian is unavailable for this account.");
            return Err(EXIT_DOMAIN);
        }
    };
    let mut client = match HostClient::connect_guardian_stream(stream, None).await {
        Ok(client) => client,
        Err(_) => {
            eprintln!("Guardian connection or project admission failed.");
            return Err(EXIT_DOMAIN);
        }
    };
    if !client.is_account_guardian() || !client.account_fence().is_some_and(|fence| {
        fence.guardian_id == guardian.to_string() && fence.boot_id == boot.to_string()
    }) {
        eprintln!("Installed Companion does not provide guardian sessions.");
        return Err(EXIT_DOMAIN);
    }
    if let Some(root) = project.as_deref() {
        if client.open_project(root).await.is_err() {
            eprintln!("Project admission failed.");
            return Err(EXIT_DOMAIN);
        }
    }
    let filter = client
        .fence()
        .map(|fence| fence.project_id.clone())
        .unwrap_or_default();
    Ok((client, filter))
}

pub(crate) async fn rows(client: &mut HostClient, scope_id: &str, project_id: &str) -> Result<Vec<AccountCatalogRow>, i32> {
    let filter = AccountCatalogFilter {
        scope_id: scope_id.to_owned(),
        project_id: project_id.to_owned(),
    };
    let mut cursor = String::new();
    let mut rows = Vec::new();
    loop {
        let Ok(page) = client
            .account_catalog_list(filter.clone(), PAGE_LIMIT, cursor.clone())
            .await
        else {
            eprintln!("Guardian catalog is unavailable or changed. Try again.");
            return Err(EXIT_DOMAIN);
        };
        let advanced = !page.rows.is_empty();
        rows.extend(page.rows);
        match page.next {
            None => return Ok(rows),
            Some(next) if advanced && rows.len() < page.total as usize && next != cursor => {
                cursor = next;
            }
            Some(_) => {
                eprintln!("Guardian catalog returned an invalid page.");
                return Err(EXIT_DOMAIN);
            }
        }
    }
}

async fn list(project: Option<String>, id: Option<String>) -> i32 {
    let (mut client, filter) = match connect(&project, false).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    let rows = match rows(&mut client, "", &filter).await {
        Ok(rows) => rows,
        Err(code) => return code,
    };
    println!("Guardian: ready ({})", route_name(route()));
    let mut shown = 0;
    for row in rows
        .iter()
        .filter(|row| id.as_ref().is_none_or(|id| &row.session_id == id))
    {
        println!("{}", row_line(row));
        shown += 1;
    }
    if shown == 0 {
        println!("No guardian sessions.");
    }
    EXIT_OK
}

/// Unique-name, exact-ID or most-recently-used live selection.
fn select<'a>(
    rows: &'a [AccountCatalogRow],
    selection: &Selection,
) -> Result<&'a AccountCatalogRow, i32> {
    let live: Vec<&AccountCatalogRow> = rows.iter().filter(|row| is_live(row)).collect();
    match selection {
        Selection::Id(id) => live
            .into_iter()
            .find(|row| &row.session_id == id)
            .ok_or_else(|| {
                eprintln!("No live session matches {id}.");
                EXIT_DOMAIN
            }),
        Selection::Name(name) => {
            let named: Vec<&AccountCatalogRow> = live
                .into_iter()
                .filter(|row| row.label.as_deref() == Some(name.as_str()))
                .collect();
            match named.len() {
                0 => {
                    eprintln!("No live session is named {name}.");
                    Err(EXIT_DOMAIN)
                }
                1 => Ok(named[0]),
                _ => {
                    eprintln!("Session name is ambiguous; use --id:");
                    for row in named {
                        println!("  {}", row.session_id);
                    }
                    Err(EXIT_SYNTAX)
                }
            }
        }
        Selection::All => match live.len() {
            0 => {
                eprintln!("No live session is resumable.");
                Err(EXIT_DOMAIN)
            }
            1 => Ok(live[0]),
            // Highest daemon record of foreground use; stable ID breaks ties.
            _ => Ok(live
                .into_iter()
                .max_by(|left, right| {
                    left.mru_seq
                        .cmp(&right.mru_seq)
                        .then_with(|| left.created_seq.cmp(&right.created_seq))
                        .then_with(|| right.session_id.cmp(&left.session_id))
                })
                .expect("Live selection is not empty")),
        },
    }
}

async fn resume(project: Option<String>, selection: Selection, takeover: bool) -> i32 {
    let (mut client, filter) = match connect(&project, true).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    if matches!(selection, Selection::All) && project.is_none()
        && client.current_guarded_session().is_some()
    {
        println!("Already in the current guarded session.");
        return EXIT_OK;
    }
    let mut rows = match rows(&mut client, route_name(route()), &filter).await {
        Ok(rows) => rows,
        Err(code) => return code,
    };
    let current = loop {
        let selected = match select(&rows, &selection) {
            Ok(row) => row,
            Err(code) => return code,
        };
        if client.current_guarded_session().is_some_and(|current| {
            current.session_id == selected.session_id && current.run_id == selected.run_id
        }) {
            println!("Already in the current guarded session.");
            return EXIT_OK;
        }
        let inspected = client
            .account_catalog_inspect(selected.session_id.clone(), selected.run_id.clone())
            .await;
        let available = match inspected {
            Ok(row) if row.session_id != selected.session_id || row.run_id != selected.run_id
                || row.project_id != selected.project_id => Err(ConnectionError::StaleAttachment),
            Ok(row) if is_live(&row) && project.is_none() => {
                client.open_guarded_project(&row).await.map(|()| row)
            }
            Ok(row) if is_live(&row) => Ok(row),
            Ok(_) => Err(ConnectionError::NotFound),
            Err(error) => Err(error),
        };
        match available {
            Ok(row) => break row,
            Err(ConnectionError::NotFound) if matches!(selection, Selection::All) => {
                let removed = selected.session_id.clone();
                rows.retain(|row| row.session_id != removed);
            }
            Err(ConnectionError::NotFound) => {
                eprintln!("Selected session is no longer live.");
                return EXIT_DOMAIN;
            }
            Err(ConnectionError::InvalidInput | ConnectionError::FeatureUnavailable) if project.is_none() => {
                eprintln!("Selected session project could not be opened. Use --project <root> for an older guardian.");
                return EXIT_DOMAIN;
            }
            Err(error) => {
                eprintln!("Selected session could not be opened: {error}");
                return EXIT_DOMAIN;
            }
        }
    };
    let attach = TerminalControl {
        fence: client.fence().cloned(),
        session_id: current.session_id.clone(),
        run_id: current.run_id.clone(),
        action: TerminalAction::TerminalAttach as i32,
        attachment_id: String::new(),
        attachment_generation: if takeover {
            current.attachment_generation
        } else {
            0
        },
        takeover,
        ..Default::default()
    };
    let mut state = match client.terminal_control(attach).await {
        Ok(state) => state,
        Err(ConnectionError::Conflict) => {
            if takeover {
                eprintln!("Takeover was rejected; the writer epoch changed.");
            } else {
                eprintln!(
                    "Session is owned by another writer. Rerun with --takeover to acquire it."
                );
            }
            return EXIT_DOMAIN;
        }
        Err(_) => {
            eprintln!("Session attach failed.");
            return EXIT_DOMAIN;
        }
    };
    attach_loop(&mut client, &mut state, true).await
}

/// Direct TTY attachment; output offsets and input receipts stay authoritative.
async fn attach_loop(client: &mut HostClient, state: &mut TerminalState, resume: bool) -> i32 {
    use std::io::IsTerminal;
    use tokio::io::AsyncWriteExt;
    let signals = match TerminationSignals::new() {
        Ok(signals) => signals,
        Err(_) => {
            eprintln!("Could not install terminal cleanup handlers.");
            detach(client, state).await;
            return EXIT_DOMAIN;
        }
    };
    #[cfg(unix)]
    let raw = RawTerminal::enable(libc::STDIN_FILENO);
    #[cfg(windows)]
    let raw = TerminalMode::raw();
    let raw = match raw {
        Ok(raw) => raw,
        Err(_) => {
            eprintln!("Could not configure the local terminal.");
            detach(client, state).await;
            return EXIT_DOMAIN;
        }
    };
    let is_terminal = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let mut stdout = tokio::io::stdout();
    if is_terminal {
        let _ = stdout.write_all(b"\x1b]9278;g;1\x07").await;
        let _ = stdout.flush().await;
    }
    let mut alternate = false;
    let result = tokio::select! {
        biased;
        signal = signals.recv() => Err(signal),
        result = attach_inner(client, state, resume, &mut alternate) => Ok(result),
    };
    if alternate {
        let _ = stdout.write_all(b"\x1b[?1049l").await;
    }
    if is_terminal {
        let _ = stdout.write_all(b"\x1b]9278;g;0\x07").await;
    }
    if alternate || is_terminal {
        let _ = stdout.flush().await;
    }
    drop(raw);
    match result {
        Ok(result) => {
            detach(client, state).await;
            result.unwrap_or_else(|code| code)
        }
        Err(signal) => {
            // The in-flight managed frame may be incomplete. Closing the
            // transport detaches without sending another frame on that stream.
            client.disconnect();
            128 + signal
        }
    }
}

#[cfg(unix)]
struct TerminationSignals {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
    hangup: tokio::signal::unix::Signal,
    quit: tokio::signal::unix::Signal,
}

#[cfg(unix)]
impl TerminationSignals {
    fn new() -> std::io::Result<Self> {
        use tokio::signal::unix::{signal, SignalKind};
        Ok(Self {
            interrupt: signal(SignalKind::interrupt())?,
            terminate: signal(SignalKind::terminate())?,
            hangup: signal(SignalKind::hangup())?,
            quit: signal(SignalKind::quit())?,
        })
    }

    async fn recv(mut self) -> i32 {
        tokio::select! {
            _ = self.interrupt.recv() => libc::SIGINT,
            _ = self.terminate.recv() => libc::SIGTERM,
            _ = self.hangup.recv() => libc::SIGHUP,
            _ = self.quit.recv() => libc::SIGQUIT,
        }
    }
}

#[cfg(windows)]
struct TerminationSignals {
    interrupt: tokio::signal::windows::CtrlC,
    break_event: tokio::signal::windows::CtrlBreak,
}

#[cfg(windows)]
impl TerminationSignals {
    fn new() -> std::io::Result<Self> {
        Ok(Self {
            interrupt: tokio::signal::windows::ctrl_c()?,
            break_event: tokio::signal::windows::ctrl_break()?,
        })
    }

    async fn recv(mut self) -> i32 {
        tokio::select! {
            _ = self.interrupt.recv() => 2,
            _ = self.break_event.recv() => 21,
        }
    }
}

async fn attach_inner(
    client: &mut HostClient,
    state: &mut TerminalState,
    resume: bool,
    alternate: &mut bool,
) -> Result<i32, i32> {
    use crate::screen_detection::native::NativeScreen;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut buffer = [0u8; 4096];
    let mut offset = 0u64;
    let mut sequence = 0u64;
    let mut dimensions = None;
    let mut detach_prefix = 0;
    let mut truncated_notice = false;
    let mut initial = resume;
    let mut projection: Option<NativeScreen> = None;
    loop {
        let resized = terminal_size();
        if Some(resized) != dimensions {
            let resize = TerminalControl {
                fence: client.fence().cloned(),
                session_id: state.session_id.clone(),
                run_id: state.run_id.clone(),
                action: TerminalAction::TerminalResize as i32,
                columns: resized.0,
                rows: resized.1,
                resize_sequence: state.accepted_resize_sequence.saturating_add(1),
                attachment_id: state.attachment_id.clone(),
                attachment_generation: state.attachment_generation,
                ..Default::default()
            };
            let Ok(reply) = client.terminal_control(resize).await else {
                eprintln!("\r\nSession resize was rejected.");
                return Err(EXIT_DOMAIN);
            };
            *state = reply;
            dimensions = Some(resized);
            if let Some(screen) = &mut projection {
                screen.resize(resized.1 as u16, resized.0 as u16);
            }
        }
        if std::mem::take(&mut initial) {
            let cut = state.output_end;
            let mut screen = NativeScreen::new(resized.1 as u16, resized.0 as u16);
            // At most one ring of retained output. Replay into a local parser only.
            for _ in 0..8 {
                if offset >= cut {
                    break;
                }
                let reply = client.terminal_control(TerminalControl {
                    fence: client.fence().cloned(),
                    session_id: state.session_id.clone(),
                    run_id: state.run_id.clone(),
                    action: TerminalAction::TerminalRead as i32,
                    output_offset: offset,
                    attachment_id: state.attachment_id.clone(),
                    attachment_generation: state.attachment_generation,
                    ..Default::default()
                }).await.map_err(|_| EXIT_DOMAIN)?;
                truncated_notice |= reply.output_truncated;
                let count = reply.output.len()
                    .min(cut.saturating_sub(reply.output_offset) as usize);
                if count == 0 {
                    break;
                }
                screen.process(&reply.output[..count]);
                offset = reply.output_offset + count as u64;
            }
            if truncated_notice {
                stdout.write_all(b"\r\n[earlier output is no longer retained]\r\n")
                    .await.map_err(|_| EXIT_DOMAIN)?;
            }
            if cut > 0 {
                stdout.write_all(&screen.cli_view(*alternate)).await.map_err(|_| EXIT_DOMAIN)?;
                *alternate = screen.alternate_screen();
                offset = cut;
            }
            stdout.flush().await.map_err(|_| EXIT_DOMAIN)?;
            projection = Some(screen);
        }
        let control = TerminalControl {
            fence: client.fence().cloned(),
            session_id: state.session_id.clone(),
            run_id: state.run_id.clone(),
            action: TerminalAction::TerminalRead as i32,
            output_offset: offset,
            attachment_id: state.attachment_id.clone(),
            attachment_generation: state.attachment_generation,
            ..Default::default()
        };
        let reply = match client.terminal_control(control).await {
            Ok(reply) => reply,
            Err(ConnectionError::StaleAttachment) => {
                eprintln!("\r\nSession attachment is stale; resume again.");
                return Err(EXIT_DOMAIN);
            }
            Err(_) => {
                eprintln!("\r\nGuardian connection was lost.");
                return Err(EXIT_DOMAIN);
            }
        };
        if reply.output_truncated && !truncated_notice {
            truncated_notice = true;
            stdout.write_all(b"\r\n[earlier output is no longer retained]\r\n")
                .await.map_err(|_| EXIT_DOMAIN)?;
        }
        if reply.output_truncated {
            let mut screen = NativeScreen::new(resized.1 as u16, resized.0 as u16);
            screen.process(&reply.output);
            stdout.write_all(&screen.cli_view(*alternate)).await.map_err(|_| EXIT_DOMAIN)?;
            *alternate = screen.alternate_screen();
            projection = Some(screen);
        } else {
            let screen = projection.get_or_insert_with(|| {
                NativeScreen::new(resized.1 as u16, resized.0 as u16)
            });
            screen.process(&reply.output);
            *alternate = screen.alternate_screen();
            stdout.write_all(&reply.output).await.map_err(|_| EXIT_DOMAIN)?;
        }
        stdout.flush().await.map_err(|_| EXIT_DOMAIN)?;
        offset = reply.output_offset + reply.output.len() as u64;
        *state = reply;
        if state.processes_active == Some(false)
            && state.output_closed
            && offset == state.output_end
        {
            return Ok(state.exit_code.unwrap_or(EXIT_OK).clamp(0, 255));
        }
        let wait = if offset < state.output_end { Duration::ZERO } else { Duration::from_millis(20) };
        tokio::select! {
            biased;
            read = stdin.read(&mut buffer) => {
                let count = match read {
                    Ok(count) => count,
                    Err(_) => 0,
                };
                let (send, detach_now) = if count == 0 {
                    // EOF detaches; it never stops the guardian-owned process.
                    (DETACH_INPUT[..detach_prefix].to_vec(), true)
                } else {
                    filter_detach_signal(&buffer[..count], &mut detach_prefix)
                };
                if send.is_empty() {
                    if detach_now {
                        return Ok(EXIT_OK);
                    }
                    continue;
                }
                sequence += 1;
                let input = TerminalControl {
                    fence: client.fence().cloned(),
                    session_id: state.session_id.clone(),
                    run_id: state.run_id.clone(),
                    action: TerminalAction::TerminalInput as i32,
                    input: send,
                    input_sequence: sequence,
                    attachment_id: state.attachment_id.clone(),
                    attachment_generation: state.attachment_generation,
                    ..Default::default()
                };
                match client.terminal_control(input).await {
                    Ok(reply) => *state = reply,
                    Err(ConnectionError::DeliveryUnknown) => {
                        eprintln!("\r\nInput delivery is unconfirmed; detaching.");
                        return Err(EXIT_DOMAIN);
                    }
                    Err(_) => {
                        eprintln!("\r\nSession input was rejected; detaching.");
                        return Err(EXIT_DOMAIN);
                    }
                }
                if detach_now {
                    return Ok(EXIT_OK);
                }
            }
            _ = tokio::time::sleep(wait) => ()
        }
    }
}

fn filter_detach_signal(bytes: &[u8], matched: &mut usize) -> (Vec<u8>, bool) {
    let mut send = Vec::with_capacity(bytes.len() + *matched);
    for &byte in bytes {
        if byte == DETACH_INPUT[*matched] {
            *matched += 1;
            if *matched == DETACH_INPUT.len() {
                *matched = 0;
                return (send, true);
            }
            continue;
        }
        send.extend_from_slice(&DETACH_INPUT[..*matched]);
        // The start byte only occurs once in the signal.
        *matched = usize::from(byte == DETACH_INPUT[0]);
        if *matched == 0 {
            send.push(byte);
        }
    }
    (send, false)
}

async fn detach(client: &mut HostClient, state: &TerminalState) {
    let _ = client
        .terminal_control(TerminalControl {
            fence: client.fence().cloned(),
            session_id: state.session_id.clone(),
            run_id: state.run_id.clone(),
            action: TerminalAction::TerminalDetach as i32,
            attachment_id: state.attachment_id.clone(),
            attachment_generation: state.attachment_generation,
            ..Default::default()
        })
        .await;
}

async fn stop(project: Option<String>, selection: Selection) -> i32 {
    let (mut client, filter) = match connect(&project, false).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    let target = match selection {
        Selection::All => None,
        selection => {
            let rows = match rows(&mut client, "", &filter).await {
                Ok(rows) => rows,
                Err(code) => return code,
            };
            let selected = match select(&rows, &selection) {
                Ok(row) => row.clone(),
                Err(code) => return code,
            };
            Some(AccountStopTarget {
                session_id: selected.session_id,
                run_id: selected.run_id,
            })
        }
    };
    let receipts = match client.account_stop(target, filter).await {
        Ok(receipts) => receipts,
        Err(_) => {
            eprintln!("Guardian refused the stop request.");
            return EXIT_DOMAIN;
        }
    };
    for receipt in &receipts.receipts {
        let outcome = match AccountStopOutcome::try_from(receipt.outcome) {
            Ok(AccountStopOutcome::Requested) => "requested",
            Ok(AccountStopOutcome::Observed) => "observed",
            _ => "unknown",
        };
        let exit = receipt
            .exit_status
            .map(|code| format!(" exit={code}"))
            .unwrap_or_default();
        println!(
            "{} run={} {}{}",
            receipt.session_id, receipt.run_id, outcome, exit
        );
    }
    if receipts.receipts.is_empty() {
        println!("No guarded runs to stop.");
    }
    match AccountStopStatus::try_from(receipts.status) {
        Ok(AccountStopStatus::Completed) => EXIT_OK,
        Ok(AccountStopStatus::Partial) | Ok(AccountStopStatus::Rejected) => EXIT_DOMAIN,
        _ => EXIT_DOMAIN,
    }
}

#[cfg(unix)]
fn terminal_size() -> (u32, u32) {
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

#[cfg(unix)]
struct RawTerminal {
    fd: std::os::fd::RawFd,
    original: Option<libc::termios>,
}

#[cfg(unix)]
impl RawTerminal {
    fn enable(fd: std::os::fd::RawFd) -> std::io::Result<Self> {
        if unsafe { libc::isatty(fd) } != 1 {
            return Ok(Self { fd, original: None });
        }
        unsafe {
            let mut original = std::mem::zeroed();
            if libc::tcgetattr(fd, &mut original) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let mut raw = original;
            libc::cfmakeraw(&mut raw);
            if libc::tcsetattr(fd, libc::TCSANOW, &raw) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(Self { fd, original: Some(original) })
        }
    }
}

#[cfg(unix)]
impl Drop for RawTerminal {
    fn drop(&mut self) {
        if let Some(original) = &self.original {
            unsafe {
                if libc::tcsetattr(self.fd, libc::TCSANOW, original) == 0 {
                    // Darwin marks pending input for reprocessing when canonical mode returns.
                    libc::tcflush(self.fd, libc::TCIFLUSH);
                }
            }
        }
    }
}

#[cfg(windows)]
pub struct TerminalMode(
    Vec<(
        windows::Win32::Foundation::HANDLE,
        windows::Win32::System::Console::CONSOLE_MODE,
    )>,
);
#[cfg(windows)]
impl TerminalMode {
    pub fn raw() -> anyhow::Result<Self> {
        use std::io::IsTerminal;
        use windows::Win32::System::Console::*;
        let mut modes = Self(Vec::new());
        if !std::io::stdin().is_terminal() {
            return Ok(modes);
        }
        unsafe {
            for (id, input) in [(STD_INPUT_HANDLE, true), (STD_OUTPUT_HANDLE, false)] {
                let handle = GetStdHandle(id)?;
                let mut original = CONSOLE_MODE::default();
                GetConsoleMode(handle, &mut original)?;
                let mode = if input {
                    (original & !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT))
                        | ENABLE_VIRTUAL_TERMINAL_INPUT
                } else {
                    original | ENABLE_PROCESSED_OUTPUT | ENABLE_VIRTUAL_TERMINAL_PROCESSING
                };
                modes.0.push((handle, original));
                SetConsoleMode(handle, mode)?;
            }
        }
        Ok(modes)
    }
}
#[cfg(windows)]
impl Drop for TerminalMode {
    fn drop(&mut self) {
        for (handle, original) in &self.0 {
            unsafe {
                let _ = windows::Win32::System::Console::SetConsoleMode(*handle, *original);
            }
        }
    }
}

#[cfg(windows)]
pub fn terminal_size() -> (u32, u32) {
    unsafe {
        use windows::Win32::System::Console::*;
        if let Ok(handle) = GetStdHandle(STD_OUTPUT_HANDLE) {
            let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
            if GetConsoleScreenBufferInfo(handle, &mut info).is_ok() {
                let columns = i32::from(info.srWindow.Right) - i32::from(info.srWindow.Left) + 1;
                let rows = i32::from(info.srWindow.Bottom) - i32::from(info.srWindow.Top) + 1;
                if (1..=1000).contains(&columns) && (1..=1000).contains(&rows) {
                    return (columns as u32, rows as u32);
                }
            }
        }
    }
    (80, 24)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn restore_original_terminal_attributes_on_detach() {
        use std::os::fd::{AsRawFd, FromRawFd};
        let mut master = -1;
        let mut slave = -1;
        let (_master, slave) = unsafe {
            assert_eq!(
                libc::openpty(
                    &mut master, &mut slave, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(),
                ),
                0
            );
            (std::fs::File::from_raw_fd(master), std::fs::File::from_raw_fd(slave))
        };
        let fd = slave.as_raw_fd();
        let attributes = || unsafe {
            let mut attributes: libc::termios = std::mem::zeroed();
            assert_eq!(libc::tcgetattr(fd, &mut attributes), 0);
            attributes
        };
        let original = attributes();
        let raw = RawTerminal::enable(fd).unwrap();
        assert_eq!(attributes().c_lflag & (libc::ICANON | libc::ECHO | libc::ISIG | libc::IEXTEN), 0);
        drop(raw);
        let restored = attributes();
        assert_eq!(restored.c_iflag, original.c_iflag);
        assert_eq!(restored.c_oflag, original.c_oflag);
        assert_eq!(restored.c_cflag, original.c_cflag);
        assert_eq!(restored.c_lflag, original.c_lflag);
        assert_eq!(restored.c_cc, original.c_cc);
        assert_eq!(restored.c_ispeed, original.c_ispeed);
        assert_eq!(restored.c_ospeed, original.c_ospeed);
    }

    fn args(value: &[&str]) -> Vec<String> {
        value.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn guarded_launch_preserves_literal_argv_and_validates_the_name() {
        let named = Command::Launch {
            name: Some("任务一".into()),
            program: "/opt/my program".into(),
            arguments: args(&["", "$HOME", "--"]),
        };
        assert_eq!(
            parse(&args(&["run", "/opt/my program", "", "$HOME", "--", "--name", "任务一"])).unwrap(),
            named
        );
        assert_eq!(
            parse(&args(&["run", "--name", "任务一", "--", "/opt/my program", "", "$HOME", "--"])).unwrap(),
            named
        );
        assert_eq!(
            parse(&args(&["run", "btop"])).unwrap(),
            Command::Launch { name: None, program: "btop".into(), arguments: vec![] }
        );
        assert_eq!(
            parse(&args(&["run", "--", "python", "3.py", "--name", "child"])).unwrap(),
            Command::Launch {
                name: None,
                program: "python".into(),
                arguments: args(&["3.py", "--name", "child"]),
            }
        );
        for invalid in [
            vec!["run"],
            vec!["run", "--", ""],
            vec!["run", "--name", "  ", "--", "sh"],
            vec!["run", "sh", "--name", "  "],
            vec!["run", "sh", "--name"],
        ] {
            assert!(parse(&args(&invalid)).is_err());
        }
    }

    #[cfg(unix)]
    #[test]
    fn executable_resolution_preserves_the_invoked_path() {
        let root = tempfile::tempdir().unwrap();
        let launcher = root.path().join("task-tool");
        std::os::unix::fs::symlink("/bin/sh", &launcher).unwrap();
        assert_eq!(
            resolve_executable("./task-tool", root.path()),
            Some(root.path().join("./task-tool"))
        );
    }

    #[test]
    fn parser_accepts_only_the_guardian_commands() {
        assert_eq!(
            parse(&args(&["list"])).unwrap(),
            Command::List {
                project: None,
                id: None,
            }
        );
        assert_eq!(
            parse(&args(&["resume", "build"])).unwrap(),
            Command::Resume {
                project: None,
                selection: Selection::Name("build".into()),
                takeover: false,
            }
        );
        assert_eq!(
            parse(&args(&[
                "stop",
                "--project",
                "/p",
                "--id",
                "00000000-0000-0000-0000-000000000000"
            ]))
            .unwrap(),
            Command::Stop {
                project: Some("/p".into()),
                selection: Selection::Id("00000000-0000-0000-0000-000000000000".into()),
            }
        );
        assert!(parse(&args(&["start"])).is_err());
        assert!(parse(&args(&["resume", "--takeover", "--takeover"])).is_err());
        assert!(parse(&args(&["stop", "--nope"])).is_err());
        assert!(parse(&args(&["stop", "--id", "not-an-id"])).is_err());
        assert!(parse(&args(&["stop", "one", "two"])).is_err());
    }

    #[test]
    fn literal_marker_and_named_id_conflict_are_rejected() {
        assert_eq!(
            parse(&args(&["resume", "--", "-dash"])).unwrap(),
            Command::Resume {
                project: None,
                selection: Selection::Name("-dash".into()),
                takeover: false,
            }
        );
        assert!(parse(&args(&[
            "resume",
            "name",
            "--id",
            "00000000-0000-0000-0000-000000000000"
        ]))
        .is_err());
        assert!(parse(&args(&["resume", "--project", "/p", "name", "--takeover"])).is_ok());
    }

    #[test]
    fn selection_prefers_exact_id_then_unique_name_then_mru() {
        let row = |id: &str, label: Option<&str>, state: &str, mru: u64| AccountCatalogRow {
            session_id: id.into(),
            run_id: format!("run-{id}"),
            project_id: "p".into(),
            kind: "shell".into(),
            policy: "guarded".into(),
            label: label.map(str::to_owned),
            state: state.into(),
            created_seq: mru,
            mru_seq: mru,
            observed_ms: 0,
            exit_status: None,
            attachment_generation: 0,
            writer_attached: None,
        };
        let rows = vec![
            row("a", Some("same"), "live", 5),
            row("b", Some("same"), "live", 9),
            row("c", Some("only"), "live", 1),
            row("d", Some("gone"), "ended", 99),
        ];
        assert_eq!(select(&rows, &Selection::All).unwrap().session_id, "b");
        assert_eq!(
            select(&rows, &Selection::Name("only".into()))
                .unwrap()
                .session_id,
            "c"
        );
        assert_eq!(
            select(&rows, &Selection::Name("same".into())).unwrap_err(),
            EXIT_SYNTAX
        );
        assert_eq!(
            select(
                &rows,
                &Selection::Id("00000000-0000-0000-0000-000000000000".into())
            )
            .unwrap_err(),
            EXIT_DOMAIN
        );
        let only_live = vec![row("x", None, "live", 1), row("y", None, "unknown", 9)];
        assert_eq!(select(&only_live, &Selection::All).unwrap().session_id, "x");
        let no_history = [
            AccountCatalogRow { created_seq: 2, ..row("a", None, "live", 0) },
            AccountCatalogRow { created_seq: 3, ..row("z", None, "live", 0) },
        ];
        assert_eq!(select(&no_history, &Selection::All).unwrap().session_id, "z");
        assert_eq!(
            select(&[row("z", None, "ended", 3)], &Selection::All).unwrap_err(),
            EXIT_DOMAIN
        );
    }

    #[test]
    fn detach_signal_survives_split_reads_and_passes_ordinary_input() {
        let mut matched = 0;
        assert_eq!(
            filter_detach_signal(b"abc\x02q\x1b", &mut matched),
            (b"abc\x02q\x1b".to_vec(), false)
        );
        let split = 4;
        assert_eq!(
            filter_detach_signal(&DETACH_INPUT[..split], &mut matched),
            (Vec::new(), false)
        );
        assert_eq!(
            filter_detach_signal(b"x", &mut matched),
            ([&DETACH_INPUT[..split], b"x"].concat(), false)
        );
        assert_eq!(
            filter_detach_signal(&DETACH_INPUT[..split], &mut matched),
            (Vec::new(), false)
        );
        assert_eq!(
            filter_detach_signal(&DETACH_INPUT[split..], &mut matched),
            (Vec::new(), true)
        );
        assert_eq!(matched, 0);
    }

    #[test]
    fn row_output_is_bounded_and_control_free() {
        let row = AccountCatalogRow {
            session_id: "id".into(),
            run_id: "run".into(),
            project_id: "p".into(),
            kind: "shell".into(),
            policy: "guarded".into(),
            label: Some("bad\u{7}label".into()),
            state: "live".into(),
            created_seq: 1,
            mru_seq: 2,
            observed_ms: 0,
            exit_status: Some(0),
            attachment_generation: 3,
            writer_attached: Some(false),
        };
        let line = row_line(&row);
        assert!(line.contains("bad?label"));
        assert!(!line.chars().any(|c| c.is_control()));
        assert!(line.contains("gen=3"));
        assert!(line.contains("writer=free"));
    }

    #[test]
    fn ended_rows_are_never_resumable() {
        let row = AccountCatalogRow {
            session_id: "id".into(),
            run_id: "run".into(),
            project_id: "p".into(),
            kind: "shell".into(),
            policy: "guarded".into(),
            label: None,
            state: "ended".into(),
            created_seq: 1,
            mru_seq: 1,
            observed_ms: 0,
            exit_status: Some(0),
            attachment_generation: 2,
            writer_attached: None,
        };
        assert!(!is_live(&row));
    }
}
