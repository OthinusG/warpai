//! Import a verified remote display without dispatching any terminal sequences.
use super::grid_handler::TermMode;
use super::Cursor;
use crate::terminal::model::ansi::{Color, NamedColor};
use crate::terminal::model::cell::{Cell, Flags};
use crate::terminal::model::index::{VisiblePoint, VisibleRow};
use warp_agent_bus::ssh_remote::{
    MouseProtocolEncoding, MouseProtocolMode, TerminalCell, TerminalColor, TerminalGrid, TerminalRow,
    TerminalScreen,
};
use warpui::color::ColorU;

fn terminal_color(color: TerminalColor, default: NamedColor) -> Color {
    match color {
        TerminalColor::Default => Color::Named(default),
        TerminalColor::Idx(index) => Color::Indexed(index),
        TerminalColor::Rgb(red, green, blue) => Color::Spec(ColorU::new(red, green, blue, 255)),
    }
}

fn terminal_cell(source: &TerminalCell) -> Cell {
    let mut cell = Cell::default();
    cell.fg = terminal_color(source.fgcolor(), NamedColor::Foreground);
    cell.bg = terminal_color(source.bgcolor(), NamedColor::Background);
    cell.flags.set(Flags::BOLD, source.bold());
    cell.flags.set(Flags::DIM, source.dim());
    cell.flags.set(Flags::ITALIC, source.italic());
    cell.flags.set(Flags::UNDERLINE, source.underline());
    cell.flags.set(Flags::INVERSE, source.inverse());
    cell.flags.set(Flags::WIDE_CHAR, source.is_wide());
    cell.flags.set(Flags::WIDE_CHAR_SPACER, source.is_wide_continuation());
    let mut contents = source.contents().chars();
    if let Some(character) = contents.next() {
        cell.c = character;
        for character in contents {
            cell.push_zerowidth(character, false);
        }
    }
    cell
}

pub(super) fn terminal_row(source: Option<TerminalRow<'_>>, columns: u16) -> super::row::Row {
    let mut row = super::row::Row::new(usize::from(columns));
    if let Some(source) = source {
        for column in 0..columns {
            if let Some(cell) = source.cell(column) {
                row[usize::from(column)] = terminal_cell(cell);
            }
        }
        if source.wrapped() {
            row[usize::from(columns - 1)].flags.insert(Flags::WRAPLINE);
        }
    }
    row
}

pub(super) fn restored_cursor(
    (row, column): (u16, u16),
    dimensions: (u16, u16),
    template: &TerminalCell,
) -> Cursor {
    let (rows, columns) = dimensions;
    Cursor {
        point: VisiblePoint {
            row: VisibleRow(usize::from(row.min(rows - 1))),
            col: usize::from(column.min(columns - 1)),
        },
        input_needs_wrap: column >= columns,
        template: terminal_cell(template),
        ..Default::default()
    }
}

pub(super) fn display_modes(screen: &TerminalScreen, source: &TerminalGrid<'_>) -> TermMode {
    let mut mode = TermMode::LINE_WRAP;
    mode.set(TermMode::SHOW_CURSOR, !screen.hide_cursor());
    mode.set(TermMode::APP_CURSOR, screen.application_cursor());
    mode.set(TermMode::APP_KEYPAD, screen.application_keypad());
    mode.set(TermMode::BRACKETED_PASTE, screen.bracketed_paste());
    mode.set(TermMode::ORIGIN, source.cursor_origin_modes().0);
    mode.insert(match screen.mouse_protocol_mode() {
        MouseProtocolMode::None => TermMode::NONE,
        MouseProtocolMode::Press | MouseProtocolMode::PressRelease => TermMode::MOUSE_REPORT_CLICK,
        MouseProtocolMode::ButtonMotion => TermMode::MOUSE_DRAG,
        MouseProtocolMode::AnyMotion => TermMode::MOUSE_MOTION,
    });
    mode.insert(match screen.mouse_protocol_encoding() {
        MouseProtocolEncoding::Default => TermMode::NONE,
        MouseProtocolEncoding::Utf8 => TermMode::UTF8_MOUSE,
        MouseProtocolEncoding::Sgr => TermMode::SGR_MOUSE,
    });
    mode
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::model::grid::grid_handler::GridHandler;
    use crate::terminal::model::grid::Dimensions;
    use warp_agent_bus::ssh_remote::TerminalCheckpoint;

    #[test]
    fn verified_checkpoint_imports_primary_history_and_alternate_cells() {
        let output = concat!(
            "earlier row\r\nsecond row\r\nthird row\r\n",
            "\x1b[1;1H\x1b[38;2;20;40;60m中e\u{301}\x1b7",
            "\x1b]52;c;aGlzdG9yaWNhbA==\x07",
            "\x1b[?1049h\x1b[?2004h\x1b[?25l",
            "\x1b[2;2Halternate",
        );
        let checkpoint = TerminalCheckpoint::from_fixture(3, 12, output.as_bytes()).unwrap();
        let screen = checkpoint.screen();
        let mut primary = GridHandler::new_for_test_with_scroll_limit(3, 12, 128);
        let mut alternate = GridHandler::new_for_alt_screen_test(3, 12);
        assert!(primary.install_guarded_grid(screen, screen.primary_grid()));
        assert!(alternate.install_guarded_grid(screen, screen.alternate_grid()));

        assert_eq!(primary.history_size(), 1);
        let historical = primary.row(0).unwrap();
        assert_eq!(historical[0].c, 'e');
        let visible = primary.row(primary.history_size()).unwrap();
        assert_eq!(visible[0].c, '中');
        assert_eq!(visible[0].fg, Color::Spec(ColorU::new(20, 40, 60, 255)));
        assert!(visible[1].flags.contains(Flags::WIDE_CHAR_SPACER));
        assert_eq!(visible[2].raw_content().to_string(), "e\u{301}");
        assert_eq!(primary.grid_storage().saved_cursor.point.col, 3);
        assert_eq!(alternate.row(1).unwrap()[1].c, 'a');
        assert_eq!(alternate.history_size(), 0);
        assert!(alternate.needs_bracketed_paste());
        assert!(!alternate.is_mode_set(TermMode::SHOW_CURSOR));

        let mut different_size = GridHandler::new_for_test(4, 12);
        assert!(!different_size.install_guarded_grid(screen, screen.primary_grid()));
        assert_eq!(different_size.history_size(), 0);
    }
}
