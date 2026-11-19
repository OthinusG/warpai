//! Memory-only terminal state for Companion-owned PTYs and private checkpoints.
use super::{activity_with_progress, shell::{ShellState, ShellViewport, MAX_SHELL_FRAME_BYTES}};
use bincode::Options;
use crate::{readiness::Activity, transport::ScreenContext};
use remote_protocol::managed::TERMINAL_SNAPSHOT_MAX_BYTES;
use serde::{Deserialize, Serialize};

const MAX_OSC_BYTES: usize = 4096 + 16;
const CHECKPOINT_VERSION: u16 = 2;
pub(crate) const GUARDED_SCROLLBACK_ROWS: usize = 128;

#[derive(Clone, Default, Serialize, Deserialize)]
struct Signals {
    title: String,
    progress: String,
    shell: Option<ShellState>,
}

impl vt100::Callbacks for Signals {
    fn set_window_title(&mut self, _: &mut vt100::Screen, title: &[u8]) {
        if title.len() <= 4096 { self.title = String::from_utf8_lossy(title).into_owned(); }
    }
    fn unhandled_osc(&mut self, _: &mut vt100::Screen, params: &[&[u8]]) {
        if params.first() == Some(&b"9".as_slice()) && params.get(1) == Some(&b"4".as_slice())
            && params.len() <= 4 && params.get(2).is_some_and(|state| matches!(*state, b"0" | b"1" | b"2" | b"3" | b"4"))
            && params.iter().skip(3).all(|value| value.len() <= 4
                && std::str::from_utf8(value).ok().and_then(|value| value.parse::<i16>().ok())
                    .is_some_and(|value| (-1..=100).contains(&value))) {
            self.progress = params[1..].iter().filter_map(|part| std::str::from_utf8(part).ok())
                .collect::<Vec<_>>().join(";").chars().take(256).collect();
        }
    }
    fn dcs_start(&mut self, _: &mut vt100::Screen, intermediates: &[u8], action: char) {
        if let Some(shell) = &mut self.shell {
            shell.dcs_start(intermediates, action);
        }
    }
    fn dcs_byte(&mut self, byte: u8) {
        if let Some(shell) = &mut self.shell {
            shell.dcs_byte(byte);
        }
    }
    fn dcs_end(&mut self, screen: &mut vt100::Screen) {
        if let Some(shell) = &mut self.shell {
            shell.dcs_end(screen);
        }
    }

}

#[derive(Clone, Serialize, Deserialize)]
enum OscInput {
    Reading(Vec<u8>),
    Discarding,
}

#[derive(Serialize, Deserialize)]
struct ScreenCheckpoint<'a> {
    version: u16,
    parser: vt100::Checkpoint<'a>,
    signals: Signals,
    escape_pending: bool,
    osc: Option<OscInput>,
}

pub(crate) struct NativeScreen {
    parser: vt100::Parser<Signals>,
    escape_pending: bool,
    osc: Option<OscInput>,
}
impl NativeScreen {
    pub(crate) fn new(rows: u16, columns: u16) -> Self {
        Self::with_scrollback(rows, columns, 0)
    }

    pub(crate) fn guarded(rows: u16, columns: u16) -> Self {
        Self::with_scrollback(rows, columns, GUARDED_SCROLLBACK_ROWS)
    }

    fn with_scrollback(rows: u16, columns: u16, history: usize) -> Self {
        let signals = Signals { shell: (history > 0).then(ShellState::default), ..Default::default() };
        Self {
            parser: vt100::Parser::new_with_callbacks(rows, columns, history, signals),
            escape_pending: false,
            osc: None,
        }
    }

    pub(crate) fn checkpoint(&self) -> Option<Vec<u8>> {
        let checkpoint = ScreenCheckpoint {
            version: CHECKPOINT_VERSION,
            parser: self.parser.checkpoint(),
            signals: self.parser.callbacks().clone(),
            escape_pending: self.escape_pending,
            osc: self.osc.clone(),
        };
        bincode::DefaultOptions::new().with_limit(TERMINAL_SNAPSHOT_MAX_BYTES as u64)
            .serialize(&checkpoint).ok()
    }

