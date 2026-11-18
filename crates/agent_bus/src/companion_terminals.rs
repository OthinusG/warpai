//! Managed service-owned PTYs; guarded runs survive connection loss, legacy runs do not.
use super::*;
use crate::native_pty::{NativePty, Size};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, VecDeque},
    io::{Read, Write},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc, Arc, Mutex, TryLockError,
    },
    time::{Duration, Instant},
};

const MAX_SESSIONS: usize = 32;
const BUFFER_BYTES: usize = 256 * 1024;
const PAGE_BYTES: usize = 32 * 1024;
const MAX_LABEL: usize = 128;
const MAX_INPUT_BYTES: usize = 4096;
const MAX_INPUT_RECEIPTS: usize = 64;
const MAX_PENDING_INPUT_BYTES: usize = 64 * 1024;
const ENDED_RETENTION: Duration = Duration::from_secs(5 * 60);
type Error = ManagedErrorCode;
type Owner = Arc<Mutex<Session>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Policy {
    ConnectionOwned,
    Guarded,
}

enum NativeInput {
    /// Human input carries the attachment generation, epoch and accepted sequence.
    User {
        attachment: u64,
        epoch: u64,
        sequence: u64,
        bytes: Vec<u8>,
    },
    Peer { wake: crate::transport::Wake, bytes: Vec<u8>, submit: bool },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum InputState {
    Accepted,
    Writing,
    Written,
    Cancelled,
    Failed,
    Unknown,
}

impl InputState {
    fn pending(self) -> bool {
        matches!(self, Self::Accepted | Self::Writing)
    }
    fn terminal(self) -> bool {
        // Unknown delivery is unresolved evidence, not safely evictable history.
        matches!(self, Self::Written | Self::Cancelled | Self::Failed)
    }
    fn disposition(self) -> InputDisposition {
        match self {
            Self::Accepted => InputDisposition::InputAccepted,
            Self::Writing => InputDisposition::InputWriting,
            Self::Written => InputDisposition::InputWritten,
            Self::Cancelled => InputDisposition::InputCancelled,
            Self::Failed => InputDisposition::InputFailed,
            Self::Unknown => InputDisposition::InputUnknown,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct InputRecord {
    attachment: u64,
    sequence: u64,
    id: String,
    digest: [u8; 32],
    len: usize,
    written: usize,
    state: InputState,
}

impl InputRecord {
    fn proto(&self) -> InputReceipt {
        InputReceipt {
            attachment_generation: self.attachment,
            input_sequence: self.sequence,
            receipt_id: self.id.clone(),
            disposition: self.state.disposition() as i32,
            written_bytes: self.written as u64,
        }
    }
}

/// Bounded per-attachment ledger. Only pending entries count against the byte budget.
#[derive(Default)]
struct InputLedger {
    entries: VecDeque<InputRecord>,
    pending_bytes: usize,
}

impl InputLedger {
    fn find(&self, attachment: u64, sequence: u64) -> Option<&InputRecord> {
        self.entries
            .iter()
            .find(|entry| entry.attachment == attachment && entry.sequence == sequence)
    }
    fn reserve(
        &mut self,
        attachment: u64,
        sequence: u64,
        id: String,
        digest: [u8; 32],
        len: usize,
    ) -> Result<InputRecord, Error> {
        if sequence == 0 || len == 0 || len > MAX_INPUT_BYTES {
            return Err(Error::ManagedInvalidInput);
        }
        if self.find(attachment, sequence).is_some() {
            return Err(Error::ManagedInputConflict);
        }
        while self.entries.len() >= MAX_INPUT_RECEIPTS {
            if !self
                .entries
                .front()
                .is_some_and(|entry| entry.state.terminal())
            {
                return Err(Error::ManagedCapacityExceeded);
            }
            self.entries.pop_front();
        }
        if self
            .pending_bytes
            .checked_add(len)
            .is_none_or(|total| total > MAX_PENDING_INPUT_BYTES)
        {
            return Err(Error::ManagedCapacityExceeded);
        }
        let record = InputRecord {
            attachment,
            sequence,
            id,
            digest,
            len,
            written: 0,
            state: InputState::Accepted,
        };
        self.pending_bytes += len;
        self.entries.push_back(record.clone());
        Ok(record)
    }
    fn remove(&mut self, attachment: u64, sequence: u64) {
        if let Some(index) = self.entries.iter().position(|entry| {
            entry.attachment == attachment && entry.sequence == sequence
        }) {
            if self.entries[index].state.pending() {
                self.pending_bytes = self.pending_bytes.saturating_sub(self.entries[index].len);
            }
            self.entries.remove(index);
        }
    }
    fn mark_writing(&mut self, attachment: u64, sequence: u64) -> bool {
        let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.attachment == attachment && entry.sequence == sequence)
        else {
            return false;
        };
        if entry.state == InputState::Accepted {
            entry.state = InputState::Writing;
        }
        entry.state == InputState::Writing
    }
    fn cancel(&mut self, attachment: u64, sequence: u64) {
        let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.attachment == attachment && entry.sequence == sequence)
        else {
            return;
        };
        let was_pending = entry.state.pending();
        entry.state = match entry.state {
            InputState::Accepted => InputState::Cancelled,
            InputState::Writing => InputState::Unknown,
            state => state,
        };
        if was_pending {
            self.pending_bytes = self.pending_bytes.saturating_sub(entry.len);
        }
    }
    fn cancel_pending(&mut self) {
        for entry in &mut self.entries {
            entry.state = match entry.state {
                InputState::Accepted => InputState::Cancelled,
                InputState::Writing => InputState::Unknown,
                state => state,
            };
        }
        self.pending_bytes = 0;
    }
    fn fail_pending(&mut self) {
        for entry in &mut self.entries {
            if entry.state.pending() {
                entry.state = InputState::Unknown;
            }
        }
        self.pending_bytes = 0;
    }
    fn finish(&mut self, attachment: u64, sequence: u64, written: usize, state: InputState) {
        let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.attachment == attachment && entry.sequence == sequence)
        else {
            return;
        };
        let was_pending = entry.state.pending();
        entry.written = written.min(entry.len);
        entry.state = state;
        if was_pending {
            self.pending_bytes = self.pending_bytes.saturating_sub(entry.len);
        }
    }
    fn clear(&mut self) {
        self.entries.clear();
        self.pending_bytes = 0;
    }
}

fn input_digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// Report how many bytes reached the PTY even when a later write or flush fails.
/// Cancellation is checked between writes; an in-flight syscall still completes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteEnd {
    Complete,
    Cancelled,
}

