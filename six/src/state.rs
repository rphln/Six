use std::fmt::Debug;
use std::ops::RangeBounds;

use crate::buffer::Buffer;
use crate::cursor::{Bounded, Codepoint, Cursor, Head, Iter};

/// An event.
// TODO: Replace this with semantic events.
pub enum Event<'a> {
    Esc,

    Backspace,
    Delete,

    Char(char),

    Ctrl(&'a Event<'a>),
    Meta(&'a Event<'a>),
    Shift(&'a Event<'a>),
}

// TODO: Replace with a trait alias once it stabilizes.
pub trait Callback<Arg>: FnOnce(&mut State, Arg) -> Mode + Send + Sync + 'static {}

impl<Arg, F> Callback<Arg> for F where F: FnOnce(&mut State, Arg) -> Mode + Send + Sync + 'static {}

/// An modal editor.
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct Editor {
    /// The state shared between modes.
    state: State,

    /// The current mode.
    mode: Mode,
}

/// The state state.
#[derive(Debug, Default)]
pub struct State {
    /// The state buffer.
    buffer: Buffer,

    /// The cursor position.
    cursor: Cursor,
}

/// An state mode.
#[derive(Derivative)]
#[derivative(Default, Debug)]
pub enum Mode {
    /// The default editor mode.
    #[derivative(Default)]
    Normal,

    /// The text insertion mode.
    Insert,

    /// Queries the user for a text range.
    Select {
        /// The fixed point of the selection.
        anchor: Cursor,
    },

    /// Queries the user for a text object and applies an operation.
    Operator {
        /// The operator name.
        name: &'static str,

        /// Operator to be executed.
        #[derivative(Debug = "ignore")]
        // TODO: Make the initial position implicit through `State::cursor`.
        and_then: Box<dyn Callback<(Cursor, Cursor)>>,
    },

    /// Queries the user for a text input and applies an operation.
    Query {
        /// The operation name.
        name: &'static str,

        /// The buffer of the query.
        buffer: Buffer,

        /// Cursor position in the buffer.
        cursor: Cursor,

        /// Whether to finish the query.
        #[derivative(Debug = "ignore")]
        until: fn(&str, Event) -> bool,

        /// Function to be called after the input is submitted.
        #[derivative(Debug = "ignore")]
        and_then: Box<dyn for<'a> Callback<&'a str>>,
    },
}

impl Editor {
    /// Returns a reference to the state inner state.
    #[must_use]
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Returns a reference to the state mode.
    #[must_use]
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Advances the state state by handling an event.
    pub fn advance(&mut self, event: Event) {
        self.mode = std::mem::take(&mut self.mode).advance(&mut self.state, event);
    }
}

impl State {
    /// Returns the cursor position.
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Returns a reference to the buffer.
    #[must_use]
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn insert(&mut self, ch: char) {
        self.buffer.insert(self.cursor.index, ch)
    }

    pub fn replace_range(&mut self, range: impl RangeBounds<usize>, text: &str) {
        self.buffer.replace_range(range, text)
    }

    /// Moves the cursor forward up to the specified units according to the specified unit.
    pub fn forward<'a, M: Iter<'a>>(&'a mut self, n: usize) {
        self.cursor = self
            .cursor
            .iter::<M>(&self.buffer)
            .take(n)
            .last()
            .unwrap_or_else(|| Cursor::eof(&self.buffer));
    }

    /// Moves the cursor forward the last position according to the specified unit.
    pub fn last<'a, M: Iter<'a>>(&'a mut self) {
        self.cursor =
            self.cursor.iter::<M>(&self.buffer).last().unwrap_or_else(|| Cursor::eof(&self.buffer));
    }

    /// Moves the cursor backward up to the specified units according to the specified unit.
    pub fn backward<'a, M: Iter<'a>>(&'a mut self, n: usize) {
        self.cursor = self
            .cursor
            .iter::<M>(&self.buffer)
            .rev()
            .take(n)
            .last()
            .unwrap_or_else(|| Cursor::eof(&self.buffer));
    }

    /// Moves the cursor backward the last position according to the specified unit.
    pub fn first<'a, M: Iter<'a>>(&'a mut self) {
        self.cursor = self
            .cursor
            .iter::<M>(&self.buffer)
            .rev()
            .last()
            .unwrap_or_else(|| Cursor::eof(&self.buffer));
    }
}

impl Mode {
    #[must_use]
    pub fn escape() -> Mode {
        Mode::Normal
    }

    #[must_use]
    pub fn insert() -> Mode {
        Mode::Insert
    }

    pub fn operator(name: &'static str, and_then: impl Callback<(Cursor, Cursor)>) -> Mode {
        Mode::Operator { name, and_then: Box::new(and_then) }
    }

    #[must_use]
    pub fn query(
        name: &'static str,
        until: fn(&str, Event) -> bool,
        and_then: impl for<'r> Callback<&'r str>,
    ) -> Mode {
        Mode::Query {
            name,
            until,

            cursor: Cursor::default(),
            buffer: Buffer::default(),

            and_then: Box::new(and_then),
        }
    }

    /// Returns an user-friendly name for the mode.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Mode::Normal { .. } => "Normal",
            Mode::Insert { .. } => "Insert",
            Mode::Select { .. } => "Select",

            Mode::Query { name, .. } => name,
            Mode::Operator { name, .. } => name,
        }
    }

    /// Advances the state state by handling an event.
    pub fn advance(self, state: &mut State, event: Event) -> Mode {
        use Mode::{Insert, Normal, Operator, Query};

        match (self, event) {
            (Normal, Event::Char('i')) => Mode::insert(),
            (Normal, Event::Char('a')) => {
                state.forward::<Codepoint>(1);
                Mode::insert()
            }

            (Normal, Event::Char('I')) => {
                state.first::<Bounded>();
                Mode::insert()
            }

            (Normal, Event::Char('A')) => {
                state.last::<Bounded>();
                Mode::insert()
            }

            (mode @ Insert, Event::Char(ch)) => {
                state.insert(ch);
                state.forward::<Codepoint>(1);
                mode
            }

            (Insert, Event::Esc) => {
                state.backward::<Bounded>(1);
                Mode::escape()
            }

            (Normal, Event::Char(';')) => Mode::query(
                "Eval",
                |_, event| matches!(event, Event::Char('\n')),
                |state: &mut State, program: &str| Mode::default(),
            ),

            (Normal, Event::Char('d')) => {
                Mode::operator("Delete", |state: &mut State, range: (Cursor, Cursor)| {
                    state.replace_range(range.0.index..range.1.index, "");
                    Mode::escape()
                })
            }

            (Operator { and_then, .. }, Event::Char('w')) => {
                let start = state.cursor();
                let end = state
                    .cursor()
                    .next::<Head>(state.buffer())
                    .unwrap_or_else(|| Cursor::eof(state.buffer()));

                and_then(state, (start, end))
            }

            (Operator { and_then, .. }, Event::Char('b')) => {
                let end = state.cursor();
                state.backward::<Head>(1);

                and_then(state, (state.cursor(), end))
            }

            (Query { mut buffer, until, name, mut cursor, and_then }, event) => {
                match event {
                    Event::Char(ch) => {
                        buffer.insert(cursor.index, ch);
                        cursor = cursor
                            .next::<Codepoint>(&buffer)
                            .unwrap_or_else(|| Cursor::eof(&buffer));
                    }

                    _ => (),
                };

                if until(buffer.as_str(), event) {
                    and_then(state, buffer.as_str())
                } else {
                    Query { buffer, until, name, cursor, and_then }
                }
            }

            (mode, _) => mode,
        }
    }
}
