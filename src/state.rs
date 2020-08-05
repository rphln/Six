use std::ops::RangeBounds;
use std::rc::Rc;

use termion::event::Key;

use crate::buffer::Buf;
use crate::cursor::Cursor;

#[derive(Default, Debug)]
pub struct State {
    /// Current content.
    buffer: Buf,

    /// Cursor position in the content.
    cursor: Cursor,

    /// Current mode.
    mode: Mode,
}

type Callback<T> = Rc<dyn Fn(&mut State, T)>;

type Range = (Cursor, Cursor);

#[derive(Clone, Derivative)]
#[derivative(Debug = "transparent")]
pub enum Mode {
    /// The default editor mode.
    Normal { count: Option<usize> },

    /// Text input mode.
    Edit,

    /// Queries the user for a text range.
    Select { anchor: Cursor, count: Option<usize> },

    /// Queries the user for a text object and applies an operation.
    Operator {
        prompt: String,

        ///  fcar
        count: Option<usize>,

        /// Callback invoked once a text object has been provided.
        #[derivative(Debug = "ignore")]
        callback: Callback<Range>,
    },

    /// Queries the user for a text input and applies an operation.
    Query {
        prompt: String,

        /// Length of the query before the callback is invoked. If `None`,
        /// invokes on `Return`.
        length: Option<usize>,

        /// Partial buffer for the query.
        partial: String,

        /// Callback invoked once the input has finished.
        #[derivative(Debug = "ignore")]
        callback: Callback<String>,
    },
}

impl State {
    pub fn col(&self) -> usize {
        self.cursor.col()
    }

    pub fn row(&self) -> usize {
        self.cursor.row()
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn buf(&self) -> &Buf {
        &self.buffer
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.buffer.lines()
    }

    pub fn with_cursor(&mut self, cursor: impl Fn(Cursor, &Buf) -> Cursor) -> &mut Self {
        self.cursor = cursor(self.cursor, &self.buffer);
        self
    }

    pub fn with_mode(&mut self, mode: impl Fn(Mode) -> Mode) -> &mut Self {
        self.mode = mode(self.mode.clone());
        self
    }

    pub fn insert(&mut self, at: Cursor, ch: char) -> &mut Self {
        self.buffer.insert(at, ch);
        self
    }

    pub fn insert_at_cursor(&mut self, ch: char) -> &mut Self {
        self.insert(self.cursor, ch)
    }

    pub fn delete(&mut self, range: impl RangeBounds<Cursor>) -> &mut Self {
        self.buffer.delete(range);
        self
    }

    pub fn edit(&mut self, range: impl RangeBounds<Cursor>, text: &str) -> &mut Self {
        self.buffer.edit(range, text);
        self
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal { count: None }
    }
}

impl Mode {
    pub fn with_count(mut self, next: Option<usize>) -> Self {
        match self {
            Mode::Operator { ref mut count, .. }
            | Mode::Normal { ref mut count, .. }
            | Mode::Select { ref mut count, .. } => *count = next,

            _ => panic!("Attempt to set count for an incompatible mode"),
        };

        self
    }

    pub fn normal(count: Option<usize>) -> Self {
        Self::Normal { count }
    }

    pub fn operator(
        prompt: impl Into<String>,
        count: Option<usize>,
        callback: Callback<Range>,
    ) -> Self {
        Self::Operator { prompt: prompt.into(), count, callback }
    }

