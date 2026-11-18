//! Private `ctl` dispatcher: list, resume and stop against the account guardian.
//!
//! The CLI never starts a guardian, never launches a vendor process and never
//! interprets remote output as shell text.
use crate::{
    companion::{guardian, Route},
    ssh_remote::{ConnectionError, HostClient},
};
use remote_protocol::proto::*;
use std::{path::PathBuf, time::Duration};

const EXIT_OK: i32 = 0;
const EXIT_DOMAIN: i32 = 1;
const EXIT_SYNTAX: i32 = 2;
const PAGE_LIMIT: u32 = 64;
const HELP: &str = "\
warpai list [--project <path>] [--id <session>]
    List guardian sessions for this account.

warpai resume [<name>] [--id <session>] [--project <path>] [--takeover]
    Attach the unique, current-project or last-used live session.
    Ctrl+B Q detaches; the remote process keeps running.

warpai stop [<name>] [--id <session>] [--project <path>]
    Stop every guarded run in this account, or one exact resolved run.

Exit codes: 0 success, 1 domain failure or unconfirmed stop, 2 invalid syntax
or an ambiguous named target. Bare resume/stop cover the current project; pass
--project to target another project root.";

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

/// Strict parser: no shell evaluation, `--` ends options, duplicates rejected.
pub fn parse(arguments: &[String]) -> Result<Command, String> {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Err("Missing command. Use list, resume or stop.".into());
    };
    let rest = &arguments[1..];
    match command {
        "-h" | "--help" | "help" => Ok(Command::Help),
        "-V" | "--version" | "version" => Ok(Command::Version),
        "list" => parse_list(rest),
        "resume" => parse_resume(rest),
        "stop" => parse_stop(rest),
        other => Err(format!("Unknown command: {other}")),
    }
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
        Command::List { project, id } => list(project, id).await,
        Command::Resume {
            project,
            selection,
            takeover,
        } => resume(project, selection, takeover).await,
        Command::Stop { project, selection } => stop(project, selection).await,
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
    format!(
        "{}  {}  {}  {}  project={}  mru={}  gen={}{}",
        row.session_id,
        label,
        encode_text(&row.kind),
        state,
        encode_text(&row.project_id),
        row.mru_seq,
        row.attachment_generation,
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

/// Connect without ever starting a guardian; `--project` selects the opened
/// root and catalog filter, otherwise the current directory is opened while
/// account-wide listing stays available.
async fn connect(project: &Option<String>) -> Result<(HostClient, String, String, String), i32> {
    let active = route();
    let stream = match guardian::probe(active).await {
        Ok((stream, _, _)) => stream,
        Err(_) => {
            eprintln!("Guardian is not running for this account.");
            return Err(EXIT_DOMAIN);
        }
    };
    let root = match project {
        Some(path) => PathBuf::from(path),
        None => match std::env::current_dir() {
            Ok(directory) => directory,
            Err(_) => {
                eprintln!("Current project directory is unavailable.");
                return Err(EXIT_DOMAIN);
            }
        },
    };
    let Some(root) = root.to_str() else {
        eprintln!("Project path is not valid UTF-8.");
        return Err(EXIT_DOMAIN);
    };
    let client = match HostClient::connect_unix(stream, root).await {
        Ok(client) => client,
        Err(_) => {
            eprintln!("Project is unavailable on the guardian account: {root}");
            return Err(EXIT_DOMAIN);
        }
    };
    if !client.is_account_guardian() {
        eprintln!("Installed Companion does not provide guardian sessions.");
        return Err(EXIT_DOMAIN);
    }
    let current_project = client
        .fence()
        .map(|fence| fence.project_id.clone())
        .unwrap_or_default();
    if current_project.is_empty() {
        eprintln!("Project admission was refused.");
        return Err(EXIT_DOMAIN);
    }
    let filter = match project {
        Some(_) => current_project.clone(),
        None => String::new(),
    };
    Ok((client, current_project, filter, root.to_owned()))
}

async fn rows(client: &mut HostClient, project_id: &str) -> Result<Vec<AccountCatalogRow>, i32> {
    match client
        .account_catalog_list(
            AccountCatalogFilter {
                scope_id: String::new(),
                project_id: project_id.to_owned(),
            },
            PAGE_LIMIT,
            String::new(),
        )
        .await
    {
        Ok(page) => Ok(page.rows),
        Err(_) => {
            eprintln!("Guardian catalog is unavailable.");
            Err(EXIT_DOMAIN)
        }
    }
}

async fn list(project: Option<String>, id: Option<String>) -> i32 {
    let (mut client, _, filter, _) = match connect(&project).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    let rows = match rows(&mut client, &filter).await {
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
                eprintln!("No live session matches {id} in this project.");
                EXIT_DOMAIN
            }),
        Selection::Name(name) => {
            let named: Vec<&AccountCatalogRow> = live
                .into_iter()
                .filter(|row| row.label.as_deref() == Some(name.as_str()))
                .collect();
            match named.len() {
                0 => {
                    eprintln!("No live session is named {name} in this project.");
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
                eprintln!("No live session is resumable in this project.");
                Err(EXIT_DOMAIN)
            }
            1 => Ok(live[0]),
            // Highest daemon record of foreground use; stable ID breaks ties.
            _ => Ok(live
                .into_iter()
                .max_by(|left, right| {
                    left.mru_seq
                        .cmp(&right.mru_seq)
                        .then_with(|| right.session_id.cmp(&left.session_id))
                })
                .expect("Live selection is not empty")),
        },
    }
}

