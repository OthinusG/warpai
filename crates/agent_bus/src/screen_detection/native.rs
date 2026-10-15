//! Memory-only screen evidence for Companion-owned PTYs; never logged or serialized.
use super::activity_with_progress;
use crate::{readiness::Activity, transport::ScreenContext};

#[derive(Default)]
struct Signals { title: String, progress: String }

impl vt100::Callbacks for Signals {
    fn set_window_title(&mut self, _: &mut vt100::Screen, title: &[u8]) {
        if title.len() <= 4096 { self.title = String::from_utf8_lossy(title).into_owned(); }
    }
    fn unhandled_osc(&mut self, _: &mut vt100::Screen, params: &[&[u8]]) {
        if params.first() == Some(&b"9".as_slice()) && params.get(1) == Some(&b"4".as_slice()) {
            self.progress = params[1..].iter().filter_map(|part| std::str::from_utf8(part).ok())
                .collect::<Vec<_>>().join(";").chars().take(256).collect();
        }
    }
}

pub(crate) struct NativeScreen(vt100::Parser<Signals>);
impl NativeScreen {
    pub(crate) fn new(rows: u16, columns: u16) -> Self {
        Self(vt100::Parser::new_with_callbacks(rows, columns, 0, Signals::default()))
    }
    pub(crate) fn process(&mut self, bytes: &[u8]) { self.0.process(bytes); }
    pub(crate) fn resize(&mut self, rows: u16, columns: u16) { self.0.screen_mut().set_size(rows, columns); }
    pub(crate) fn bracketed_paste(&self) -> bool { self.0.screen().bracketed_paste() }
    pub(crate) fn activity(&self, context: &ScreenContext) -> Option<Activity> {
        let screen = self.0.screen();
        let (rows, columns) = screen.size();
        let start = rows.saturating_sub(60);
        let text = screen.rows(0, columns).skip(usize::from(start)).collect::<Vec<_>>().join("\n");
        let cursor = screen.cursor_position().0;
        activity_with_progress(&context.program, &text, &self.0.callbacks().title,
            &self.0.callbacks().progress, context.quiet(),
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
        assert_eq!(screen.0.callbacks().title, "✦ 🐋 原生会话");
        screen.process(b"old transcript\x1b[?1049h\x1b[2J\x1b[HAllow once\r\nDeny");
        assert_eq!(screen.0.screen().contents(), "Allow once\nDeny");
        screen.process(b"\x1b[2J\x1b[H\x1b]9;4;1;-1\x07");
        assert_eq!(screen.0.callbacks().progress, "4;1;-1");
        screen.resize(30, 100);
        assert_eq!(screen.0.screen().size(), (30, 100));
        screen.process(b"\x1b[?1049l");
        assert!(screen.0.screen().contents().contains("old transcript"));
    }
}