fn write_counted(
    writer: &mut impl Write,
    bytes: &[u8],
    mut cancelled: impl FnMut() -> bool,
) -> (usize, std::io::Result<WriteEnd>) {
    let mut written = 0;
    while written < bytes.len() {
        if cancelled() {
            return (written, Ok(WriteEnd::Cancelled));
        }
        match writer.write(&bytes[written..]) {
            Ok(0) => {
                return (
                    written,
                    Err(std::io::Error::from(std::io::ErrorKind::WriteZero)),
                )
            }
            Ok(count) => written += count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return (written, Err(error)),
        }
    }
    (written, Ok(WriteEnd::Complete))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RotateOutcome {
    Rotated,
    Busy,
    Unavailable,
}

/// Orders writer replacement against native input. A superseded attachment must not
/// start a write after takeover, and a takeover must not rotate over an in-flight write.
struct WriteFence {
    attachment: AtomicU64,
    epoch: AtomicU64,
    gate: Mutex<()>,
}
impl Default for WriteFence {
    fn default() -> Self {
        Self {
            attachment: AtomicU64::new(1),
            epoch: AtomicU64::new(1),
            gate: Mutex::new(()),
        }
    }
}
impl WriteFence {
    fn rotate(&self, attachment: u64) -> RotateOutcome {
        let _gate = match self.gate.try_lock() {
            Ok(gate) => gate,
            Err(TryLockError::WouldBlock) => return RotateOutcome::Busy,
            Err(TryLockError::Poisoned(_)) => return RotateOutcome::Unavailable,
        };
        if self
            .epoch
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |epoch| epoch.checked_add(1))
            .is_err()
        {
            return RotateOutcome::Unavailable;
        }
        self.attachment.store(attachment, Ordering::SeqCst);
        RotateOutcome::Rotated
    }
    /// Detach/transport loss invalidates queued input without waiting for a blocked write.
    fn cancel_input(&self) {
        // Wrapping is safe here: queued input carries the old epoch and must never match.
        self.epoch.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Default)]
pub(super) struct Terminals {
    owners: Mutex<BTreeMap<String, Owner>>,
}
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
    policy: Policy,
    label: Option<String>,
    run_id: String,
    binding: Option<tasks::RunBinding>,
    pty: NativePty,
    output: Arc<Mutex<Output>>,
    screen: Option<Arc<Mutex<crate::screen_detection::native::NativeScreen>>>,
    input: mpsc::SyncSender<NativeInput>,
    connection: Option<String>,
    attachment_id: String,
    writer: Arc<WriteFence>,
    ledger: Arc<Mutex<InputLedger>>,
    sequence: u64,
    failed: Arc<AtomicBool>,
    revoked: Arc<AtomicBool>,
    stop_requested: bool,
    ended_at: Option<Instant>,
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
fn valid_label(label: Option<&str>) -> bool {
    label.is_none_or(|label| {
        !label.is_empty()
            && label.len() <= MAX_LABEL
            && !label.chars().any(char::is_control)
    })
}