async fn resume(project: Option<String>, selection: Selection, takeover: bool) -> i32 {
    let (mut client, current_project, filter, _) = match connect(&project).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    let rows = match rows(&mut client, &filter).await {
        Ok(rows) => rows,
        Err(code) => return code,
    };
    let selected = match select(&rows, &selection) {
        Ok(row) => row.clone(),
        Err(code) => return code,
    };
    if selected.project_id != current_project {
        eprintln!(
            "Session {} belongs to another project; pass --project <root>.",
            selected.session_id
        );
        return EXIT_DOMAIN;
    }
    let current = match client
        .account_catalog_inspect(selected.session_id.clone(), selected.run_id.clone())
        .await
    {
        Ok(row) if is_live(&row) => row,
        _ => {
            eprintln!("Selected session is no longer live.");
            return EXIT_DOMAIN;
        }
    };
    let dimensions = terminal_size();
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
        columns: dimensions.0,
        rows: dimensions.1,
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
    attach_loop(&mut client, &mut state).await
}

/// Direct TTY attachment; output offsets and input receipts stay authoritative.
async fn attach_loop(client: &mut HostClient, state: &mut TerminalState) -> i32 {
    let raw = RawTerminal::enable();
    let result = attach_inner(client, state).await;
    drop(raw);
    result.unwrap_or_else(|code| code)
}

async fn attach_inner(client: &mut HostClient, state: &mut TerminalState) -> Result<i32, i32> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut buffer = [0u8; 4096];
    let mut offset = 0u64;
    let mut sequence = 0u64;
    let mut input_open = true;
    let mut dimensions = terminal_size();
    let mut prefix = false;
    let mut truncated_notice = false;
    loop {
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
            let _ = stdout
                .write_all(b"\r\n[earlier output is no longer retained]\r\n")
                .await;
        }
        let _ = stdout.write_all(&reply.output).await;
        let _ = stdout.flush().await;
        offset = reply.output_offset + reply.output.len() as u64;
        *state = reply;
        if state.processes_active == Some(false)
            && state.output_closed
            && offset == state.output_end
        {
            detach(client, state).await;
            return Ok(state.exit_code.unwrap_or(EXIT_OK).clamp(0, 255));
        }
        if offset < state.output_end {
            continue;
        }
        let resized = terminal_size();
        if resized != dimensions {
            let resize = TerminalControl {
                fence: client.fence().cloned(),
                session_id: state.session_id.clone(),
                run_id: state.run_id.clone(),
                action: TerminalAction::TerminalResize as i32,
                columns: resized.0,
                rows: resized.1,
                attachment_id: state.attachment_id.clone(),
                attachment_generation: state.attachment_generation,
                ..Default::default()
            };
            dimensions = resized;
            if client.terminal_control(resize).await.is_err() {
                eprintln!("\r\nSession resize was rejected.");
                return Err(EXIT_DOMAIN);
            }
        }
        tokio::select! {
            read = stdin.read(&mut buffer), if input_open => {
                let count = match read {
                    Ok(count) => count,
                    Err(_) => 0,
                };
                if count == 0 {
                    // EOF detaches; it never stops the guardian-owned process.
                    detach(client, state).await;
                    return Ok(EXIT_OK);
                }
                let mut send = Vec::with_capacity(count);
                let (filtered, detach_now) = escape_filter(&buffer[..count], &mut prefix);
                send.extend(filtered);
                if detach_now {
                    detach(client, state).await;
                    return Ok(EXIT_OK);
                }
                if send.is_empty() {
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
                        detach(client, state).await;
                        return Err(EXIT_DOMAIN);
                    }
                    Err(_) => {
                        eprintln!("\r\nSession input was rejected; detaching.");
                        detach(client, state).await;
                        return Err(EXIT_DOMAIN);
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(20)) => ()
        }
    }
}

