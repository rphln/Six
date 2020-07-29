use std::collections::Bound::{self, *};

use std::ops::RangeBounds;
use std::rc::Rc;

use termion::event::Key;

use crate::buffer::{Buffer, Lines};
use crate::cursor::Cursor;

type Range = (Cursor, Cursor);

#[derive(Debug, Clone)]
pub struct State<Buf: Buffer> {
    /// Current content.
    buffer: Buf,

    /// Cursor position in the content.
    cursor: Cursor,

    /// Current mode.
    mode: Mode<Buf>,
}

#[derive(Clone)]
pub struct Callback<Buf: Buffer, P>(Rc<dyn Fn(State<Buf>, P) -> State<Buf>>);

use std::fmt;

impl<B: Buffer, P> fmt::Debug for Callback<B, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Callback").finish()
    }
}

impl<Buf: Buffer, P> Callback<Buf, P> {
    pub fn new<C>(callback: C) -> Callback<Buf, P>
    where
        C: Fn(State<Buf>, P) -> State<Buf> + 'static,
    {
        Self(Rc::new(callback))
    }
}

#[derive(Debug, Clone)]
pub enum Mode<Buf: Buffer> {
    /// The default editor mode.
    Normal { count: Option<usize> },

    /// Text input mode.
    Edit,

    /// Queries the user for a text range.
    Select { anchor: Cursor, count: Option<usize> },

    /// Queries the user for a text object and applies an operation.
    Operator {
        prompt: Option<String>,
        count: Option<usize>,

        /// Callback invoked once a text object has been provided.
        and_then: Callback<Buf, Range>,
    },

    /// Queries the user for a text input and applies an operation.
    Query {
        prompt: Option<String>,

        /// Length of the query before the callback is invoked. If `None`,
        /// invokes on `Return`.
        length: Option<usize>,

        /// Partial buffer for the query.
        partial: String,

        /// Callback invoked once the input has finished.
        and_then: Callback<Buf, String>,
    },
}

impl<Buf: Buffer> Mode<Buf> {
    pub fn set_count(&mut self, next: Option<usize>) {
        match self {
            Mode::Operator { ref mut count, .. }
            | Mode::Normal { ref mut count, .. }
            | Mode::Select { ref mut count, .. } => *count = next,

            _ => panic!("Tried to set count for an incompatible mode"),
        }
    }
}

macro_rules! cursor_delegate_unwrap {
    ( $name:ident ) => {
        pub fn $name(&mut self, count: usize) {
            self.cursor = self.cursor.$name(count, &self.buffer).unwrap_or_else(|err| err.at);
        }
    };
}

impl<Buf: Buffer> State<Buf> {
    pub fn with_buffer(buffer: Buf) -> Self {
        State { buffer, cursor: Cursor::new(0, 0), mode: Mode::Normal { count: None } }
    }

    pub fn col(&self) -> usize {
        self.cursor.col(&self.buffer)
    }

    pub fn row(&self) -> usize {
        self.cursor.row(&self.buffer)
    }