/// True resumes the same writer; false acquires a new attachment after a fenced takeover.
fn attach_decision(
    policy: Policy,
    held: u64,
    held_attachment: &str,
    presented: u64,
    presented_attachment: &str,
    takeover: bool,
    same_connection: bool,
    occupied: bool,
) -> Result<bool, Error> {
    if policy != Policy::Guarded {
        return Err(Error::ManagedFeatureUnavailable);
    }
    let exact = presented == held
        && !presented_attachment.is_empty()
        && presented_attachment == held_attachment;
    if exact && (same_connection || !occupied) {
        return Ok(true);
    }
    if takeover {
        // Takeover must fence the current epoch; a fresh or stale observation cannot acquire.
        if presented == 0 {
            return Err(Error::ManagedConflict);
        }
        if presented != held {
            return Err(Error::ManagedStaleAttachment);
        }
        if !presented_attachment.is_empty() && presented_attachment != held_attachment {
            return Err(Error::ManagedStaleAttachment);
        }
        return Ok(false);
    }
    if exact && occupied {
        return Err(Error::ManagedConflict);
    }
    Err(if presented == 0 {
        Error::ManagedConflict
    } else {
        Error::ManagedStaleAttachment
    })
}

/// Legacy connections use only their admission; guarded controls must carry the exact writer tuple.
fn holds_attachment(
    session: &Session,
    fence: &ManagedFence,
    request: &TerminalControl,
) -> bool {
    session.connection.as_deref() == Some(fence.connection_id.as_str())
        && !request.attachment_id.is_empty()
        && request.attachment_id == session.attachment_id
        && request.attachment_generation == session.writer.attachment.load(Ordering::SeqCst)
}

fn may_control(session: &Session, fence: &ManagedFence, request: &TerminalControl) -> bool {
    if session.connection.as_deref() != Some(fence.connection_id.as_str()) {
        return false;
    }
    match session.policy {
        Policy::ConnectionOwned => true,
        Policy::Guarded => holds_attachment(session, fence, request),
    }
}

/// Ended output stays briefly for an exact resume, within the admission ceiling.
fn retain_session(session: &mut Session, now: Instant) -> bool {
    let owned = session.pty.is_active().unwrap_or(true);
    if !owned {
        session.binding.take();
    }
    let closed = session.output.lock().map_or(false, |output| output.closed);
    if !owned && closed && session.connection.is_none() {
        if session.policy != Policy::Guarded {
            return false;
        }
        match session.ended_at {
            None => session.ended_at = Some(now),
            Some(ended) if now.saturating_duration_since(ended) >= ENDED_RETENTION => return false,
            Some(_) => {}
        }
    } else {
        session.ended_at = None;
    }
    session.connection.is_some() || owned || !closed || session.ended_at.is_some()
}

fn ended_session(session: &mut Session) -> Option<Instant> {
    if session.policy != Policy::Guarded {
        return None;
    }
    let ended = session.ended_at?;
    (session.connection.is_none()
        && !session.pty.is_active().unwrap_or(true)
        && session.output.lock().map_or(false, |output| output.closed))
    .then_some(ended)
}

