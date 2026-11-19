/// A parser for terminal output which produces an in-memory representation of
/// the terminal contents.
pub struct Parser<CB: crate::callbacks::Callbacks = ()> {
    parser: vte::Parser,
    screen: crate::perform::WrappedScreen<CB>,
}

/// Portable parser and screen state, independent of process-local callbacks.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Checkpoint<'a> {
    decoder: vte::Checkpoint,
    screen: crate::screen::ScreenCheckpoint<'a>,
}

impl Parser {
    /// Creates a new terminal parser of the given size and with the given
    /// amount of scrollback.
    #[must_use]
    pub fn new(rows: u16, cols: u16, scrollback_len: usize) -> Self {
        Self {
            parser: vte::Parser::new(),
            screen: crate::perform::WrappedScreen::new(
                rows,
                cols,
                scrollback_len,
            ),
        }
    }
}

impl<CB: crate::callbacks::Callbacks> Parser<CB> {
    /// Capture both buffers and decoder state without calling terminal effects.
    pub fn checkpoint(&self) -> Checkpoint<'_> {
        Checkpoint { decoder: self.parser.checkpoint(), screen: self.screen.screen.checkpoint() }
    }

    /// Restore a validated checkpoint without feeding historical bytes to callbacks.
    pub fn from_checkpoint(checkpoint: Checkpoint<'_>, callbacks: CB, max_osc_bytes: usize, max_history: usize) -> Option<Self> {
        Some(Self {
            parser: vte::Parser::from_checkpoint(checkpoint.decoder, max_osc_bytes)?,
            screen: crate::perform::WrappedScreen {
                screen: crate::screen::Screen::from_checkpoint(checkpoint.screen, max_history)?,
                callbacks,
            },
        })
    }

    /// Creates a new terminal parser of the given size and with the given
    /// amount of scrollback. Terminal events will be reported via method
    /// calls on the provided [`Callbacks`](crate::callbacks::Callbacks)
    /// implementation.
    pub fn new_with_callbacks(
        rows: u16,
        cols: u16,
        scrollback_len: usize,
        callbacks: CB,
    ) -> Self {
        Self {
            parser: vte::Parser::new(),
            screen: crate::perform::WrappedScreen::new_with_callbacks(
                rows,
                cols,
                scrollback_len,
                callbacks,
            ),
        }
    }

    /// Processes the contents of the given byte string, and updates the
    /// in-memory terminal state.
    pub fn process(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut self.screen, bytes);
    }

    /// Returns a reference to a [`Screen`](crate::Screen) object containing
    /// the terminal state.
    #[must_use]
    pub fn screen(&self) -> &crate::Screen {
        &self.screen.screen
    }

    /// Returns a mutable reference to a [`Screen`](crate::Screen) object
    /// containing the terminal state.
    #[must_use]
    pub fn screen_mut(&mut self) -> &mut crate::Screen {
        &mut self.screen.screen
    }

    /// Returns a reference to the [`Callbacks`](crate::callbacks::Callbacks)
    /// state object passed into the constructor.
    pub fn callbacks(&self) -> &CB {
        &self.screen.callbacks
    }

    /// Returns a mutable reference to the
    /// [`Callbacks`](crate::callbacks::Callbacks) state object passed into
    /// the constructor.
    pub fn callbacks_mut(&mut self) -> &mut CB {
        &mut self.screen.callbacks
    }

    /// Whether the next byte starts at a complete UTF-8 and terminal boundary.
    pub fn at_input_boundary(&self) -> bool {
        self.parser.at_input_boundary()
    }

    /// Rewrite the pending prefix without dispatching any historical action.
    pub fn incomplete_prefix(&self) -> Option<Vec<u8>> {
        self.parser.incomplete_prefix()
    }
}

impl Default for Parser {
    /// Returns a parser with dimensions 80x24 and no scrollback.
    fn default() -> Self {
        Self::new(24, 80, 0)
    }
}

impl std::io::Write for Parser {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.process(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
