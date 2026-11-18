//! Private `ctl` dispatcher: list, resume and stop against the account guardian.
//!
//! The CLI never starts a guardian, never launches a vendor process and never
//! interprets remote output as shell text.
use crate::{
    companion::{guardian, Route},
    ssh_remote::{ConnectionError, HostClient},
};
use remote_protocol::proto::*;
use std::time::Duration;

const EXIT_OK: i32 = 0;
const EXIT_DOMAIN: i32 = 1;
const EXIT_SYNTAX: i32 = 2;
const PAGE_LIMIT: u32 = 32;
const HELP: &str = "\
warpai list [--project <path>] [--id <session>]
    List guardian sessions for this account.

warpai resume [<name>] [--id <session>] [--project <path>] [--takeover]
    Attach the current, unique or last-used live session in this helper's scope.
    Ctrl+B Q detaches; the remote process keeps running.

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

/// Account discovery and bare stop do not require a project admission.
async fn connect(
    project: &Option<String>,
) -> Result<(HostClient, String), i32> {
    let active = route();
    let (stream, guardian, boot) = match guardian::probe(active).await {
        Ok(connected) => connected,
        Err(_) => {
            eprintln!("Guardian is unavailable for this account.");
            return Err(EXIT_DOMAIN);
        }
    };
    let mut client = match HostClient::connect_unix(stream, None).await {
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
    let (mut client, filter) = match connect(&project).await {
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
    let (mut client, filter) = match connect(&project).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    if matches!(selection, Selection::All) && project.is_none()
        && client.current_guarded_session().is_some()
    {
        println!("Already in the current guarded session.");
        return EXIT_OK;
    }
    let rows = match rows(&mut client, route_name(route()), &filter).await {
        Ok(rows) => rows,
        Err(code) => return code,
    };
    let selected = match select(&rows, &selection) {
        Ok(row) => row.clone(),
        Err(code) => return code,
    };
    if client.current_guarded_session().is_some_and(|current| {
        current.session_id == selected.session_id && current.run_id == selected.run_id
    }) {
        println!("Already in the current guarded session.");
        return EXIT_OK;
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
    if project.is_none() && client.open_guarded_project(&current).await.is_err() {
        eprintln!("Selected session project is unavailable. Use --project <root> for an older guardian.");
        return EXIT_DOMAIN;
    }
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
    attach_loop(&mut client, &mut state).await
}

/// Direct TTY attachment; output offsets and input receipts stay authoritative.
async fn attach_loop(client: &mut HostClient, state: &mut TerminalState) -> i32 {
    let signals = match TerminationSignals::new() {
        Ok(signals) => signals,
        Err(_) => {
            eprintln!("Could not install terminal cleanup handlers.");
            detach(client, state).await;
            return EXIT_DOMAIN;
        }
    };
    let raw = match RawTerminal::enable(libc::STDIN_FILENO) {
        Ok(raw) => raw,
        Err(_) => {
            eprintln!("Could not configure the local terminal.");
            detach(client, state).await;
            return EXIT_DOMAIN;
        }
    };
    let result = tokio::select! {
        biased;
        signal = signals.recv() => Err(signal),
        result = attach_inner(client, state) => Ok(result),
    };
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

struct TerminationSignals {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
    hangup: tokio::signal::unix::Signal,
    quit: tokio::signal::unix::Signal,
}

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

async fn attach_inner(client: &mut HostClient, state: &mut TerminalState) -> Result<i32, i32> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut buffer = [0u8; 4096];
    let mut offset = 0u64;
    let mut sequence = 0u64;
    let mut dimensions = None;
    let mut prefix = false;
    let mut truncated_notice = false;
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
        stdout.write_all(&reply.output).await.map_err(|_| EXIT_DOMAIN)?;
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
                if count == 0 {
                    // EOF detaches; it never stops the guardian-owned process.
                    return Ok(EXIT_OK);
                }
                let (send, detach_now) = escape_filter(&buffer[..count], &mut prefix);
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
    let (mut client, filter) = match connect(&project).await {
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

struct RawTerminal {
    fd: std::os::fd::RawFd,
    original: Option<libc::termios>,
}

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

impl Drop for RawTerminal {
    fn drop(&mut self) {
        if let Some(original) = &self.original {
            unsafe {
                libc::tcsetattr(self.fd, libc::TCSANOW, original);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mode = || unsafe {
            let mut attributes: libc::termios = std::mem::zeroed();
            assert_eq!(libc::tcgetattr(fd, &mut attributes), 0);
            attributes.c_lflag & (libc::ICANON | libc::ECHO | libc::ISIG | libc::IEXTEN)
        };
        let original = mode();
        let raw = RawTerminal::enable(fd).unwrap();
        assert_eq!(mode(), 0);
        drop(raw);
        assert_eq!(mode(), original);
    }

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