/// Expire retention normally; at admission pressure, evict oldest ended output first.
fn prune_sessions(sessions: &mut BTreeMap<String, Owner>, now: Instant, evict_ended: bool) {
    sessions.retain(|_, owner| {
        let Ok(mut session) = owner.lock() else {
            return true;
        };
        retain_session(&mut session, now)
    });
    if !evict_ended || sessions.len() < MAX_SESSIONS {
        return;
    }
    let mut ended: Vec<_> = sessions
        .iter()
        .filter_map(|(id, owner)| {
            let mut session = owner.lock().ok()?;
            ended_session(&mut session).map(|ended| (ended, id.clone()))
        })
        .collect();
    ended.sort_by_key(|(ended, _)| *ended);
    for (_, id) in ended {
        if sessions.len() < MAX_SESSIONS {
            break;
        }
        sessions.remove(&id);
    }
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
        if !valid_label(request.label.as_deref())
            || (request.label.is_some() && !request.guarded)
        {
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
                    || original.guarded != request.guarded
                    || original.label != request.label
                {
                    return Err(Error::ManagedConflict);
                }
                if session.connection.as_deref() != Some(&fence.connection_id) {
                    return Err(Error::ManagedStaleAttachment);
                }
                // Relaunch reconciles only the original connection; adoption goes through attach.
                return session.state(fence, None, None).map(Some);
            }
            if sessions.len() >= MAX_SESSIONS {
                return Err(Error::ManagedCapacityExceeded);
            }
            Ok(None)
        };
        {
            let mut sessions = lock(&self.owners)?;
            let evict = sessions.len() >= MAX_SESSIONS;
            prune_sessions(&mut sessions, Instant::now(), evict);
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
        let mut sessions = lock(&self.owners)?;
        let evict = sessions.len() >= MAX_SESSIONS;
        prune_sessions(&mut sessions, Instant::now(), evict);
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
        let writer_fence = Arc::new(WriteFence::default());
        let ledger = Arc::new(Mutex::new(InputLedger::default()));
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
        let write_writer = writer_fence.clone();
        let write_ledger = ledger.clone();
        let write_binding = binding.as_ref().map(|binding| binding.broker.clone());
        std::thread::spawn(move || {
            while let Ok(input) = receiver.recv() {
                if write_revoked.load(Ordering::Acquire) || write_failed.load(Ordering::Acquire) {
                    match input {
                        NativeInput::User { attachment, sequence, .. } => {
                            if let Ok(mut ledger) = write_ledger.lock() {
                                ledger.cancel(attachment, sequence);
                            }
                        }
                        NativeInput::Peer { wake, .. } => {
                            if let Some(broker) = &write_binding {
                                broker.finish_wake(&wake, false);
                            }
                        }
                    }
                    continue;
                }
                let (bytes, delivery, user) = match input {
                    // Input queued by an attachment that has since been replaced never reaches the PTY.
                    NativeInput::User { attachment, epoch, sequence, bytes } => {
                        (bytes, None, Some((attachment, epoch, sequence)))
                    }
                    NativeInput::Peer { wake, bytes, submit } => {
                        let Some(broker) = &write_binding else { continue; };
                        if !broker.wake_valid(&wake) { broker.finish_wake(&wake, false); continue; }
                        (bytes, Some((wake, submit)), None)
                    }
                };
                let gate = match user {
                    Some((attachment, epoch, sequence)) => {
                        let Ok(gate) = write_writer.gate.lock() else {
                            if let Ok(mut ledger) = write_ledger.lock() { ledger.fail_pending(); }
                            write_failed.store(true, Ordering::Release);
                            continue;
                        };
                        if write_writer.epoch.load(Ordering::SeqCst) != epoch {
                            if let Ok(mut ledger) = write_ledger.lock() { ledger.cancel(attachment, sequence); }
                            continue;
                        }
                        let queued = match write_ledger.lock() {
                            Ok(mut ledger) => ledger.mark_writing(attachment, sequence),
                            Err(_) => {
                                write_failed.store(true, Ordering::Release);
                                continue;
                            }
                        };
                        if !queued { continue; }
                        Some(gate)
                    }
                    None => None,
                };
                let (written, end) = write_counted(&mut writer, &bytes, || match user {
                    Some((_, epoch, _)) => write_writer.epoch.load(Ordering::SeqCst) != epoch,
                    None => write_revoked.load(Ordering::Acquire) || write_failed.load(Ordering::Acquire),
                });
                // A failed flush means the bytes may have reached the PTY; keep them unknown.
                let (outcome, io_failed) = match end {
                    Ok(WriteEnd::Complete) => {
                        let flush_failed = writer.flush().is_err();
                        (if flush_failed { InputState::Unknown } else { InputState::Written }, flush_failed)
                    }
                    Ok(WriteEnd::Cancelled) => (
                        if written == 0 { InputState::Cancelled } else { InputState::Unknown },
                        false,
                    ),
                    Err(_) => (InputState::Failed, true),
                };
                let success = outcome == InputState::Written;
                if let Some((attachment, _, sequence)) = user {
                    if let Ok(mut ledger) = write_ledger.lock() {
                        ledger.finish(attachment, sequence, written, outcome);
                    } else {
                        write_failed.store(true, Ordering::Release);
                    }
                }
                drop(gate);
                if let (Some(broker), Some((wake, submit))) = (&write_binding, delivery) {
                    if submit || !success { broker.finish_wake(&wake, success && submit); }
                }
                if io_failed {
                    if let Ok(mut ledger) = write_ledger.lock() { ledger.fail_pending(); }
                    write_failed.store(true, Ordering::Release);
                }
            }
        });
        let policy = if request.guarded {
            Policy::Guarded
        } else {
            Policy::ConnectionOwned
        };
        let label = request.label.clone();
        let mut session = Session {
            launch: request,
            policy,
            label,
            run_id,
            binding,
            pty,
            output,
            screen,
            input,
            connection: Some(fence.connection_id.clone()),
            attachment_id: Uuid::new_v4().to_string(),
            writer: writer_fence,
            ledger,
            sequence: 0,
            failed,
            revoked,
            stop_requested: false,
            ended_at: None,
        };
        let state = session.state(fence, None, None)?;
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
        let owner = lock(&self.owners)?
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
            || (!request.attachment_id.is_empty() && !valid_id(&request.attachment_id))
        {
            return Err(Error::ManagedInvalidInput);
        }
        let mut receipt = None;
        match action {
            TerminalAction::TerminalInput => {
                if !may_control(&session, fence, &request) {
                    return Err(Error::ManagedStaleAttachment);
                }
                if request.input.is_empty()
                    || request.input.len() > MAX_INPUT_BYTES
                    || request.input_sequence == 0
                {
                    return Err(Error::ManagedInvalidInput);
                }
                let digest = input_digest(&request.input);
                let attachment = session.writer.attachment.load(Ordering::SeqCst);
                if request.input_sequence == session.sequence && session.sequence > 0 {
                    // A retry of the accepted tail is idempotent only for identical bytes.
                    match session.receipt(attachment, session.sequence) {
                        Some(existing)
                            if existing.digest == digest && existing.len == request.input.len() =>
                        {
                            receipt = Some(existing);
                        }
                        Some(_) => return Err(Error::ManagedInputConflict),
                        None => return Err(Error::ManagedDeliveryUnknown),
                    }
                } else {
                    let Some(next) = session.sequence.checked_add(1) else {
                        // Sequence overflow invalidates the attachment instead of wrapping.
                        session.failed.store(true, Ordering::Release);
                        return Err(Error::ManagedInputConflict);
                    };
                    if next != request.input_sequence {
                        return Err(Error::ManagedInputConflict);
                    }
                    if session.failed.load(Ordering::Acquire)
                        || session.pty.exit_code().map_err(path_error)?.is_some()
                    {
                        return Err(Error::ManagedUnavailable);
                    }
                    let len = request.input.len();
                    session.ledger.lock().map_err(|_| Error::ManagedUnavailable)?.reserve(
                        attachment,
                        request.input_sequence,
                        Uuid::new_v4().to_string(),
                        digest,
                        len,
                    )?;
                    if let Some(binding) = &session.binding {
                        binding.broker.input_bytes(&binding.terminal, &request.input);
                    }
                    let sequence = request.input_sequence;
                    if session
                        .input
                        .try_send(NativeInput::User {
                            attachment,
                            epoch: session.writer.epoch.load(Ordering::SeqCst),
                            sequence,
                            bytes: request.input,
                        })
                        .is_err()
                    {
                        if let Ok(mut ledger) = session.ledger.lock() {
                            ledger.remove(attachment, sequence);
                        }
                        return Err(Error::ManagedCapacityExceeded);
                    }
                    session.sequence = sequence;
                    receipt = session.receipt(attachment, sequence);
                }
            }
            TerminalAction::TerminalResize => {
                if !may_control(&session, fence, &request) {
                    return Err(Error::ManagedStaleAttachment);
                }
                let size = dimensions(request.columns, request.rows)?;
                session
                    .pty
                    .resize(size)
                    .map_err(path_error)?;
                if let Some(screen) = &session.screen { lock(screen)?.resize(size.rows, size.columns); }
                if let Some(binding) = &session.binding { binding.broker.output(&binding.terminal); }
            }
            TerminalAction::TerminalStop => {
                // This slice has no authenticated operator role; stop stays writer-bound.
                if !may_control(&session, fence, &request) {
                    return Err(Error::ManagedStaleAttachment);
                }
                session.pty.stop().map_err(path_error)?;
                session.stop_requested = true;
            }
            TerminalAction::TerminalRead => {
                if !may_control(&session, fence, &request) {
                    return Err(Error::ManagedStaleAttachment);
                }
            }
            TerminalAction::TerminalAttach => {
                let held = session.writer.attachment.load(Ordering::SeqCst);
                let same_connection =
                    session.connection.as_deref() == Some(fence.connection_id.as_str());
                let occupied = session.connection.is_some() && !same_connection;
                let reattach = attach_decision(
                    session.policy,
                    held,
                    &session.attachment_id,
                    request.attachment_generation,
                    &request.attachment_id,
                    request.takeover,
                    same_connection,
                    occupied,
                )?;
                if reattach {
                    session.connection = Some(fence.connection_id.clone());
                } else {
                    let next = held.checked_add(1).ok_or(Error::ManagedUnavailable)?;
                    match session.writer.rotate(next) {
                        RotateOutcome::Rotated => {}
                        RotateOutcome::Busy => return Err(Error::ManagedConflict),
                        RotateOutcome::Unavailable => {
                            if let Ok(mut ledger) = session.ledger.lock() {
                                ledger.fail_pending();
                            }
                            session.failed.store(true, Ordering::Release);
                            return Err(Error::ManagedUnavailable);
                        }
                    }
                    session.ledger.lock().map_err(|_| Error::ManagedUnavailable)?.clear();
                    session.attachment_id = Uuid::new_v4().to_string();
                    // Input sequences and receipts are scoped to one attachment.
                    session.sequence = 0;
                    session.connection = Some(fence.connection_id.clone());
                }
            }
            TerminalAction::TerminalDetach => {
                if session.policy != Policy::Guarded {
                    return Err(Error::ManagedFeatureUnavailable);
                }
                if !may_control(&session, fence, &request) {
                    return Err(Error::ManagedStaleAttachment);
                }
                // The attachment identity survives so the same writer can reattach after a transport loss.
                session.connection = None;
                session.writer.cancel_input();
                if let Ok(mut ledger) = session.ledger.lock() {
                    ledger.cancel_pending();
                }
            }
            TerminalAction::TerminalRelease => {
                if !may_control(&session, fence, &request) {
                    return Err(Error::ManagedStaleAttachment);
                }
                if session.pty.exit_code().map_err(path_error)?.is_none()
                    || session.pty.is_active().map_err(path_error)?
                    || !lock(&session.output)?.closed
                {
                    return Err(Error::ManagedConflict);
                }
                let state = session.state(fence, None, None)?;
                drop(session);
                let mut sessions = lock(&self.owners)?;
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
            receipt,
        )
    }

    pub(super) fn disconnect(&self, connection: &str) {
        let Ok(owners) = self
            .owners
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
                session.writer.cancel_input();
                if let Ok(mut ledger) = session.ledger.lock() {
                    ledger.cancel_pending();
                }
                if session.policy == Policy::Guarded {
                    // Transport loss detaches a guarded run; its process and attachment stay live.
                    continue;
                }
                session.revoked.store(true, Ordering::Release);
                session.binding.take();
                // Keep native ownership on stop failure; idle exit must still observe it.
                session.stop_requested = session.pty.stop().is_ok();
            }
        }
    }

    pub(super) fn active(&self) -> bool {
        let Ok(mut owners) = self.owners.lock() else {
            return true;
        };
        let now = Instant::now();
        let mut active = false;
        owners.retain(|_, owner| {
            let Ok(mut session) = owner.lock() else {
                active = true;
                return true;
            };
            let retained = retain_session(&mut session, now);
            active |= retained;
            retained
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

    #[test]
    fn attach_requires_matching_writer_or_explicit_takeover() {
        let id = Uuid::new_v4().to_string();
        let other = Uuid::new_v4().to_string();
        assert_eq!(
            attach_decision(Policy::Guarded, 2, &id, 2, &id, false, true, false),
            Ok(true)
        );
        assert_eq!(
            attach_decision(Policy::Guarded, 2, &id, 2, &id, false, false, false),
            Ok(true)
        );
        assert_eq!(
            attach_decision(Policy::Guarded, 2, &id, 2, &id, false, false, true),
            Err(Error::ManagedConflict)
        );
        assert_eq!(
            attach_decision(Policy::Guarded, 2, &id, 2, &id, true, false, true),
            Ok(false)
        );
        assert_eq!(
            attach_decision(Policy::Guarded, 2, &id, 2, &id, true, true, false),
            Ok(true)
        );
        assert_eq!(
            attach_decision(Policy::Guarded, 2, &id, 2, &other, false, true, false),
            Err(Error::ManagedStaleAttachment)
        );
        assert_eq!(
            attach_decision(Policy::Guarded, 2, &id, 1, &id, false, false, false),
            Err(Error::ManagedStaleAttachment)
        );
        assert_eq!(
            attach_decision(Policy::Guarded, 2, &id, 0, "", false, false, false),
            Err(Error::ManagedConflict)
        );
        assert_eq!(
            attach_decision(Policy::Guarded, 2, &id, 0, "", true, false, false),
            Err(Error::ManagedConflict)
        );
        assert_eq!(
            attach_decision(Policy::Guarded, 2, &id, 1, &id, true, false, false),
            Err(Error::ManagedStaleAttachment)
        );
        assert_eq!(
            attach_decision(Policy::Guarded, 2, &id, 2, "", true, false, true),
            Ok(false)
        );
        assert_eq!(
            attach_decision(Policy::Guarded, 2, &id, 2, &other, true, false, true),
            Err(Error::ManagedStaleAttachment)
        );
        assert_eq!(
            attach_decision(Policy::ConnectionOwned, 1, &id, 1, &id, false, true, false),
            Err(Error::ManagedFeatureUnavailable)
        );
    }

    #[test]
    fn writer_rotation_refuses_an_in_flight_native_write() {
        let fence = WriteFence::default();
        let epoch = fence.epoch.load(Ordering::SeqCst);
        let gate = fence.gate.lock().unwrap();
        assert_eq!(fence.rotate(2), RotateOutcome::Busy);
        assert_eq!(fence.attachment.load(Ordering::SeqCst), 1);
        assert_eq!(fence.epoch.load(Ordering::SeqCst), epoch);
        drop(gate);
        assert_eq!(fence.rotate(2), RotateOutcome::Rotated);
        assert_eq!(fence.attachment.load(Ordering::SeqCst), 2);
        assert_ne!(fence.epoch.load(Ordering::SeqCst), epoch);
    }

    #[test]
    fn writer_rotation_fails_closed_when_the_gate_is_poisoned() {
        let fence = Arc::new(WriteFence::default());
        let poisoned = fence.clone();
        let _ = std::thread::spawn(move || {
            let _gate = poisoned.gate.lock().unwrap();
            panic!("poison writer gate");
        })
        .join();
        assert!(fence.gate.is_poisoned());
        let epoch = fence.epoch.load(Ordering::SeqCst);
        assert_eq!(fence.rotate(2), RotateOutcome::Unavailable);
        assert_eq!(fence.attachment.load(Ordering::SeqCst), 1);
        assert_eq!(fence.epoch.load(Ordering::SeqCst), epoch);
    }

    #[test]
    fn input_ledger_tracks_terminal_and_pending_states() {
        let mut ledger = InputLedger::default();
        let digest = input_digest(b"hello");
        let record = ledger
            .reserve(1, 1, "receipt".into(), digest, 5)
            .unwrap();
        assert_eq!(record.state, InputState::Accepted);
        assert_eq!(ledger.pending_bytes, 5);
        assert_eq!(
            ledger.reserve(1, 1, "other".into(), digest, 5),
            Err(Error::ManagedInputConflict)
        );
        assert!(ledger.mark_writing(1, 1));
        assert_eq!(ledger.find(1, 1).unwrap().state, InputState::Writing);
        ledger.cancel(1, 1);
        assert_eq!(ledger.find(1, 1).unwrap().state, InputState::Unknown);
        assert_eq!(ledger.pending_bytes, 0);
        ledger.finish(1, 1, 3, InputState::Failed);
        let record = ledger.find(1, 1).unwrap();
        assert_eq!(record.state, InputState::Failed);
        assert_eq!(record.written, 3);
        assert_eq!(record.proto().disposition, InputDisposition::InputFailed as i32);

        ledger.reserve(1, 2, "queued".into(), digest, 5).unwrap();
        ledger.cancel(1, 2);
        assert_eq!(ledger.find(1, 2).unwrap().state, InputState::Cancelled);
        assert_eq!(ledger.pending_bytes, 0);
        ledger.remove(1, 2);
        assert!(ledger.find(1, 2).is_none());
    }

    #[test]
    fn stale_generation_cannot_cancel_a_new_attachment() {
        let mut ledger = InputLedger::default();
        ledger
            .reserve(1, 1, "old".into(), [1; 32], 1)
            .unwrap();
        ledger.clear();
        ledger
            .reserve(2, 1, "new".into(), [2; 32], 1)
            .unwrap();
        // An old writer message dequeued after takeover must not target the new sequence.
        ledger.cancel(1, 1);
        assert_eq!(ledger.find(2, 1).unwrap().state, InputState::Accepted);
    }

    #[test]
    fn input_ledger_bounds_pending_bytes_and_entries() {
        let mut ledger = InputLedger::default();
        assert_eq!(
            ledger.reserve(1, 0, "zero".into(), [0; 32], 1),
            Err(Error::ManagedInvalidInput)
        );
        assert_eq!(
            ledger.reserve(1, 1, "empty".into(), [0; 32], 0),
            Err(Error::ManagedInvalidInput)
        );
        assert_eq!(
            ledger.reserve(1, 2, "huge".into(), [0; 32], MAX_INPUT_BYTES + 1),
            Err(Error::ManagedInvalidInput)
        );
        for sequence in 1..=16u64 {
            ledger
                .reserve(1, sequence, format!("{sequence}"), [0; 32], MAX_INPUT_BYTES)
                .unwrap();
        }
        assert_eq!(ledger.pending_bytes, MAX_PENDING_INPUT_BYTES);
        assert_eq!(
            ledger.reserve(1, 17, "over".into(), [0; 32], 1),
            Err(Error::ManagedCapacityExceeded)
        );
        ledger.pending_bytes = 0;
        for entry in &mut ledger.entries {
            entry.state = InputState::Written;
        }
        assert!(ledger.reserve(1, 17, "after".into(), [0; 32], 1).is_ok());

        let mut full = InputLedger::default();
        for sequence in 1..=MAX_INPUT_RECEIPTS as u64 {
            full.reserve(1, sequence, format!("{sequence}"), [0; 32], 1)
                .unwrap();
        }
        assert_eq!(
            full.reserve(1, 65, "pending".into(), [0; 32], 1),
            Err(Error::ManagedCapacityExceeded)
        );
        full.pending_bytes = 0;
        for entry in &mut full.entries {
            entry.state = InputState::Written;
        }
        assert!(full.reserve(1, 65, "evicted".into(), [0; 32], 1).is_ok());
        assert_eq!(full.entries.len(), MAX_INPUT_RECEIPTS);
        assert!(full.find(1, 1).is_none());
    }

    #[test]
    fn unknown_receipts_backpressure_instead_of_eviction() {
        let mut ledger = InputLedger::default();
        for sequence in 1..=MAX_INPUT_RECEIPTS as u64 {
            ledger
                .reserve(1, sequence, format!("{sequence}"), [0; 32], 1)
                .unwrap();
            assert!(ledger.mark_writing(1, sequence));
            ledger.cancel(1, sequence);
        }
        assert_eq!(ledger.pending_bytes, 0);
        assert_eq!(
            ledger.reserve(1, MAX_INPUT_RECEIPTS as u64 + 1, "blocked".into(), [0; 32], 1),
            Err(Error::ManagedCapacityExceeded)
        );
        for entry in &mut ledger.entries {
            entry.state = InputState::Failed;
        }
        assert!(ledger
            .reserve(1, MAX_INPUT_RECEIPTS as u64 + 1, "settled".into(), [0; 32], 1)
            .is_ok());
    }

    struct ShortWriter {
        remaining: usize,
        written: usize,
    }

    impl Write for ShortWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let count = bytes.len().min(self.remaining).min(2);
            if count == 0 {
                return Err(std::io::Error::from(std::io::ErrorKind::WriteZero));
            }
            self.remaining -= count;
            self.written += count;
            Ok(count)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn counted_writer_reports_short_and_failed_writes() {
        let mut writer = ShortWriter {
            remaining: 5,
            written: 0,
        };
        let (written, result) = write_counted(&mut writer, b"hello", || false);
        assert_eq!(result.unwrap(), WriteEnd::Complete);
        assert_eq!(written, 5);
        assert_eq!(writer.written, 5);

        let mut blocked = ShortWriter {
            remaining: 3,
            written: 0,
        };
        let (written, result) = write_counted(&mut blocked, b"hello", || false);
        assert_eq!(written, 3);
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::WriteZero
        );

        let mut cancelled = ShortWriter {
            remaining: 5,
            written: 0,
        };
        let mut calls = 0;
        let (written, result) = write_counted(&mut cancelled, b"hello", || {
            calls += 1;
            calls > 1
        });
        assert_eq!(result.unwrap(), WriteEnd::Cancelled);
        assert_eq!(written, 1);
        assert_eq!(cancelled.written, 1);
    }

    #[test]
    fn labels_are_bounded_nonsecret_selectors() {
        assert!(valid_label(None));
        assert!(valid_label(Some("build")));
        assert!(!valid_label(Some("")));
        assert!(!valid_label(Some("bad\u{7}label")));
        assert!(!valid_label(Some("x".repeat(MAX_LABEL + 1).as_str())));
    }
}

impl Session {
    fn receipt(&self, attachment: u64, sequence: u64) -> Option<InputRecord> {
        if sequence == 0 {
            return None;
        }
        self.ledger
            .lock()
            .ok()
            .and_then(|ledger| ledger.find(attachment, sequence).cloned())
    }

    fn state(
        &mut self,
        fence: &ManagedFence,
        offset: Option<u64>,
        receipt: Option<InputRecord>,
    ) -> Result<TerminalState, Error> {
        let processes_active = Some(self.pty.is_active().map_err(path_error)?);
        let exit_code = self.pty.exit_code().map_err(path_error)?;
        if processes_active == Some(false) {
            self.binding.take();
        }
        let guarded = self.policy == Policy::Guarded;
        let (attachment_generation, attachment_id) = if guarded {
            (
                self.writer.attachment.load(Ordering::SeqCst),
                self.attachment_id.clone(),
            )
        } else {
            (0, String::new())
        };
        let receipt = if guarded {
            let attachment = self.writer.attachment.load(Ordering::SeqCst);
            receipt.or_else(|| self.receipt(attachment, self.sequence))
        } else {
            None
        };
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
            guarded: Some(guarded),
            label: self.label.clone(),
            attachment_generation,
            attachment_id,
            input_receipt: receipt.as_ref().map(InputRecord::proto),
        })
    }
}
