use std::collections::HashMap;
use std::io;

use std::fmt;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use six::buffer::Buffer;
use six::cursor;
use six::cursor::Cursor;

const QUERY_REGISTER: &'static str = "=";
const COUNT_REGISTER: &'static str = "#";

const EDIT_START_REGISTER: &'static str = "<";
const EDIT_END_REGISTER: &'static str = ">";

#[derive(Debug, Clone)]
struct State {
    buffer: String,
    cursor: Cursor,
    mode: Mode,
    registers: HashMap<&'static str, Register>,
}

type Transform = fn(State) -> State;

#[derive(Clone)]
enum Mode {
    /// The default editor mode.
    Normal,

    /// Text input mode.
    Insert,

    /// Queries the user for a text range.
    Select { anchor: Cursor },

    /// Queries the user for a text object and applies an operation.
    Operator {
        prompt: Option<String>,
        and_then: Transform,
    },

    /// Queries the user for a text input and applies an operation.
    Query {
        prompt: Option<String>,
        length: Option<usize>,
        and_then: Transform,
    },
}

impl fmt::Debug for Mode {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Mode::Operator { .. } => "Operator",
            Mode::Query { .. } => "Query",
            Mode::Select { .. } => "Select",
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
        };

        write!(formatter, "{}", name)
    }
}

#[derive(Debug, Clone)]
enum Register {
    Text(String),
    Number(usize),
    Mark(Cursor),
}

impl From<usize> for Register {
    fn from(n: usize) -> Register {
        Register::Number(n)
    }
}

impl From<String> for Register {
    fn from(n: String) -> Register {
        Register::Text(n)
    }
}

impl From<Cursor> for Register {
    fn from(n: Cursor) -> Register {
        Register::Mark(n)
    }
}

impl State {
    pub fn col(&self) -> usize {
        self.cursor.col(&self.buffer)
    }

    pub fn row(&self) -> usize {
        self.cursor.row(&self.buffer)
    }

    pub fn insert(mut self, at: Cursor, ch: char) -> Self {
        Buffer::insert(&mut self.buffer, at, ch);
        self
    }

    pub fn insert_at_cursor(self, ch: char) -> Self {
        let cursor = self.cursor;
        self.insert(cursor, ch)
    }

    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    pub fn bol(mut self) -> Self {
        self.cursor = self.cursor.bol(&self.buffer);
        self
    }

    pub fn eol(mut self) -> Self {
        self.cursor = self.cursor.eol(&self.buffer);
        self
    }

    pub fn down(mut self, count: usize) -> Self {
        self.cursor = self
            .cursor
            .down(count, &self.buffer)
            .unwrap_or_else(|err| match err {
                cursor::ErrorKind::Interrupted { at, .. } => at,
            });
        self
    }

    pub fn up(mut self, count: usize) -> Self {
        self.cursor = self
            .cursor
            .up(count, &self.buffer)
            .unwrap_or_else(|err| match err {
                cursor::ErrorKind::Interrupted { at, .. } => at,
            });
        self
    }

    pub fn right(mut self, count: usize) -> Self {
        self.cursor = self
            .cursor
            .right(count, &self.buffer)
            .unwrap_or_else(|err| match err {
                cursor::ErrorKind::Interrupted { at, .. } => at,
            });
        self
    }

    pub fn left(mut self, count: usize) -> Self {
        self.cursor = self
            .cursor
            .left(count, &self.buffer)
            .unwrap_or_else(|err| match err {
                cursor::ErrorKind::Interrupted { at, .. } => at,
            });
        self
    }

    pub fn forward(mut self, count: usize) -> Self {
        self.cursor = self
            .cursor
            .forward(count, &self.buffer)
            .unwrap_or_else(|err| match err {
                cursor::ErrorKind::Interrupted { at, .. } => at,
            });

        self
    }

    pub fn count(&self) -> Option<usize> {
        match self.registers.get(COUNT_REGISTER) {
            Some(&Register::Number(n)) => Some(n),
            _ => None,
        }
    }

    pub fn consume_count(&mut self, name: &str) -> Option<usize> {
        self.registers.remove(name).and_then(|register| {
            if let Register::Number(n) = register {
                Some(n)
            } else {
                None
            }
        })
    }

    pub fn consume_mark(&mut self, mark: &str) -> Option<Cursor> {
        self.registers
            .remove(mark)
            .and_then(|register| match register {
                Register::Mark(mark) => Some(mark),
                _ => None,
            })
    }
}