    pub(crate) fn from_checkpoint(data: &[u8]) -> Option<Self> {
        if data.len() > TERMINAL_SNAPSHOT_MAX_BYTES { return None; }
        let checkpoint: ScreenCheckpoint<'_> = bincode::DefaultOptions::new()
            .with_limit(TERMINAL_SNAPSHOT_MAX_BYTES as u64).deserialize(data).ok()?;
        if checkpoint.version != CHECKPOINT_VERSION || checkpoint.signals.title.len() > 4096 * 3
            || checkpoint.signals.progress.len() > 256
            || !checkpoint.signals.shell.as_ref().is_some_and(ShellState::valid)
            || matches!(&checkpoint.osc, Some(OscInput::Reading(buffer))
                if buffer.len() > MAX_SHELL_FRAME_BYTES + 16
                    || buffer.iter().any(|byte| matches!(byte, 0x07 | 0x18 | 0x1a | 0x1b)))
            || checkpoint.escape_pending && checkpoint.osc.is_some()
        {
            return None;
        }
        let parser = vt100::Parser::from_checkpoint(checkpoint.parser, checkpoint.signals, MAX_SHELL_FRAME_BYTES + 16, GUARDED_SCROLLBACK_ROWS)?;
        Some(Self { parser, escape_pending: checkpoint.escape_pending, osc: checkpoint.osc })
    }