/// Local detach escape: `Ctrl+B Q` detaches; a doubled prefix forwards one
/// literal `0x02`. The prefix may arrive split across reads.
fn escape_filter(bytes: &[u8], prefix: &mut bool) -> (Vec<u8>, bool) {
    let mut send = Vec::with_capacity(bytes.len());
    for byte in bytes {
        if *prefix {
            *prefix = false;
            if *byte == b'q' {
                return (send, true);
            }
            send.push(*byte);
        } else if *byte == 0x02 {
            *prefix = true;
        } else {
            send.push(*byte);
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
    let (mut client, _, filter, _) = match connect(&project).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    let target = match selection {
        Selection::All => None,
        selection => {
            let rows = match rows(&mut client, &filter).await {
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

struct RawTerminal(Option<libc::termios>);

impl RawTerminal {
    fn enable() -> Self {
        use std::io::IsTerminal;
        if !std::io::stdin().is_terminal() {
            return Self(None);
        }
        unsafe {
            let mut original: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(0, &mut original) != 0 {
                return Self(None);
            }
            let mut raw = original;
            libc::cfmakeraw(&mut raw);
            if libc::tcsetattr(0, libc::TCSANOW, &raw) != 0 {
                return Self(None);
            }
            Self(Some(original))
        }
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        if let Some(original) = &self.0 {
            unsafe {
                libc::tcsetattr(0, libc::TCSANOW, original);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(value: &[&str]) -> Vec<String> {
        value.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn parser_accepts_only_the_three_functional_commands() {
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
        assert_eq!(
            select(&[row("z", None, "ended", 3)], &Selection::All).unwrap_err(),
            EXIT_DOMAIN
        );
    }

    #[test]
    fn detach_escape_survives_split_reads_and_doubled_prefix() {
        let mut prefix = false;
        let (send, detach) = escape_filter(b"ab", &mut prefix);
        assert_eq!(send, b"ab");
        assert!(!detach && !prefix);
        let (send, detach) = escape_filter(&[0x02], &mut prefix);
        assert!(send.is_empty() && !detach && prefix);
        let (send, detach) = escape_filter(b"x", &mut prefix);
        assert_eq!(send, b"x");
        assert!(!detach && !prefix);
        let (send, detach) = escape_filter(&[0x02, 0x02], &mut prefix);
        assert_eq!(send, [0x02]);
        assert!(!detach && !prefix);
        let (send, detach) = escape_filter(&[0x02, b'q'], &mut prefix);
        assert!(send.is_empty() && detach && !prefix);
        let (send, detach) = escape_filter(&[0x02, b'Q'], &mut prefix);
        assert_eq!(send, b"Q");
        assert!(!detach && !prefix);
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
        };
        let line = row_line(&row);
        assert!(line.contains("bad?label"));
        assert!(!line.chars().any(|c| c.is_control()));
        assert!(line.contains("gen=3"));
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
        };
        assert!(!is_live(&row));
    }
}
