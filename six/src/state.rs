use std::fmt::Debug;

use crate::buffer::Buffer;
use crate::cursor::{Bounded, Codepoint, Cursor, Iter};

/// An event.
// TODO: Replace this with a bitmask.
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
pub trait Callback<Arg>: FnOnce(Mode, &mut State, Arg) -> Mode + Send + Sync {}

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

    /// Returns an user-friendly name for the mode.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Mode::Normal { .. } => "Normal",
            Mode::Insert { .. } => "Insert",
            Mode::Select { .. } => "Select",
        }
    }

    /// Advances the state state by handling an event.
    pub fn advance(self, state: &mut State, event: Event) -> Mode {
        use Mode::{Insert, Normal};

        match (self, event) {
            (Normal, Event::Char('i')) => Mode::insert(),
            (Normal, Event::Char('a')) => {
                state.forward::<Codepoint>(1);
                Mode::insert()
            },

            (Normal, Event::Char('I')) => {
                state.first::<Bounded>();
                Mode::insert()
            },

            (Normal, Event::Char('A')) => {
                state.last::<Bounded>();
                Mode::insert()
            },

            (mode @ Insert, Event::Char(ch)) => {
                state.insert(ch);
                state.forward::<Codepoint>(1);
                mode
            },

            (Insert, Event::Esc) => {
                state.backward::<Bounded>(1);
                Mode::escape()
            },

            _ => todo!(),
        }
    }
}

// pub enum Mode {

//     /// Queries the user for a text object and applies an operation.
//     Pending {
//         /// The prompt displayed to the user.
//         prompt: &'static str,

//         /// Operator to be executed.
//         name: &'static str,
//     },

//     /// Applies an operation over a queried text object.
//     Operator {
//         /// Initial position of the text object.
//         start: Cursor,

//         /// Final position of the text object.
//         end: Cursor,

//         /// Operator to be executed.
//         name: &'static str,
//     },

//     /// Queries the user for a text input and applies an operation.
//     Querying {
//         /// The prompt displayed to the user.
//         prompt: &'static str,

//         /// The maximum length of the queried string.
//         length: Option<usize>,

//         /// The buffer of the query.
//         buffer: Buffer,

//         /// Cursor position in the buffer.
//         cursor: Cursor,

//         /// Operation to be executed.
//         name: &'static str,
//     },

//     /// Applies an operation over a queried text.
//     Query {
//         /// The buffer of the query.
//         content: Buffer,

//         /// Operation to be executed.
//         name: &'static str,
//     },
// }