    pub fn mode(&self) -> &Mode<Buf> {
        &self.mode
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn lines(&self) -> Lines {
        self.buffer.lines()
    }

    pub fn insert(&mut self, at: Cursor, ch: char) {
        Buffer::insert(&mut self.buffer, at, ch);
    }

    pub fn delete<R>(&mut self, range: R)
    where
        R: RangeBounds<Cursor>,
    {
        Buffer::delete(&mut self.buffer, range);
    }

    pub fn insert_at_cursor(&mut self, ch: char) {
        let cursor = self.cursor;
        self.insert(cursor, ch);
    }

    pub fn set_mode(&mut self, mode: Mode<Buf>) {
        self.mode = mode;
    }

    pub fn bol(&mut self) {
        self.cursor = self.cursor.bol(&self.buffer);
    }

    pub fn eol(&mut self) {
        self.cursor = self.cursor.eol(&self.buffer);
    }

    cursor_delegate_unwrap!(down);
    cursor_delegate_unwrap!(up);
    cursor_delegate_unwrap!(right);
    cursor_delegate_unwrap!(left);
    cursor_delegate_unwrap!(forward);
    cursor_delegate_unwrap!(backward);
    cursor_delegate_unwrap!(forward_words);
    cursor_delegate_unwrap!(backward_words);
}

pub fn event_loop<Buf: Buffer>(state: &mut State<Buf>, event: Key) -> Option<&mut State<Buf>> {
    use Key::{Backspace, Char, Ctrl, Delete, Esc};
    use Mode::{Edit, Normal, Operator, Query, Select};

    if matches!(event, Ctrl('d')) {
        return None;
    }

    match (&state.mode, event) {
        (Edit, Esc) => {
            state.set_mode(Normal { count: None });
            state.left(1);
        }

        (Normal { count }, Char(ch @ '0'..='9'))
        | (Operator { count, .. }, Char(ch @ '0'..='9')) => {
            let delta = ch.to_digit(10).unwrap() as usize;
            let n = count.or(Some(0)).map(|count| count * 10 + delta);

            state.mode.set_count(n);
        }

        (_, Esc) => {
            state.set_mode(Normal { count: None });
        }

        (Normal { count, .. }, Char('h'))
        | (Select { count, .. }, Char('h'))
        | (Normal { count, .. }, Key::Left)
        | (Select { count, .. }, Key::Left) => {
            let count = count.unwrap_or(1);
            state.mode.set_count(None);
            state.left(count);
        }
        (Normal { count, .. }, Char('j'))
        | (Select { count, .. }, Char('j'))
        | (Normal { count, .. }, Key::Down)
        | (Select { count, .. }, Key::Down) => {
            let count = count.unwrap_or(1);
            state.mode.set_count(None);
            state.down(count);
        }
        (Normal { count, .. }, Char('k'))
        | (Select { count, .. }, Char('k'))
        | (Normal { count, .. }, Key::Up)
        | (Select { count, .. }, Key::Up) => {
            let count = count.unwrap_or(1);
            state.mode.set_count(None);
            state.up(count);
        }
        (Normal { count, .. }, Char('l'))
        | (Select { count, .. }, Char('l'))
        | (Normal { count, .. }, Key::Right)
        | (Select { count, .. }, Key::Right) => {
            let count = count.unwrap_or(1);
            state.mode.set_count(None);
            state.right(count);
        }

        (_, Key::Left) => state.left(1),
        (_, Key::Down) => state.down(1),
        (_, Key::Up) => state.up(1),
        (_, Key::Right) => state.right(1),

        (Normal { .. }, Char('v')) => state.set_mode(Select { anchor: state.cursor, count: None }),

        (Normal { .. }, Char('i')) => state.set_mode(Edit),
        (Normal { .. }, Char('a')) => {
            state.set_mode(Edit);
            state.right(1);
        }

        (Normal { .. }, Char('I')) => {
            state.bol();
            state.set_mode(Edit);
        }
        (Normal { .. }, Char('A')) => {
            state.eol();
            state.set_mode(Edit);
        }

        (Normal { .. }, Char('o')) => {
            state.eol();
            state.insert_at_cursor('\n');
            state.forward(1);
            state.set_mode(Edit);
        }
        (Normal { .. }, Char('O')) => {
            state.bol();
            state.insert_at_cursor('\n');
            state.set_mode(Edit);
        }

        (Normal { count, .. }, Char('W')) => {
            let count = count.unwrap_or(1);
            state.forward_words(count);
        }

        (Normal { count, .. }, Char('B')) => {
            let count = count.unwrap_or(1);
            state.backward_words(count);
        }

        (Mode::Normal { .. }, Char('s')) => {
            state.set_mode(Mode::Operator {
                prompt: Some("Surround".into()),
                count: None,

                and_then: Callback::new(
                    |mut state: State<Buf>, (before, after): (Cursor, Cursor)| {
                        let after = after
                            .backward_while(&state.buffer, |p| {
                                state
                                    .buffer
                                    .get(p.left(1, &state.buffer).unwrap())
                                    .map(|ch| ch.is_whitespace())
                                    .unwrap_or(false)
                            })
                            .unwrap_or_else(|e| e.at);

                        let before = before
                            .forward_while(&state.buffer, |p| {
                                state.buffer.get(p).map(|ch| ch.is_whitespace()).unwrap_or(false)
                            })
                            .unwrap_or_else(|e| e.at);

                        state.set_mode(Mode::Query {
                            prompt: Some("Surround with".into()),
                            length: Some(1),
                            partial: String::with_capacity(1),
                            and_then: Callback::new(move |mut state, result: String| {
                                let (prefix, suffix) = match result.chars().nth(0).unwrap() {
                                    '(' | ')' => ('(', ')'),
                                    '[' | ']' => ('[', ']'),
                                    '{' | '}' => ('{', '}'),

                                    '|' => ('|', '|'),
                                    '"' => ('"', '"'),
                                    '\'' => ('\'', '\''),

                                    ch => (ch, ch),
                                };

                                state.insert(after, suffix);
                                state.insert(before, prefix);

                                state.set_mode(Normal { count: None });

                                state
                            }),
                        });
                        state
                    },
                ),
            });
        }

        (Operator { and_then, count, .. }, Char('W')) => {
            let end = state
                .cursor
                .forward_words(count.unwrap_or(1), &state.buffer)
                .unwrap_or_else(|e| e.at);

            *state = and_then.0(state.clone(), (state.cursor, end));
        }

        (Operator { and_then, count, .. }, Char('B')) => {
            let start = state
                .cursor
                .backward_words(count.unwrap_or(1), &state.buffer)
                .unwrap_or_else(|e| e.at);

            *state = and_then.0(state.clone(), (start, state.cursor));
        }

        (Normal { .. }, Char('d')) => state.set_mode(Operator {
            prompt: Some("Delete".to_string()),
            count: None,
            and_then: Callback::new(|state, (start, end)| {
                let mut state = state.clone();

                state.delete((Included(start), Excluded(end)));
                state.cursor = start;

                state
            }),
        }),

        (Edit, Char(ch)) => {
            state.insert_at_cursor(ch);
            state.forward(1);
        }

        (Edit, Backspace) => {
            state.backward(1);

            if state.buffer.get(state.cursor).is_some() {
                state.delete(state.cursor..=state.cursor);
            };
        }

        (Edit, Delete) => {
            if state.buffer.get(state.cursor).is_some() {
                state.delete(state.cursor..=state.cursor);
            };
        }

        (Edit, Ctrl('w')) => {
            let anchor = state.cursor;
            state.backward_words(1);

            state.delete(state.cursor..anchor);
        }

        (Query { and_then, .. }, Char(ch)) => {
            match (&mut state.mode, ch) {
                (Query { length: None, .. }, '\n') => (),
                (Query { length, ref mut partial, .. }, _) => {
                    partial.push(ch);
                }

                _ => unreachable!(),
            };

            let (dispatch, partial, and_then) = match (&state.mode, ch) {
                (Query { partial, and_then, length: None, .. }, '\n') => (true, partial, and_then),
                (Query { partial, and_then, length: Some(n), .. }, _) if partial.len() == *n => {
                    (true, partial, and_then)
                }
                (Query { partial, and_then, .. }, _) => (false, partial, and_then),

                _ => unreachable!(),
            };

            if dispatch {
                *state = and_then.0(state.clone(), partial.clone());
            }
        }

        _ => (),
    };

    Some(state)
}