fn event_loop(mut state: State, event: Key) -> Option<State> {
    use Key::{Char, Ctrl};
    use Mode::*;

    if matches!(event, Ctrl('d')) {
        return None;
    }

    Some(match (&state.mode, event) {
        (Insert, Key::Esc) => state.mode(Normal).left(1),

        (_, Key::Esc) => state.mode(Normal),

        (Normal, Char('h')) | (Select { .. }, Char('h')) | (_, Key::Left) => state.left(1),
        (Normal, Char('j')) | (Select { .. }, Char('j')) | (_, Key::Down) => state.down(1),
        (Normal, Char('k')) | (Select { .. }, Char('k')) | (_, Key::Up) => state.up(1),
        (Normal, Char('l')) | (Select { .. }, Char('l')) | (_, Key::Right) => state.right(1),

        (Normal, Char('i')) => state.mode(Insert),
        (Normal, Char('a')) => state.mode(Insert).right(1),

        (Normal, Char('I')) => state.bol().mode(Insert),
        (Normal, Char('A')) => state.eol().mode(Insert),

        (Normal, Char('o')) => state.eol().insert_at_cursor('\n').forward(1).mode(Insert),
        (Normal, Char('O')) => state.bol().insert_at_cursor('\n').mode(Insert),

        (Normal, Char('W')) => {
            let count = state.consume_count(COUNT_REGISTER).unwrap_or(1);

            state.cursor = cursor::unwrap(state.cursor.forward_words(count, &state.buffer));

            state
        }

        (Normal, Char('d')) => state.mode(Operator {
            prompt: Some("Delete".to_string()),
            and_then: |mut state| {
                let start = state.consume_mark(EDIT_START_REGISTER).expect("start");
                let end = state.consume_mark(EDIT_END_REGISTER).expect("end");

                state.buffer.delete(start..end);
                state
            },
        }),

        (Operator { and_then, .. }, Char('W')) => {
            let count = state.count().unwrap_or(1);

            let end = state
                .cursor
                .forward_words(count, &state.buffer)
                .unwrap_or_else(|err| match err {
                    cursor::ErrorKind::Interrupted { at, .. } => at,
                });

            state
                .registers
                .insert(EDIT_START_REGISTER, Register::Mark(state.cursor));
            state
                .registers
                .insert(EDIT_END_REGISTER, Register::Mark(end));

            and_then(state)
        }

        (Normal, Char('B')) => {
            let count = state.consume_count(COUNT_REGISTER).unwrap_or(1);
            state.cursor = cursor::unwrap(state.cursor.backward_words(count, &state.buffer));

            state
        }

        (Insert, Char(ch)) => state.insert_at_cursor(ch).forward(1),

        (Insert, Key::Backspace) => {
            state.cursor = cursor::unwrap(state.cursor.backward(1, &state.buffer));

            if state.buffer.get(state.cursor).is_some() {
                state.buffer.delete(state.cursor..=state.cursor);
            }

            state
        }

        (Insert, Key::Delete) => {
            if state.buffer.get(state.cursor).is_some() {
                state.buffer.delete(state.cursor..=state.cursor);
            }

            state
        }

        (Insert, Ctrl('w')) => {
            let anchor = state.cursor;
            state.cursor = state
                .cursor
                .backward_words(1, &state.buffer)
                .unwrap_or_else(|err| match err {
                    cursor::ErrorKind::Interrupted { at, .. } => at,
                });

            state.buffer.delete(state.cursor..anchor);
            state
        }

        (Normal, Char(ch)) | (Operator { .. }, Char(ch)) if ch.is_digit(10) => {
            let k = ch.to_digit(10).expect("digit") as usize;

            state.registers.insert(COUNT_REGISTER, {
                state.count().map(|n| n * 10 + k).unwrap_or(k).into()
            });

            state
        }

        _ => state,
    })
}

fn main() -> Result<(), io::Error> {
    #[allow(unused_variables)]
    let stdout = io::stdout().into_raw_mode()?;

    let state = State {
        buffer: String::new(),
        cursor: Cursor::default(),
        mode: Mode::Normal,
        registers: HashMap::new(),
    };

    io::stdin().keys().try_fold(state, |state, key| {
        event_loop(state, key.expect("key")).map(|state| {
            print!("{:?}\r\n", state);
            state
        })
    });

    Ok(())
}
