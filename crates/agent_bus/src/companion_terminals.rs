//! Connection-owned Agent PTYs; disconnect stops the owned process group/job.
use super::*;
use crate::native_pty::{NativePty, Size};
use std::{
    collections::{BTreeMap, VecDeque},
    io::{Read, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
};

const MAX_SESSIONS: usize = 32;
const BUFFER_BYTES: usize = 256 * 1024;
const PAGE_BYTES: usize = 32 * 1024;
type Error = ManagedErrorCode;
type Owner = Arc<Mutex<Session>>;

enum NativeInput {
    User(Vec<u8>),
    Peer { wake: crate::transport::Wake, bytes: Vec<u8>, submit: bool },
}

#[derive(Default)]
pub(super) struct Terminals(Mutex<BTreeMap<String, Owner>>);
#[derive(Default)]
struct Output {
    bytes: VecDeque<u8>,
    end: u64,
    closed: bool,
}
impl Output {
    fn append(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.end = self
            .end
            .checked_add(bytes.len() as u64)
            .ok_or(Error::ManagedUnavailable)?;
        self.bytes.extend(bytes);
        let discard = self.bytes.len().saturating_sub(BUFFER_BYTES);
        self.bytes.drain(..discard);
        Ok(())
    }
}
struct Session {
    launch: TerminalLaunch,
    run_id: String,
    binding: Option<tasks::RunBinding>,
    pty: NativePty,
    output: Arc<Mutex<Output>>,
    screen: Option<Arc<Mutex<crate::screen_detection::native::NativeScreen>>>,
    input: mpsc::SyncSender<NativeInput>,
    connection: Option<String>,
    sequence: u64,
    failed: Arc<AtomicBool>,
    revoked: Arc<AtomicBool>,
    stop_requested: bool,
}

fn lock<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, Error> {
    mutex.lock().map_err(|_| Error::ManagedUnavailable)
}
fn dimensions(columns: u32, rows: u32) -> Result<Size, Error> {
    if !(1..=1000).contains(&columns) || !(1..=1000).contains(&rows) {
        return Err(Error::ManagedInvalidInput);
    }
    Ok(Size {
        columns: columns as u16,
        rows: rows as u16,
    })
}
fn valid_id(id: &str) -> bool {
    Uuid::parse_str(id).is_ok_and(|id| !id.is_nil())
}

impl Terminals {
    pub(super) fn launch(
        &self,
        request: TerminalLaunch,
        fence: &ManagedFence,
        root: &Path,
        tasks: &tasks::Projects,
    ) -> Result<TerminalState, Error> {
        if !valid_id(&request.session_id) {
            return Err(Error::ManagedInvalidInput);
        }
        if request.agent_program.as_ref().is_some_and(|program| {
            program.is_empty()
                || program.len() > 64
                || program == crate::OPERATOR_PROGRAM
                || !program
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
        }) {
            return Err(Error::ManagedInvalidInput);
        }
        let size = dimensions(request.columns, request.rows)?;
        let reconcile = |sessions: &BTreeMap<String, Owner>| -> Result<Option<TerminalState>, Error> {
            if let Some(owner) = sessions.get(&request.session_id) {
                let mut session = lock(owner)?;
                let original = &session.launch;
                if original.fence.as_ref().map(|f| &f.project_id) != Some(&fence.project_id)
                    || original.executable != request.executable
                    || original.arguments != request.arguments
                    || original.columns != request.columns
                    || original.rows != request.rows
                    || original.agent_program != request.agent_program
                    || original.working_directory != request.working_directory
                {
                    return Err(Error::ManagedConflict);
                }
                if session.connection.as_deref() != Some(&fence.connection_id) {
                    return Err(Error::ManagedStaleAttachment);
                }
                // Reconcile only the original connection; disconnected runs cannot be adopted.
                return session.state(fence, None).map(Some);
            }
            if sessions.len() >= MAX_SESSIONS {
                return Err(Error::ManagedCapacityExceeded);
            }
            Ok(None)
        };
        {
            let sessions = lock(&self.0)?;
            if let Some(state) = reconcile(&sessions)? { return Ok(state); }
        }
        #[cfg(not(feature = "wsl_companion"))]
        if request.working_directory.is_some() { return Err(Error::ManagedFeatureUnavailable); }
        #[cfg(not(feature = "wsl_companion"))]
        let working_directory = root.to_owned();
        #[cfg(feature = "wsl_companion")]
        let working_directory = match request.working_directory.as_deref() {
            Some(directory) if directory.len() <= 4096 && !directory.contains('\0')
                && Path::new(directory).is_absolute() => {
                let directory = Path::new(directory).canonicalize().map_err(path_error)?;
                if !directory.is_dir() { return Err(Error::ManagedInvalidInput); }
                directory
            }
            Some(_) => return Err(Error::ManagedInvalidInput),
            None => root.to_owned(),
        };
        if let Some(program) = &request.agent_program {
            let executable = Path::new(&request.executable);
            if !executable.is_absolute() || request.executable.len() > 4096
                || request.executable.contains('\0') || request.arguments.len() > 61 {
                return Err(Error::ManagedInvalidInput);
            }
            tasks.prepare_agent(program, executable)?;
        }
        let mut sessions = lock(&self.0)?;
        if let Some(state) = reconcile(&sessions)? { return Ok(state); }
        let run_id = Uuid::new_v4().to_string();
        let binding = request
            .agent_program
            .as_deref()
            .map(|program| tasks.bind_run(fence, root, &request.session_id, &run_id, program))
            .transpose()?;
        let (executable, arguments) = if let Some(program) = &request.agent_program {
            let vendor = Path::new(&request.executable);
            if !vendor.is_absolute()
                || request.executable.len() > 4096
                || request.executable.contains('\0')
                || request.arguments.len() > 61
            {
                return Err(Error::ManagedInvalidInput);
            }
            let executable = std::env::current_exe().map_err(path_error)?;
            let mut arguments = vec![
                "--bound-agent".into(),
                program.clone(),
                request.executable.clone(),
            ];
            arguments.extend(request.arguments.iter().cloned());
            (executable, arguments)
        } else {
            (
                PathBuf::from(&request.executable),
                request.arguments.clone(),
            )
        };
        let pty = NativePty::spawn_with_environment(
            &executable,
            &arguments,
            &working_directory,
            size,
            binding
                .as_ref()
                .map_or(&[], |binding| binding.environment.as_slice()),
        )
        .map_err(path_error)?;
        let mut reader = pty.reader.try_clone().map_err(path_error)?;
        let mut writer = pty.writer.try_clone().map_err(path_error)?;
        let output = Arc::new(Mutex::new(Output::default()));
        let screen = binding.as_ref().map(|_| Arc::new(Mutex::new(
            crate::screen_detection::native::NativeScreen::new(size.rows, size.columns))));
        let revoked = Arc::new(AtomicBool::new(false));
        let (input, receiver) = mpsc::sync_channel::<NativeInput>(16);
        let failed = Arc::new(AtomicBool::new(false));
        let collected = output.clone();
        let captured = screen.clone();
        let observed = binding.as_ref().map(|binding| (binding.broker.clone(), binding.terminal.clone()));
        std::thread::spawn(move || {
            let mut buffer = [0; 8192];
            loop {
                let count = match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(count) => count,
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                };
                if let Some((broker, terminal)) = &observed { broker.output(terminal); }
                {
                    let Ok(mut output) = collected.lock() else { return; };
                    if output.append(&buffer[..count]).is_err() { break; }
                }
                if let Some(screen) = &captured {
                    if let Ok(mut screen) = screen.lock() { screen.process(&buffer[..count]); }
                }
                if let Some((broker, terminal)) = &observed { broker.output(terminal); }
            }
            if let Ok(mut output) = collected.lock() {
                output.closed = true;
            }
        });
        if let (Some(binding), Some(screen)) = (&binding, &screen) {
            let broker = binding.broker.clone();
            let terminal = binding.terminal.clone();
            let screen = screen.clone();
            let stopped = revoked.clone();
            let input = input.clone();
            std::thread::spawn(move || {
                let mut sampled: Option<crate::transport::ScreenContext> = None;
                let mut delivery: Option<(crate::transport::Wake, bool)> = None;
                while !stopped.load(Ordering::Acquire) {
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    let Some(context) = broker.screen_context(&terminal) else { break; };
                    if !sampled.as_ref().is_some_and(|previous| context.same_screen(previous)) {
                        let activity = screen.lock().ok().and_then(|screen| screen.activity(&context));
                        if let Some(activity) = activity {
                            if broker.observe_screen(&terminal, &context, activity) { sampled = Some(context); }
                        }
                    }
                    if let Some((wake, paste)) = delivery.take() {
                        if !broker.wake_valid(&wake) { broker.finish_wake(&wake, false); continue; }
                        let bytes = if paste {
                            let text = format!("Warpai peer work is waiting (message {}). Call warp_agent_inbox, process this message or task, and acknowledge it through MCP. Before ending your turn, call warp_agent_ready.", wake.message_id);
                            if screen.lock().is_ok_and(|screen| screen.bracketed_paste()) {
                                [b"\x1b[200~".as_slice(), text.as_bytes(), b"\x1b[201~".as_slice()].concat()
                            } else { text.into_bytes() }
                        } else { b"\r".to_vec() };
                        if input.try_send(NativeInput::Peer { wake: wake.clone(), bytes, submit: !paste }).is_err() {
                            broker.finish_wake(&wake, false);
                        } else if paste { delivery = Some((wake, false)); }
                        continue;
                    }
                    if let Some(wake) = broker.wakeups().into_iter().find(|wake| wake.terminal == terminal) {
                        if Uuid::parse_str(&wake.message_id).is_err() || !broker.claim_wake(&wake) { continue; }
                        if broker.discard_native_draft(&wake)
                            && input.try_send(NativeInput::Peer { wake: wake.clone(), bytes: b"\x03".to_vec(), submit: false }).is_err() {
                            broker.finish_wake(&wake, false);
                        } else { delivery = Some((wake, true)); }
                    }
                }
            });
        }
        let write_revoked = revoked.clone();
        let write_failed = failed.clone();
        let write_binding = binding.as_ref().map(|binding| binding.broker.clone());
        std::thread::spawn(move || {
            while let Ok(input) = receiver.recv() {
                if write_revoked.load(Ordering::Acquire) || write_failed.load(Ordering::Acquire) {
                    continue;
                }
                let (bytes, delivery) = match input {
                    NativeInput::User(bytes) => (bytes, None),
                    NativeInput::Peer { wake, bytes, submit } => {
                        let Some(broker) = &write_binding else { continue; };
                        if !broker.wake_valid(&wake) { broker.finish_wake(&wake, false); continue; }
                        (bytes, Some((wake, submit)))
                    }
                };
                let written = writer.write_all(&bytes).and_then(|_| writer.flush()).is_ok();
                if let (Some(broker), Some((wake, submit))) = (&write_binding, delivery) {
                    if submit || !written { broker.finish_wake(&wake, written && submit); }
                }
                if !written {
                    write_failed.store(true, Ordering::Release);
                }
            }
        });
        let mut session = Session {
            launch: request,
            run_id,
            binding,
            pty,
            output,
            screen,
            input,
            connection: Some(fence.connection_id.clone()),
            sequence: 0,
            failed,
            revoked,
            stop_requested: false,
        };
        let state = session.state(fence, None)?;
        sessions.insert(
            session.launch.session_id.clone(),
            Arc::new(Mutex::new(session)),
        );
        Ok(state)
    }

    pub(super) fn control(
        &self,
        request: TerminalControl,
        fence: &ManagedFence,
    ) -> Result<TerminalState, Error> {
        let owner = lock(&self.0)?
            .get(&request.session_id)
            .cloned()
            .ok_or(Error::ManagedStaleAttachment)?;
        let mut session = lock(&owner)?;
        if session.launch.fence.as_ref().map(|f| &f.project_id) != Some(&fence.project_id)
            || request.run_id != session.run_id
        {
            return Err(Error::ManagedStaleAttachment);
        }
        let action =
            TerminalAction::try_from(request.action).map_err(|_| Error::ManagedInvalidInput)?;
        if action != TerminalAction::TerminalInput
            && (!request.input.is_empty() || request.input_sequence != 0)
            || action != TerminalAction::TerminalResize
                && (request.columns != 0 || request.rows != 0)
            || action != TerminalAction::TerminalRead && request.output_offset != 0
        {
            return Err(Error::ManagedInvalidInput);
        }
        if session.connection.as_deref() != Some(&fence.connection_id) {
            return Err(Error::ManagedStaleAttachment);
        }
        match action {
            TerminalAction::TerminalInput => {
                if request.input.is_empty()
                    || request.input.len() > 4096
                    || session.sequence.checked_add(1) != Some(request.input_sequence)
                {
                    return Err(Error::ManagedInvalidInput);
                }
                if session.failed.load(Ordering::Acquire)
                    || session.pty.exit_code().map_err(path_error)?.is_some()
                {
                    return Err(Error::ManagedUnavailable);
                }
                if let Some(binding) = &session.binding { binding.broker.input_bytes(&binding.terminal, &request.input); }
                session
                    .input
                    .try_send(NativeInput::User(request.input))
                    .map_err(|_| Error::ManagedCapacityExceeded)?;
                session.sequence = request.input_sequence;
            }
            TerminalAction::TerminalResize => {
                let size = dimensions(request.columns, request.rows)?;
                session
                    .pty
                    .resize(size)
                    .map_err(path_error)?;
                if let Some(screen) = &session.screen { lock(screen)?.resize(size.rows, size.columns); }
                if let Some(binding) = &session.binding { binding.broker.output(&binding.terminal); }
            }
            TerminalAction::TerminalStop => {
                // Process stop never waits for a blocked native input writer.
                session.pty.stop().map_err(path_error)?;
                session.stop_requested = true;
            }
            TerminalAction::TerminalRead => (),
            TerminalAction::TerminalRelease => {
                if session.pty.exit_code().map_err(path_error)?.is_none()
                    || session.pty.is_active().map_err(path_error)?
                    || !lock(&session.output)?.closed
                {
                    return Err(Error::ManagedConflict);
                }
                let state = session.state(fence, None)?;
                drop(session);
                let mut sessions = lock(&self.0)?;
                if sessions
                    .get(&request.session_id)
                    .is_none_or(|current| !Arc::ptr_eq(current, &owner))
                {
                    return Err(Error::ManagedStaleAttachment);
                }
                sessions.remove(&request.session_id);
                return Ok(state);
            }
            _ => return Err(Error::ManagedInvalidInput),
        }
        session.state(
            fence,
            (action == TerminalAction::TerminalRead).then_some(request.output_offset),
        )
    }

    pub(super) fn disconnect(&self, connection: &str) {
        let Ok(owners) = self
            .0
            .lock()
            .map(|m| m.values().cloned().collect::<Vec<_>>())
        else {
            return;
        };
        for owner in owners {
            if let Ok(mut session) = owner.lock() {
                if session.connection.as_deref() != Some(connection) {
                    continue;
                }
                session.connection = None;
                session.revoked.store(true, Ordering::Release);
                session.binding.take();
                // Keep native ownership on stop failure; idle exit must still observe it.
                session.stop_requested = session.pty.stop().is_ok();
            }
        }
    }

    pub(super) fn active(&self) -> bool {
        let Ok(mut owners) = self.0.lock() else {
            return true;
        };
        let mut active = false;
        owners.retain(|_, owner| {
            let Ok(mut session) = owner.lock() else {
                active = true;
                return true;
            };
            let owned = session.pty.is_active().unwrap_or(true);
            if !owned {
                session.binding.take();
            }
            let draining = session.output.lock().map_or(true, |output| !output.closed);
            active |= owned || draining;
            session.connection.is_some() || owned || draining
        });
        active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_buffer_preserves_absolute_offsets_when_full() {
        let mut output = Output::default();
        output.append(&vec![1; BUFFER_BYTES]).unwrap();
        output.append(b"tail").unwrap();
        assert_eq!(output.end, BUFFER_BYTES as u64 + 4);
        assert_eq!(output.bytes.len(), BUFFER_BYTES);
        assert_eq!(
            output
                .bytes
                .iter()
                .rev()
                .take(4)
                .copied()
                .collect::<Vec<_>>(),
            b"liat"
        );
        output.end = u64::MAX;
        assert!(output.append(b"overflow").is_err());
        assert_eq!(output.bytes.len(), BUFFER_BYTES);
    }
}

impl Session {
    fn state(&mut self, fence: &ManagedFence, offset: Option<u64>) -> Result<TerminalState, Error> {
        let processes_active = Some(self.pty.is_active().map_err(path_error)?);
        let exit_code = self.pty.exit_code().map_err(path_error)?;
        if processes_active == Some(false) {
            self.binding.take();
        }
        let output = lock(&self.output)?;
        let first = output.end - output.bytes.len() as u64;
        let requested = offset.unwrap_or(output.end);
        if requested > output.end {
            return Err(Error::ManagedInvalidInput);
        }
        let start = requested.max(first);
        Ok(TerminalState {
            fence: Some(fence.clone()),
            session_id: self.launch.session_id.clone(),
            run_id: self.run_id.clone(),
            stop_requested: self.stop_requested,
            exit_code,
            processes_active,
            output_offset: start,
            output_end: output.end,
            output: if offset.is_some() {
                output
                    .bytes
                    .iter()
                    .skip((start - first) as usize)
                    .take(PAGE_BYTES)
                    .copied()
                    .collect()
            } else {
                Vec::new()
            },
            output_truncated: requested < first,
            accepted_input_sequence: self.sequence,
            input_failed: self.failed.load(Ordering::Acquire),
            output_closed: output.closed,
        })
    }
}
