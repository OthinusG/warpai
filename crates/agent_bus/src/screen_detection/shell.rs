//! Render-only shell lifecycle state derived from the bundled bootstrap protocol.
use remote_protocol::ansi_c_decoder::{is_ansi_c_quoted, parse_ansi_c_quoted_string};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const MAX_FIELD_BYTES: usize = 4096;
const MAX_BLOCKS: usize = 16;
const MAX_PROMPT_BYTES: usize = 8 * 1024;
const MAX_COMPLETED_BYTES: usize = 32 * 1024;
pub(super) const MAX_SHELL_FRAME_BYTES: usize = 20 * 1024;
pub(crate) const SHELL_STATE_CHARGE: usize = 144 * 1024;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShellPrompt {
    pub pwd: Option<String>,
    pub ps1: Option<String>,
    pub ps1_is_encoded: Option<bool>,
    pub honor_ps1: Option<bool>,
    pub rprompt: Option<String>,
    pub git_head: Option<String>,
    pub git_branch: Option<String>,
    pub virtual_env: Option<String>,
    pub conda_env: Option<String>,
    pub node_version: Option<String>,
    pub kube_config: Option<String>,
    pub session_id: Option<u64>,
    #[serde(default)]
    is_after_in_band_command: bool,
}

impl ShellPrompt {
    fn valid(&self) -> bool {
        let fields = [
            &self.pwd,
            &self.ps1,
            &self.rprompt,
            &self.git_head,
            &self.git_branch,
            &self.virtual_env,
            &self.conda_env,
            &self.node_version,
            &self.kube_config,
        ];
        fields
            .iter()
            .filter_map(|field| field.as_ref())
            .map(String::len)
            .sum::<usize>()
            <= MAX_PROMPT_BYTES
            && fields
                .into_iter()
                .flatten()
                .all(|value| value.len() <= MAX_FIELD_BYTES)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShellRow {
    pub row: u64,
    pub column: u16,
}

#[derive(Clone, Copy)]
pub(super) struct ShellViewport {
    anchor: Option<ShellRow>,
    retained_start: u64,
}

impl ShellViewport {
    pub(super) fn from_screen(screen: &vt100::Screen) -> Self {
        Self {
            anchor: screen
                .primary_row_anchor()
                .map(|(row, column)| ShellRow { row, column }),
            retained_start: screen.retained_primary_row_start(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShellBlock {
    pub id: Option<String>,
    pub command: String,
    pub cwd: Option<String>,
    pub start: ShellRow,
    pub end: Option<ShellRow>,
    pub exit_code: Option<i32>,
}

impl ShellBlock {
    fn text_bytes(&self) -> usize {
        self.command.len()
            + self.cwd.as_ref().map_or(0, String::len)
            + self.id.as_ref().map_or(0, String::len)
    }

    fn valid(&self, finished: bool) -> bool {
        self.command.len() <= MAX_FIELD_BYTES
            && self
                .cwd
                .as_ref()
                .is_none_or(|value| value.len() <= MAX_FIELD_BYTES)
            && self.id.as_ref().is_none_or(|value| value.len() <= 128)
            && self.start.column <= 1000
            && self.end.is_some() == finished
            && self.end.is_none_or(|end| end.column <= 1000)
            && self.exit_code.is_some() == finished
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShellState {
    pub prompt: Option<ShellPrompt>,
    pub input_buffer: Option<String>,
    pub active: Option<ShellBlock>,
    pub completed: VecDeque<ShellBlock>,
    pub history_truncated: bool,
    next_block_id: Option<String>,
    pending: Option<KeyValue>,
    dcs: Option<Vec<u8>>,
    in_band: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum KeyValue {
    Precmd(Box<ShellPrompt>),
    Preexec(Option<String>),
    CommandFinished {
        exit_code: Option<i32>,
        next_block_id: Option<String>,
    },
    InputBuffer(Option<String>),
    Clear,
}

#[derive(Deserialize)]
#[serde(tag = "hook", content = "value")]
enum Hook {
    Precmd(Box<ShellPrompt>),
    Preexec {
        command: String,
    },
    CommandFinished {
        exit_code: i32,
        next_block_id: String,
    },
    InputBuffer {
        buffer: String,
    },
    Clear {},
}

fn decode_hex(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() > MAX_SHELL_FRAME_BYTES || !data.len().is_multiple_of(2) {
        return None;
    }
    data.chunks_exact(2)
        .map(|digits| {
            let high = char::from(digits[0]).to_digit(16)?;
            let low = char::from(digits[1]).to_digit(16)?;
            u8::try_from(high * 16 + low).ok()
        })
        .collect()
}

fn field(parts: &[&[u8]]) -> Option<String> {
    let mut text = String::new();
    for (index, part) in parts.iter().enumerate() {
        if !text.is_empty() || index > 0 {
            text.push(';');
        }
        text.push_str(std::str::from_utf8(part).ok()?);
        if text.len() > MAX_FIELD_BYTES {
            return None;
        }
    }
    if is_ansi_c_quoted(&text) {
        text = parse_ansi_c_quoted_string(text);
    }
    Some(text)
}

impl KeyValue {
    fn new(name: &[u8]) -> Option<Self> {
        match name {
            b"Precmd" => {
                let prompt = ShellPrompt {
                    ps1_is_encoded: Some(true),
                    ..Default::default()
                };
                Some(Self::Precmd(Box::new(prompt)))
            }
            b"Preexec" => Some(Self::Preexec(None)),
            b"CommandFinished" => Some(Self::CommandFinished {
                exit_code: None,
                next_block_id: None,
            }),
            b"InputBuffer" => Some(Self::InputBuffer(None)),
            b"Clear" => Some(Self::Clear),
            _ => None,
        }
    }

    fn update(&mut self, key: &[u8], value: String) {
        let nonempty = |value: String| (!value.is_empty()).then_some(value);
        match (self, key) {
            (Self::Preexec(command), b"command") => *command = Some(value),
            (Self::InputBuffer(buffer), b"buffer") => *buffer = Some(value),
            (Self::CommandFinished { exit_code, .. }, b"exit_code") => {
                *exit_code = value.parse().ok()
            }
            (Self::CommandFinished { next_block_id, .. }, b"next_block_id") => {
                *next_block_id = nonempty(value).filter(|value| value.len() <= 128);
            }
            (Self::Precmd(prompt), key) => match key {
                b"pwd" => prompt.pwd = nonempty(value),
                b"ps1" => prompt.ps1 = nonempty(value),
                b"rprompt" => prompt.rprompt = nonempty(value),
                b"ps1_is_encoded" => prompt.ps1_is_encoded = value.parse().ok(),
                b"honor_ps1" => prompt.honor_ps1 = value.parse().ok(),
                b"git_head" => prompt.git_head = nonempty(value),
                b"git_branch" => prompt.git_branch = nonempty(value),
                b"virtual_env" => prompt.virtual_env = nonempty(value),
                b"conda_env" => prompt.conda_env = nonempty(value),
                b"node_version" => prompt.node_version = nonempty(value),
                b"kube_config" => prompt.kube_config = nonempty(value),
                b"session_id" => prompt.session_id = value.parse().ok(),
                _ => (),
            },
            _ => (),
        }
    }

    fn valid(&self) -> bool {
        match self {
            Self::Precmd(prompt) => prompt.valid(),
            Self::Preexec(text) | Self::InputBuffer(text) => text
                .as_ref()
                .is_none_or(|text| text.len() <= MAX_FIELD_BYTES),
            Self::CommandFinished { next_block_id, .. } => {
                next_block_id.as_ref().is_none_or(|id| id.len() <= 128)
            }
            Self::Clear => true,
        }
    }

    fn finish(self) -> Option<Hook> {
        Some(match self {
            Self::Precmd(prompt) => Hook::Precmd(prompt),
            Self::Preexec(command) => Hook::Preexec { command: command? },
            Self::CommandFinished {
                exit_code,
                next_block_id,
            } => Hook::CommandFinished {
                exit_code: exit_code?,
                next_block_id: next_block_id?,
            },
            Self::InputBuffer(buffer) => Hook::InputBuffer { buffer: buffer? },
            Self::Clear => Hook::Clear {},
        })
    }
}

impl ShellState {
    pub(super) fn valid(&self) -> bool {
        self.completed.len() <= MAX_BLOCKS
            && self
                .completed
                .iter()
                .map(ShellBlock::text_bytes)
                .sum::<usize>()
                <= MAX_COMPLETED_BYTES
            && self.completed.iter().all(|block| block.valid(true))
            && self.active.as_ref().is_none_or(|block| block.valid(false))
            && self.prompt.as_ref().is_none_or(ShellPrompt::valid)
            && self
                .input_buffer
                .as_ref()
                .is_none_or(|text| text.len() <= MAX_FIELD_BYTES)
            && self.next_block_id.as_ref().is_none_or(|id| id.len() <= 128)
            && self.pending.as_ref().is_none_or(KeyValue::valid)
            && self
                .dcs
                .as_ref()
                .is_none_or(|data| data.len() <= MAX_SHELL_FRAME_BYTES && data.iter().all(u8::is_ascii_hexdigit))
    }

    pub(super) fn dcs_start(&mut self, intermediates: &[u8], action: char) {
        self.dcs = (intermediates == b"$" && action == 'd').then(Vec::new);
    }

    pub(super) fn dcs_byte(&mut self, byte: u8) {
        let Some(data) = self.dcs.as_mut() else {
            return;
        };
        if data.len() == MAX_SHELL_FRAME_BYTES || !byte.is_ascii_hexdigit() {
            self.dcs = None;
        } else {
            data.push(byte);
        }
    }

    pub(super) fn dcs_end(&mut self, screen: &vt100::Screen) {
        if let Some(data) = self.dcs.take() {
            self.encoded(ShellViewport::from_screen(screen), &data);
        }
    }

    pub(super) fn incomplete_dcs(&self) -> Option<&[u8]> {
        self.dcs.as_deref()
    }

    pub(super) fn osc(&mut self, viewport: ShellViewport, payload: &[u8]) {
        let parts: Vec<_> = payload.splitn(5, |byte| *byte == b';').collect();
        match parts.as_slice() {
            [b"9277", b"A", ..] => self.in_band = true,
            [b"9277", b"B", ..] => self.in_band = false,
            [b"9278", b"d", data] => self.encoded(viewport, data),
            [b"9278", b"k", b"A", name] => self.pending = KeyValue::new(name),
            [b"9278", b"k", b"B", key, value @ ..] => {
                if let (Some(pending), Some(value)) = (&mut self.pending, field(value)) {
                    pending.update(key, value);
                    if !pending.valid() {
                        self.pending = None;
                        self.history_truncated = true;
                    }
                }
            }
            [b"9278", b"k", b"C"] => {
                if let Some(hook) = self.pending.take().and_then(KeyValue::finish) {
                    self.apply(viewport, hook);
                }
            }
            _ => (),
        }
    }

    fn encoded(&mut self, viewport: ShellViewport, bytes: &[u8]) {
        if let Some(hook) = decode_hex(bytes).and_then(|json| serde_json::from_slice(&json).ok()) {
            self.apply(viewport, hook);
        }
    }

    fn apply(&mut self, viewport: ShellViewport, hook: Hook) {
        let Some(anchor) = viewport.anchor.filter(|_| !self.in_band) else {
            return;
        };
        match hook {
            Hook::Precmd(prompt) if !prompt.is_after_in_band_command && prompt.valid() => {
                self.prompt = Some(*prompt);
            }
            Hook::Preexec { command } if command.len() <= MAX_FIELD_BYTES => {
                let cwd = self.prompt.as_ref().and_then(|prompt| prompt.pwd.clone());
                self.input_buffer = None;
                self.active = Some(ShellBlock {
                    id: self.next_block_id.take(),
                    command,
                    cwd,
                    start: anchor,
                    end: None,
                    exit_code: None,
                });
            }
            Hook::CommandFinished {
                exit_code,
                next_block_id,
            } if next_block_id.len() <= 128 => {
                self.next_block_id = Some(next_block_id);
                if let Some(mut command) = self.active.take() {
                    command.end = Some(anchor);
                    command.exit_code = Some(exit_code);
                    if self.completed.len() == MAX_BLOCKS {
                        self.completed.pop_front();
                        self.history_truncated = true;
                    }
                    self.completed.push_back(command);
                    while self
                        .completed
                        .iter()
                        .map(ShellBlock::text_bytes)
                        .sum::<usize>()
                        > MAX_COMPLETED_BYTES
                    {
                        self.completed.pop_front();
                        self.history_truncated = true;
                    }
                }
            }
            Hook::InputBuffer { buffer } if buffer.len() <= MAX_FIELD_BYTES => {
                self.input_buffer = Some(buffer);
            }
            Hook::Clear {} => {
                self.history_truncated |= !self.completed.is_empty();
                self.completed.clear();
            }
            _ => (),
        }
        while self.completed.front().is_some_and(|block| {
            block
                .end
                .is_some_and(|end| end.row < viewport.retained_start)
        }) {
            self.completed.pop_front();
            self.history_truncated = true;
        }
    }
}
