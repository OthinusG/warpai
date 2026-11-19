//! Memory-only screen evidence for Companion-owned PTYs; never logged or serialized.
use super::activity_with_progress;
use crate::{readiness::Activity, transport::ScreenContext};

const MAX_OSC_BYTES: usize = 4096 + 16;

#[derive(Default)]
struct Signals { title: String, progress: String }

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
}

enum OscInput {
    Reading(Vec<u8>),
    Discarding,
}

pub(crate) struct NativeScreen {
    parser: vt100::Parser<Signals>,
    escape_pending: bool,
    osc: Option<OscInput>,
}
impl NativeScreen {
    pub(crate) fn cli_view(&self, previous_alternate: bool) -> Vec<u8> {
        let mut view = Vec::new();
        if self.alternate_screen() && !previous_alternate {
            view.extend_from_slice(b"\x1b[?1049h");
        } else if previous_alternate && !self.alternate_screen() {
            view.extend_from_slice(b"\x1b[?1049l");
        }
        view.extend(self.parser.screen().state_formatted());
        view
    }

    pub(crate) fn alternate_screen(&self) -> bool {
        self.parser.screen().alternate_screen()
    }

    pub(crate) fn new(rows: u16, columns: u16) -> Self {
        Self {
            parser: vt100::Parser::new_with_callbacks(rows, columns, 0, Signals::default()),
            escape_pending: false,
            osc: None,
        }
    }
    pub(crate) fn process(&mut self, mut bytes: &[u8]) {
        // vte's std parser accumulates an OSC until termination. Bound the
        // readiness projection while the native output stream stays intact.
        while !bytes.is_empty() {
            if let Some(osc) = &mut self.osc {
                let end = bytes.iter().position(|&byte| matches!(byte, 0x07 | 0x18 | 0x1a | 0x1b));
                let payload = &bytes[..end.unwrap_or(bytes.len())];
                if let OscInput::Reading(buffer) = osc {
                    if payload.len() <= MAX_OSC_BYTES - buffer.len() {
                        buffer.extend_from_slice(payload);
                    } else {
                        *osc = OscInput::Discarding;
                    }
                }
                let Some(end) = end else { return; };
                if let Some(OscInput::Reading(payload)) = self.osc.take() {
                    self.parser.process(&payload);
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
}
