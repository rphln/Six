use std::collections::Bound::{self, *};

use std::collections::HashMap;
use std::ops::RangeBounds;
use std::rc::Rc;

use termion::event::Key;

use crate::buffer::Buffer;
use crate::cursor::{unwrap, Cursor};

#[derive(Clone)]
pub struct State {
    /// Current content.
    pub buffer: Rc<String>,

    /// Cursor position in the content.
    cursor: Cursor,

    /// Current mode.
    pub mode: Mode,

    /// Editor registers.
    registers: HashMap<Register, RegisterType>,
}

#[derive(Clone)]
pub enum Mode {
    /// The default editor mode.
    Normal { count: Option<usize> },

    /// Text input mode.
    Edit,

    /// Queries the user for a text range.
    Select { anchor: Cursor },

    /// Queries the user for a text object and applies an operation.
    Operator {
        prompt: Option<String>,
        count: Option<usize>,

        /// Callback invoked once a text object has been provided.
        // Why can't I just do `Box<dyn RangeBound<Cursor>>` or something like it.
        and_then: Rc<dyn Fn(State, (Bound<Cursor>, Bound<Cursor>)) -> State>,
    },

    /// Queries the user for a text input and applies an operation.
    Query {
        prompt: Option<String>,
        length: Option<usize>,

        /// Callback invoked once the input has finished.
        and_then: Rc<dyn Fn(State, String) -> State>,
    },
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Register {
    /// Special register used to store how many times an action should be repeated.
    Repeat,

    /// Special register used to store the result of a query made to the user.
    Query,

    /// Special register used internally to store the initial position of a text object.
    EditStart,

    /// Special register used internally to store the final position of a text object.
    EditEnd,

    /// An user-defined register.
    Other(&'static str),
}

#[derive(Clone)]
pub enum RegisterType {
    Text(String),
    Number(usize),
    Mark(Cursor),
}

macro_rules! cursor_delegate_unwrap {
    ( $name:ident  ) => {
        pub fn $name(mut self, count: usize) -> Self {
            self.cursor = unwrap(self.cursor.$name(count, self.buffer.as_ref()));
            self
        }
    };
}

impl State {
    pub fn col(&self) -> usize {
        self.cursor.col(self.buffer.as_ref())
    }

    pub fn row(&self) -> usize {
        self.cursor.row(self.buffer.as_ref())
    }

    pub fn insert(mut self, at: Cursor, ch: char) -> Self {
        Buffer::insert(Rc::make_mut(&mut self.buffer), at, ch);
        self
    }

    pub fn delete<B>(mut self, range: B) -> Self
    where
        B: RangeBounds<Cursor>,
    {
        Buffer::delete(Rc::make_mut(&mut self.buffer), range);
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
        self.cursor = self.cursor.bol(self.buffer.as_ref());
        self
    }

    pub fn eol(mut self) -> Self {
        self.cursor = self.cursor.eol(self.buffer.as_ref());
        self
    }

    cursor_delegate_unwrap!(down);
    cursor_delegate_unwrap!(up);
    cursor_delegate_unwrap!(right);
    cursor_delegate_unwrap!(left);
    cursor_delegate_unwrap!(forward);
    cursor_delegate_unwrap!(backward);
    cursor_delegate_unwrap!(forward_words);
    cursor_delegate_unwrap!(backward_words);

    pub fn consume_count(&mut self, name: Register) -> Option<usize> {
        self.registers.remove(&name).and_then(|register| {
            if let RegisterType::Number(n) = register {
                Some(n)
            } else {
                None
            }
        })
    }

    pub fn consume_mark(&mut self, name: Register) -> Option<Cursor> {
        self.registers.remove(&name).and_then(|register| {
            if let RegisterType::Mark(n) = register {
                Some(n)
            } else {
                None
            }
        })
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            buffer: Rc::new(String::default()),
            cursor: Cursor::new(0, 0),
            registers: HashMap::new(),
            mode: Mode::Normal { count: None },
        }
    }
}

pub fn event_loop(state: State, event: Key) -> Option<State> {
    use Key::{Backspace, Char, Ctrl, Delete, Esc};
    use Mode::{Edit, Normal, Operator, Query, Select};

    if matches!(event, Ctrl('d')) {
        return None;
    }

    Some(match (&state.mode, event) {
        (Edit, Esc) => state.mode(Normal { count: None }).left(1),

        (Normal { mut count }, Char(ch)) | (Operator { mut count, .. }, Char(ch))
            if ch.is_digit(10) =>
        {
            let k = ch.to_digit(10).expect("digit") as usize;
            count = count.map(|n| n * 10 + k).or(Some(k));

            state
        }

        (_, Esc) | (_, Ctrl('c')) => state.mode(Normal { count: None }),

        (Normal { .. }, Char('h')) | (Select { .. }, Char('h')) | (_, Key::Left) => state.left(1),
        (Normal { .. }, Char('j')) | (Select { .. }, Char('j')) | (_, Key::Down) => state.down(1),
        (Normal { .. }, Char('k')) | (Select { .. }, Char('k')) | (_, Key::Up) => state.up(1),
        (Normal { .. }, Char('l')) | (Select { .. }, Char('l')) | (_, Key::Right) => state.right(1),

        (Normal { .. }, Char('i')) => state.mode(Edit),
        (Normal { .. }, Char('a')) => state.mode(Edit).right(1),

        (Normal { .. }, Char('I')) => state.bol().mode(Edit),
        (Normal { .. }, Char('A')) => state.eol().mode(Edit),

        (Normal { .. }, Char('o')) => state.eol().insert_at_cursor('\n').forward(1).mode(Edit),
        (Normal { .. }, Char('O')) => state.bol().insert_at_cursor('\n').mode(Edit),

        // (Normal { .. }, Char('W')) => {
        //     let count = state.consume_count(Register::Repeat).unwrap_or(1);

        //     state.cursor = cursor::unwrap(state.cursor.forward_words(count, &state.buffer));

        //     state
        // }
        (Normal { .. }, Char('d')) => state.mode(Operator {
            prompt: Some("Delete".to_string()),
            count: None,
            and_then: Rc::new(State::delete),
        }),

        (Operator { and_then, count, .. }, Char('W')) => {
            let end = unwrap(state.cursor.forward_words(count.unwrap_or(1), state.buffer.as_ref()));

            and_then(state.clone(), (Included(state.cursor), Excluded(end)))
        }

        // (Normal { .. }, Char('B')) => {
        //     let count = state.consume_count(COUNT_REGISTER).unwrap_or(1);
        //     state.cursor = cursor::unwrap(state.cursor.backward_words(count, &state.buffer));

        //     state
        // }
        (Edit, Char(ch)) => state.insert_at_cursor(ch).forward(1),

        (Edit, Key::Backspace) => {
            let mut state = state.backward(1);

            if state.buffer.get(state.cursor).is_some() {
                Buffer::delete(Rc::make_mut(&mut state.buffer), state.cursor..=state.cursor)
            }

            state
        }

        // (Edit, Key::Delete) => {
        //     if state.buffer.get(state.cursor).is_some() {
        //         state.buffer.delete(state.cursor..=state.cursor);
        //     }

        //     state
        // }

        // (Edit, Ctrl('w')) => {
        //     let anchor = state.cursor;
        //     state.cursor = state
        //         .cursor
        //         .backward_words(1, &state.buffer)
        //         .unwrap_or_else(|err| match err {
        //             cursor::ErrorKind::Interrupted { at, .. } => at,
        //         });

        //     state.buffer.delete(state.cursor..anchor);
        //     state
        // }
        _ => state,
    })
}