    pub(crate) fn size(&self) -> (u16, u16) { self.parser.screen().size() }
    pub(crate) fn screen(&self) -> &vt100::Screen { self.parser.screen() }
    pub(crate) fn shell(&self) -> Option<&ShellState> { self.parser.callbacks().shell.as_ref() }
    pub(crate) fn at_input_boundary(&self) -> bool {
        self.osc.is_none() && !self.escape_pending && self.parser.at_input_boundary()
    }
    pub(crate) fn incomplete_prefix(&self) -> Option<Vec<u8>> {
        if let Some(osc) = &self.osc {
            let OscInput::Reading(payload) = osc else { return None };
            return Some([b"\x1b]".as_slice(), payload].concat());
        }
        if let Some(data) = self.shell().and_then(ShellState::incomplete_dcs) {
            return Some([b"\x1bP$d".as_slice(), data].concat());
        }
        self.parser.incomplete_prefix()
    }
    pub(crate) fn history_truncated(&self) -> bool {
        self.screen().history_truncated() || self.shell().is_some_and(|shell| shell.history_truncated)
    }
    pub(crate) fn process(&mut self, mut bytes: &[u8]) {
        // vte's std parser accumulates an OSC until termination. Bound the
        // readiness projection while the native output stream stays intact.
        let osc_limit = if self.shell().is_some() { MAX_SHELL_FRAME_BYTES + 16 } else { MAX_OSC_BYTES };
        while !bytes.is_empty() {
            if let Some(osc) = &mut self.osc {
                let end = bytes.iter().position(|&byte| matches!(byte, 0x07 | 0x18 | 0x1a | 0x1b));
                let payload = &bytes[..end.unwrap_or(bytes.len())];
                if let OscInput::Reading(buffer) = osc {
                    if payload.len() <= osc_limit - buffer.len() {
                        buffer.extend_from_slice(payload);
                    } else {
                        *osc = OscInput::Discarding;
                    }
                }
                let Some(end) = end else { return; };
                if let Some(OscInput::Reading(payload)) = self.osc.take() {
                    let is_shell = payload.starts_with(b"9278;") || payload.starts_with(b"9277;");
                    if is_shell && self.shell().is_some() {
                        let viewport = ShellViewport::from_screen(self.parser.screen());
                        if let Some(shell) = &mut self.parser.callbacks_mut().shell {
                            shell.osc(viewport, &payload);
                        }
                    } else if payload.len() <= MAX_OSC_BYTES {
                        self.parser.process(&payload);
                    }
                }
                self.parser.process(&[bytes[end]]);
                self.escape_pending = bytes[end] == 0x1b;
                bytes = &bytes[end + 1..];
                continue;
            }

            if !self.escape_pending && !bytes.contains(&0x1b) {
                self.parser.process(bytes);
                return;
            }
            let start = bytes.iter().position(|&byte| {
                if self.escape_pending && byte == b']' {
                    self.escape_pending = false;
                    return true;
                }
                match byte {
                    0x1b => self.escape_pending = true,
                    // C0 execution and ignored bytes leave vte in its Escape state.
                    0x00..=0x17 | 0x19 | 0x1c..=0x1f | 0x7f..=0xff => (),
                    _ => self.escape_pending = false,
                }
                false
            });
            let end = start.map_or(bytes.len(), |start| start + 1);
            self.parser.process(&bytes[..end]);
            bytes = &bytes[end..];
            if start.is_some() {
                self.osc = Some(OscInput::Reading(Vec::new()));
            }
        }
    }
    pub(crate) fn resize(&mut self, rows: u16, columns: u16) { self.parser.screen_mut().set_size(rows, columns); }
    pub(crate) fn bracketed_paste(&self) -> bool { self.parser.screen().bracketed_paste() }
    #[cfg(test)]
    pub(crate) fn test_contents(&self) -> String { self.parser.screen().contents() }
    pub(crate) fn activity(&self, context: &ScreenContext) -> Option<Activity> {
        let screen = self.parser.screen();
        let (rows, columns) = screen.size();
        let start = rows.saturating_sub(60);
        let text = screen.rows(0, columns).skip(usize::from(start)).collect::<Vec<_>>().join("\n");
        let cursor = screen.cursor_position().0;
        activity_with_progress(&context.program, &text, &self.parser.callbacks().title,
            &self.parser.callbacks().progress, context.quiet(),
            (!screen.hide_cursor() && cursor >= start).then_some(usize::from(cursor - start)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_capture_handles_split_unicode_redraw_and_alternate_screen() {
        let mut screen = NativeScreen::new(24, 80);
        let title = "\x1b]0;✦ 🐋 原生会话\x07".as_bytes();
        for byte in title { screen.process(&[*byte]); }
        assert_eq!(screen.parser.callbacks().title, "✦ 🐋 原生会话");
        screen.process(b"old transcript\x1b[?1049h\x1b[2J\x1b[HAllow once\r\nDeny");
        assert_eq!(screen.test_contents(), "Allow once\nDeny");
        screen.process(b"\x1b[2J\x1b[H\x1b]9;4;1;-1\x07");
        assert_eq!(screen.parser.callbacks().progress, "4;1;-1");
        screen.process(b"\x1b]9;4;1;101\x07\x1b]9;opaque notification\x07");
        assert_eq!(screen.parser.callbacks().progress, "4;1;-1");
        screen.resize(30, 100);
        assert_eq!(screen.parser.screen().size(), (30, 100));
        screen.process(b"\x1b[?1049l");
        assert!(screen.test_contents().contains("old transcript"));
    }

    #[test]
    fn oversized_osc_is_dropped_before_parser_allocation_and_recovers_at_termination() {
        let mut screen = NativeScreen::new(24, 80);
        screen.process(b"\x1b]0;stable\x07\x1b");
        screen.process(b"\0]0;");
        for _ in 0..8 {
            screen.process(&[b'a'; 1024]);
        }
        assert!(matches!(screen.osc, Some(OscInput::Discarding)));
        screen.process(b"\x1b");
        screen.process(b"\\visible");
        assert_eq!(screen.parser.callbacks().title, "stable");
        assert_eq!(screen.test_contents(), "visible");
        screen.process(b"\x1b]0;next\x18");
        assert_eq!(screen.parser.callbacks().title, "next");
    }

    #[test]
    fn bounded_projection_matches_vt100_across_normal_construct_boundaries() {
        let input = "normal🐋\x1b\0]0;first\x07\x1b]9;4;1;42\x1b\\\x1b[?1049h\x1b]2;alternate\x1a\r\n文本\x1b[?1049l".as_bytes();
        for cut in 0..=input.len() {
            let mut bounded = NativeScreen::new(24, 80);
            let mut original = vt100::Parser::new_with_callbacks(24, 80, 0, Signals::default());
            let (left, right) = input.split_at(cut);
            for part in [left, right] {
                bounded.process(part);
                original.process(part);
            }
            assert_eq!(bounded.test_contents(), original.screen().contents(), "cut {cut}");
            assert_eq!(bounded.parser.callbacks().title, original.callbacks().title, "cut {cut}");
            assert_eq!(bounded.parser.callbacks().progress, original.callbacks().progress, "cut {cut}");
        }
    }

    #[test]
    fn native_checkpoint_preserves_both_screens_history_and_partial_decoder_state() {
        let mut input = "old title\r\nline two\r\nline three\r\nline four\r\nnormal🐋 ".repeat(4).into_bytes();
        input.extend_from_slice("\x1b[12;30H\x1b7\x1b[?1049h\x1b[38:2:2:119:255m替代\x1b]0;✦ 🐋 原生会话\x1b\\\x1b[?2004h\x1b[?1049l\x1b8done".as_bytes());
        let mut uninterrupted = NativeScreen::guarded(12, 40);
        uninterrupted.process(&input);
        let expected = uninterrupted.checkpoint().unwrap();
        for cut in 0..=input.len() {
            let mut original = NativeScreen::guarded(12, 40);
            original.process(&input[..cut]);
            let data = original.checkpoint().unwrap();
            let mut restored = NativeScreen::from_checkpoint(&data).unwrap_or_else(|| panic!("restore at byte {cut}"));
            restored.process(&input[cut..]);
            assert_eq!(restored.checkpoint().unwrap(), expected, "cut {cut}");
        }
        let mut invalid = expected.clone();
        invalid.push(0);
        assert!(NativeScreen::from_checkpoint(&invalid).is_none());
    }

    #[test]
    fn shell_checkpoint_preserves_dcs_osc_and_key_value_hooks_at_each_cut() {
        fn encoded(input: &mut Vec<u8>, prefix: &[u8], json: &str, suffix: &[u8]) {
            input.extend_from_slice(prefix);
            for byte in json.bytes() {
                input.extend(format!("{byte:02x}").bytes());
            }
            input.extend_from_slice(suffix);
        }
        let mut input = Vec::new();
        encoded(&mut input, b"\x1b]9278;d;",
            r#"{"hook":"Precmd","value":{"pwd":"/work/项目","ps1":"$ ","session_id":42}}"#, b"\x07");
        input.extend_from_slice(b"$ \x1b]9278;k;A;Preexec\x07");
        let command = format!("echo 你好{}\nnext", ";again".repeat(20));
        input.extend(format!("\x1b]9278;k;B;command;$'{}'\x07",
            command.replace('\n', "\\n")).bytes());
        input.extend_from_slice(b"\x1b]9278;k;C\x07\r\nfirst line\r\n");
        input.extend_from_slice(b"\x1b[?1049hvisual dialog\x1b[?1049l");
        encoded(&mut input, b"\x1bP$d",
            r#"{"hook":"CommandFinished","value":{"exit_code":7,"next_block_id":"precmd-42-1"}}"#, b"\x9c");
        input.extend_from_slice(b"\x1b]9278;k;A;Preexec\x07\x1b]9278;k;B;command;second\x07\x1b]9278;k;C\x07");
        encoded(&mut input, b"\x1b]9278;d;",
            r#"{"hook":"InputBuffer","value":{"buffer":"partial 草稿"}}"#, b"\x1b\\");

        let mut uninterrupted = NativeScreen::guarded(12, 80);
        uninterrupted.process(&input);
        let expected = uninterrupted.checkpoint().unwrap();
        let shell = uninterrupted.shell().unwrap();
        assert_eq!(shell.prompt.as_ref().unwrap().pwd.as_deref(), Some("/work/项目"));
        assert_eq!(shell.completed.len(), 1);
        assert_eq!(shell.completed[0].command, command);
        assert_eq!(shell.completed[0].exit_code, Some(7));
        assert!(shell.completed[0].end.unwrap().row > shell.completed[0].start.row);
        assert_eq!(shell.active.as_ref().unwrap().id.as_deref(), Some("precmd-42-1"));
        assert_eq!(shell.input_buffer.as_deref(), Some("partial 草稿"));
        for (cut, _) in input.iter().enumerate() {
            let mut first = NativeScreen::guarded(12, 80);
            first.process(&input[..cut]);
            let mut restored = NativeScreen::from_checkpoint(&first.checkpoint().unwrap())
                .unwrap_or_else(|| panic!("restore at byte {cut}"));
            restored.process(&input[cut..]);
            assert_eq!(restored.checkpoint().unwrap(), expected, "cut {cut}");
        }
    }

    #[test]
    fn checkpoint_import_emits_effects_only_for_live_suffix() {
        #[derive(Default)]
        struct Effects { clipboard: usize, bell: usize }
        impl vt100::Callbacks for Effects {
            fn copy_to_clipboard(&mut self, _: &mut vt100::Screen, _: &[u8], _: &[u8]) {
                self.clipboard += 1;
            }
            fn audible_bell(&mut self, _: &mut vt100::Screen) { self.bell += 1; }
        }
        let mut original = vt100::Parser::new_with_callbacks(10, 20, 0, Effects::default());
        original.process(b"\x1b]52;c;eA==\x07\x07visible\x1b]52;c;eQ==");
        assert_eq!(original.callbacks().clipboard, 1);
        assert_eq!(original.callbacks().bell, 1);
        let data = bincode::DefaultOptions::new().serialize(&original.checkpoint()).unwrap();
        let checkpoint = bincode::DefaultOptions::new().deserialize(&data).unwrap();
        let mut restored = vt100::Parser::from_checkpoint(checkpoint, Effects::default(), MAX_OSC_BYTES, 0).unwrap();
        assert_eq!(restored.screen().contents(), "visible");
        assert_eq!(restored.callbacks().clipboard, 0);
        assert_eq!(restored.callbacks().bell, 0);
        restored.process(b"\x07\x07");
        assert_eq!(restored.callbacks().clipboard, 1);
        assert_eq!(restored.callbacks().bell, 1);
    }
}