    pub fn query(
        prompt: impl Into<String>,
        length: Option<usize>,
        callback: Callback<String>,
    ) -> Self {
        Self::Query { prompt: prompt.into(), partial: String::default(), length, callback }
    }
}

pub fn event_loop(state: &mut State, event: termion::event::Key) {
    use termion::event::Key::{Backspace, Char, Ctrl, Delete, Esc};
    use Mode::{Edit, Normal, Operator, Query, Select};

    match (state.mode.clone(), event) {
        (Edit, Esc) => {
            state.with_cursor(|cur, buf| cur.at_left(1, buf)).with_mode(|_| Mode::normal(None))
        },

        (_, Esc) => state.with_mode(|_| Mode::normal(None)),

        (Normal { .. }, Char('v')) => {
            let anchor = state.cursor().clone();
            state.with_mode(|_| Select { anchor, count: None })
        },

        (Normal { .. }, Char('i')) => state.with_mode(|_| Edit),
        (Normal { .. }, Char('a')) => {
            state.with_mode(|_| Edit).with_cursor(|cursor, buffer| cursor.at_right(1, buffer))
        },

        (Normal { .. }, Char('I')) => {
            state.with_mode(|_| Edit).with_cursor(|cursor, buffer| cursor.at_bol(buffer))
        },
        (Normal { .. }, Char('A')) => {
            state.with_mode(|_| Edit).with_cursor(|cursor, buffer| cursor.at_eol(buffer))
        },

        (Normal { .. }, Char('o')) => state
            .with_cursor(|cur, buf| cur.at_eol(buf))
            .insert_at_cursor('\n')
            .with_cursor(|cur, buf| cur.forward(1, buf))
            .with_mode(|_| Mode::Edit),

        (Normal { .. }, Char('O')) => state
            .with_cursor(|cur, buf| cur.at_bol(buf))
            .insert_at_cursor('\n')
            .with_mode(|_| Mode::Edit),

        (Normal { .. }, Char('s')) => {
            let callback = |state: &mut State, (start, end): Range| {
                // Replicates `vim-surround` by skipping any whitespaces at the end.
                let end = end.backward_while(&state.buffer, |position| {
                    let position = position.at_left(1, &state.buffer);

                    match &state.buffer.get(position) {
                        Some(ch) => ch.is_whitespace(),
                        None => false,
                    }
                });

                let surround = move |state: &mut State, sandwich: String| {
                    let mut chars = sandwich.chars();

                    let prefix = chars.next().expect("prefix");
                    let suffix = chars.next().expect("suffix");

                    state
                        .insert(end, suffix)
                        .insert(start, prefix)
                        .with_mode(|_| Mode::default())
                        .with_cursor(|_, _| start);
                };

                state.with_mode(|_| Mode::query("Surround with", Some(2), Rc::new(surround)));
            };

            state.with_mode(|_| Mode::operator("Surround", None, Rc::new(callback)))
        },

        (Normal { count }, Char('d')) => state.with_mode(|_| {
            Mode::operator(
                "Delete",
                count,
                Rc::new(|state, (start, end)| {
                    state
                        .with_cursor(|_, _| start)
                        .delete(start..end)
                        .with_mode(|_| Mode::default());
                }),
            )
        }),

        (Normal { .. }, Char(';')) => state.with_mode(|_| {
            Mode::query(
                "Eval",
                None,
                Rc::new(|state, _| {
                    state.with_mode(|_| Mode::default());
                }),
            )
        }),

        (Operator { callback, count, .. }, Char('W')) => {
            let start = state.cursor;
            let end = start.forward_words(count.unwrap_or(1), &state.buffer);

            callback(state, (start, end));
            state
        },

        (Operator { callback, count, .. }, Char('B')) => {
            let end = state.cursor;
            let start = end.backward_words(count.unwrap_or(1), &state.buffer);

            callback(state, (start, end));
            state
        },

        (Query { callback, .. }, Char(ch)) => {
            let (partial, dispatch) = match &mut state.mode {
                Query { partial, length: None, .. } if matches!(ch, '\n') => (partial, true),

                Query { partial, length, .. } => {
                    partial.push(ch);

                    if let Some(length) = *length {
                        let len = partial.len();
                        (partial, length == len)
                    } else {
                        (partial, false)
                    }
                },

                _ => unreachable!(),
            };

            if dispatch {
                let partial = partial.to_string();
                callback(state, partial);
            }

            state
        },

        (Edit, Char(ch)) => {
            state.insert(state.cursor, ch).with_cursor(|cur, buf| cur.forward(1, buf))
        },

        (Edit, Backspace) => {
            state.with_cursor(|cur, buf| cur.at_left(1, buf));

            if state.buffer.get(state.cursor).is_some() {
                state.delete(state.cursor..=state.cursor);
            };

            state
        },

        (Edit, Delete) => {
            if state.buffer.get(state.cursor).is_some() {
                state.delete(state.cursor..=state.cursor);
            };

            state
        },

        (Edit, Ctrl('w')) => {
            let anchor = state.cursor().clone();

            state.with_cursor(|cur, buf| cur.backward_words(1, buf));
            state.delete(state.cursor..anchor);

            state
        },

        (Normal { count, .. }, Char('W')) => state
            .with_mode(|mode| mode.with_count(None))
            .with_cursor(|cur, buf| cur.forward_words(count.unwrap_or(1), buf)),

        (Normal { count, .. }, Char('B')) => state
            .with_mode(|mode| mode.with_count(None))
            .with_cursor(|cur, buf| cur.backward_words(count.unwrap_or(1), buf)),

        (Normal { count, .. }, Char('h'))
        | (Select { count, .. }, Char('h'))
        | (Normal { count, .. }, Key::Left)
        | (Select { count, .. }, Key::Left) => state
            .with_mode(|mode| mode.with_count(None))
            .with_cursor(|cur, buf| cur.at_left(count.unwrap_or(1), buf)),

        (Normal { count, .. }, Char('j'))
        | (Select { count, .. }, Char('j'))
        | (Normal { count, .. }, Key::Down)
        | (Select { count, .. }, Key::Down) => state
            .with_mode(|mode| mode.with_count(None))
            .with_cursor(|cur, buf| cur.below(count.unwrap_or(1), buf)),

        (Normal { count, .. }, Char('k'))
        | (Select { count, .. }, Char('k'))
        | (Normal { count, .. }, Key::Up)
        | (Select { count, .. }, Key::Up) => state
            .with_mode(|mode| mode.with_count(None))
            .with_cursor(|cur, buf| cur.above(count.unwrap_or(1), buf)),

        (Normal { count, .. }, Char('l'))
        | (Select { count, .. }, Char('l'))
        | (Normal { count, .. }, Key::Right)
        | (Select { count, .. }, Key::Right) => state
            .with_mode(|mode| mode.with_count(None))
            .with_cursor(|cur, buf| cur.at_right(count.unwrap_or(1), buf)),

        (Edit, Key::Left) => state.with_cursor(|cur, buf| cur.at_left(1, buf)),
        (Edit, Key::Down) => state.with_cursor(|cur, buf| cur.below(1, buf)),
        (Edit, Key::Up) => state.with_cursor(|cur, buf| cur.above(1, buf)),
        (Edit, Key::Right) => state.with_cursor(|cur, buf| cur.at_right(1, buf)),

        (Normal { count }, Char(ch @ '0'..='9'))
        | (Select { count, .. }, Char(ch @ '0'..='9'))
        | (Operator { count, .. }, Char(ch @ '0'..='9')) => {
            let delta = ch.to_digit(10).unwrap() as usize;
            let n = count.or(Some(0)).map(|count| count * 10 + delta);

            state.with_mode(|mode| mode.with_count(n))
        },

        (Operator { .. }, _) => state.with_mode(|_| Mode::default()),

        _ => state,
    };
}
