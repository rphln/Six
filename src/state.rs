use std::borrow::Borrow;
use std::fmt;
use std::ops::RangeBounds;
use std::sync::Arc;

use std::error::Error;
use termion::event::Key;

use rlua::{
    // Function,
    // MetaMethod,
    // Variadic,
    Lua,
    UserData,
    UserDataMethods,
};

use crate::buffer::{Buf, BufferView};
use crate::cursor::Cursor;

#[derive(Debug, Clone, Default)]
pub struct EditView {
    /// Current content.
    buffer: Buf,

    /// Cursor position in the content.
    cursor: Cursor,
}

impl EditView {
    /// Returns a reference to the text buffer.
    #[inline]
    pub fn buffer(&self) -> &Buf {
        &self.buffer
    }

    /// Returns a mutable reference to the text buffer.
    pub fn buffer_mut(&mut self) -> &mut Buf {
        &mut self.buffer
    }

    /// Returns a copy of the cursor position.
    #[inline]
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Updates the cursor position by applying a function over its previous value.
    pub fn with_cursor(&mut self, map: impl FnOnce(Cursor, &Buf) -> Cursor) {
        self.cursor = map(self.cursor(), self.buffer());
    }
}

#[derive(Default, Debug)]
pub struct State {
    /// Primary editor view.
    view: EditView,

    /// Current mode.
    mode: Mode,
}

// TODO: Replace with a trait alias.
pub trait Callback<T>:
    Send + Sync + 'static + Fn(&mut State, &mut Lua, T) -> Result<(), Box<dyn Error>>
{
}

impl<T, U> Callback<T> for U where
    U: Send + Sync + 'static + Fn(&mut State, &mut Lua, T) -> Result<(), Box<dyn Error>>
{
}

// impl<'a, T: ?Sized + 'a> fmt::Debug for dyn Callback<&'a T> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.write_str(std::any::type_name::<Self>())
//     }
// }

impl fmt::Debug for dyn Callback<Range> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(std::any::type_name::<Self>())
    }
}

impl fmt::Debug for dyn for<'r> Callback<&'r str> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(std::any::type_name::<Self>())
    }
}

pub type Range = (Cursor, Cursor);

#[derive(Clone, Debug)]
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
        callback: Arc<dyn Callback<Range>>,
    },

    /// Queries the user for a text input and applies an operation.
    Query {
        prompt: String,

        /// Length of the query before the callback is invoked. If `None`,
        /// invokes on `Return`.
        length: Option<usize>,

        /// Partial buffer for the query.
        partial: EditView,

        /// Callback invoked once the input has finished.
        callback: Arc<dyn for<'c> Callback<&'c str>>,
    },
}

impl UserData for Cursor {}

impl UserData for &mut State {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("insert_at_cursor", |_, state, text: String| {
            state.edit(state.cursor()..state.cursor(), text.as_str());
            Ok(())
        });

        methods.add_method("cursor", |_, state, ()| Ok(state.cursor()));

        methods.add_method_mut("delete", |_, state, at: Cursor| {
            state.delete(at..=at);
            Ok(())
        })
    }
}

impl State {
    /// Returns a reference to the mode.
    #[inline]
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Updates the mode by applying a function over its previous value.
    pub fn with_mode(&mut self, map: impl FnOnce(Mode) -> Mode) {
        // FIXME: We're probably cloning without any need.
        self.mode = map(self.mode.clone());
    }

    /// Returns a copy of the cursor position.
    #[inline]
    pub fn cursor(&self) -> Cursor {
        self.view.cursor()
    }

    /// Updates the cursor position by applying a function over its previous value.
    #[inline]
    pub fn with_cursor(&mut self, map: impl FnOnce(Cursor, &Buf) -> Cursor) {
        self.view.with_cursor(map)
    }

    /// Returns a reference to the text buffer.
    #[inline]
    pub fn buffer(&self) -> &Buf {
        self.view.buffer()
    }

    /// Returns a reference to the editor view.
    #[inline]
    pub fn editor(&self) -> &EditView {
        &self.view
    }

    pub fn insert(&mut self, at: impl Borrow<Cursor>, ch: char) {
        self.view.buffer.insert(at, ch);
    }

    pub fn insert_at_cursor(&mut self, ch: char) {
        self.insert(self.view.cursor, ch)
    }

    pub fn delete(&mut self, range: impl RangeBounds<Cursor>) {
        self.view.buffer.delete(range);
    }

    pub fn edit(&mut self, range: impl RangeBounds<Cursor>, text: &str) {
        self.view.buffer.edit(range, text);
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

    pub fn with_partial(mut self, map: impl FnOnce(&mut EditView)) -> Self {
        match self {
            Mode::Query { ref mut partial, .. } => map(partial),
            _ => panic!("Attempt to set `partial` for an incompatible mode"),
        };

        self
    }

    pub fn normal(count: Option<usize>) -> Self {
        Self::Normal { count }
    }

    pub fn operator(
        prompt: impl Into<String>,
        count: Option<usize>,
        callback: impl Callback<Range>,
    ) -> Self {
        Self::Operator { prompt: prompt.into(), count, callback: Arc::new(callback) }
    }

    pub fn query(
        prompt: impl Into<String>,
        length: Option<usize>,
        callback: impl for<'r> Callback<&'r str>,
    ) -> Self {
        Self::Query {
            prompt: prompt.into(),
            partial: EditView::default(),
            length,
            callback: Arc::new(callback),
        }
    }
}

pub fn event_loop(
    state: &mut State,
    lua: &mut Lua,
    event: termion::event::Key,
) -> Result<(), Box<dyn Error>> {
    use termion::event::Key::{Backspace, Char, Ctrl, Delete, Esc};
    use Mode::{Edit, Normal, Operator, Query, Select};

    match (state.mode().clone(), event) {
        (Edit, Esc) => {
            state.with_cursor(|cur, buffer| cur.at_left(1, buffer));
            state.with_mode(|_| Mode::normal(None));
        },

        (_, Esc) => state.with_mode(|_| Mode::normal(None)),

        (Normal { .. }, Char('v')) => {
            let anchor = state.cursor();
            state.with_mode(|_| Select { anchor, count: None });
        },

        (Normal { .. }, Char('i')) => state.with_mode(|_| Edit),
        (Normal { .. }, Char('a')) => {
            state.with_mode(|_| Edit);
            state.with_cursor(|cursor, buffer| cursor.at_right(1, buffer))
        },

        (Normal { .. }, Char('I')) => {
            state.with_mode(|_| Edit);
            state.with_cursor(|cursor, buffer| cursor.at_bol(buffer));
        },
        (Normal { .. }, Char('A')) => {
            state.with_mode(|_| Edit);
            state.with_cursor(|cursor, buffer| cursor.at_eol(buffer));
        },

        (Normal { .. }, Char('o')) => {
            let eol = state.cursor().at_eol(state.buffer());
            state.insert(eol, '\n');

            state.with_cursor(|_, buffer| eol.forward(1, buffer));
            state.with_mode(|_| Mode::Edit);
        },

        (Normal { .. }, Char('O')) => {
            let bol = state.cursor().at_bol(state.buffer());
            state.insert(bol, '\n');

            state.with_cursor(|_, buffer| bol.backward(1, buffer));
            state.with_mode(|_| Mode::Edit);
        },

        (Normal { .. }, Char('s')) => {
            let surround = |state: &mut State, _lua: &mut Lua, (start, end): Range| {
                // Replicates `vim-surround` by skipping any whitespaces at the end.
                let end = end.backward_while(state.buffer(), |position| {
                    let position = position.at_left(1, state.buffer());
                    state.buffer().get(position)?.is_whitespace().into()
                });

                let surround = move |state: &mut State, _lua: &mut Lua, sandwich: &str| {
                    let mut chars = sandwich.chars();

                    let prefix = chars.next().expect("prefix");
                    let suffix = chars.next().expect("suffix");

                    state.insert(end, suffix);
                    state.insert(start, prefix);

                    state.with_mode(|_| Mode::default());
                    state.with_cursor(|_, _| start);

                    Ok(())
                };

                state.with_mode(|_| Mode::query("Surround with", Some(2), surround));

                Ok(())
            };

            state.with_mode(|_| Mode::operator("Surround", None, surround));
        },

        (Normal { count }, Char('d')) => {
            state.with_mode(|_| {
                Mode::operator(
                    "Delete",
                    count,
                    |state: &mut State, _lua: &mut Lua, (start, end): Range| {
                        state.with_cursor(|_, _| start);
                        state.delete(start..=end);
                        state.with_mode(|_| Mode::default());

                        Ok(())
                    },
                )
            });
        },

        (Normal { .. }, Char(';')) => {
            state.with_mode(|_| {
                Mode::query("Eval", None, |state: &mut State, lua: &mut Lua, program: &str| {
                    state.with_mode(|_| Mode::default());

                    lua.context(|ctx| {
                        ctx.scope(|scope| {
                            ctx.globals().set("state", scope.create_nonstatic_userdata(state)?)?;

                            ctx.load(program).exec()
                        })
                    })?;

                    Ok(())
                })
            });
        },

        (Normal { count }, Char('f')) => state.with_mode(|_| {
            Mode::query("Jump to next", Some(1), move |state: &mut State, _: &mut Lua, ch: &str| {
                let ch = ch.chars().next().expect("ch");

                state.with_cursor(|cur, buf| {
                    (1..=count.unwrap_or(1))
                        .try_fold(cur, |cur, _| {
                            cur.try_forward(1, buf)?
                                .try_forward_while(buf, |p| Some(buf.get(p)? != ch))
                        })
                        .unwrap_or(cur)
                });

                state.with_mode(|_| Normal { count: None });

                Ok(())
            })
        }),

        (Operator { ref callback, count, .. }, Char('W')) => {
            let start = state.cursor();
            let end = start.forward_words(count.unwrap_or(1), state.buffer());

            callback(state, lua, (start, end))?;
        },

        (Operator { callback, count, .. }, Char('B')) => {
            let end = state.cursor();
            let start = end.backward_words(count.unwrap_or(1), state.buffer());

            callback(state, lua, (start, end))?;
        },

        (Query { mut partial, callback, length, .. }, Char(ch)) => {
            let at = partial.cursor();
            partial.buffer_mut().insert(at, ch);

            partial.with_cursor(|cur, buffer| cur.forward(1, buffer));

            match length {
                None if ch == '\n' => callback(state, lua, partial.buffer().as_str()),
                Some(length) if length == partial.buffer().len() => {
                    callback(state, lua, partial.buffer().as_str())
                },

                _ => {
                    state.with_mode(|mode| mode.with_partial(|p| *p = partial));
                    Ok(())
                },
            }?;
        },

        (Query { .. }, Backspace) => {
            state.with_mode(|mode| {
                mode.with_partial(|partial| {
                    partial.with_cursor(|cur, buf| cur.backward(1, buf));

                    let cursor = partial.cursor();
                    if partial.buffer().get(cursor).is_some() {
                        partial.buffer_mut().delete(cursor..=cursor);
                    }
                })
            });
        },

        (Edit, Char(ch)) => {
            state.insert(state.cursor(), ch);
            state.with_cursor(|cur, buffer| cur.forward(1, buffer));
        },

        (Edit, Backspace) => {
            state.with_cursor(|cur, buffer| cur.backward(1, buffer));

            let cursor = state.cursor();
            if state.buffer().get(cursor).is_some() {
                state.delete(cursor..=cursor)
            }
        },

        (Edit, Delete) => {
            let cursor = state.cursor();
            if state.buffer().get(cursor).is_some() {
                state.delete(cursor..=cursor)
            }
        },

        (Edit, Ctrl('w')) => {
            let anchor = state.cursor();

            state.with_cursor(|cur, buffer| cur.backward_words(1, buffer));
            state.delete(state.cursor()..anchor);
        },

        (Normal { count, .. }, Char('W')) => {
            state.with_mode(|mode| mode.with_count(None));
            state.with_cursor(|cur, buffer| cur.forward_words(count.unwrap_or(1), buffer));
        },

        (Normal { count, .. }, Char('B')) => {
            state.with_mode(|mode| mode.with_count(None));
            state.with_cursor(|cur, buffer| cur.backward_words(count.unwrap_or(1), buffer));
        },

        (Normal { count, .. }, Char('h'))
        | (Select { count, .. }, Char('h'))
        | (Normal { count, .. }, Key::Left)
        | (Select { count, .. }, Key::Left) => {
            state.with_mode(|mode| mode.with_count(None));
            state.with_cursor(|cur, buffer| cur.at_left(count.unwrap_or(1), buffer));
        },

        (Normal { count, .. }, Char('j'))
        | (Select { count, .. }, Char('j'))
        | (Normal { count, .. }, Key::Down)
        | (Select { count, .. }, Key::Down) => {
            state.with_mode(|mode| mode.with_count(None));
            state.with_cursor(|cur, buffer| cur.below(count.unwrap_or(1), buffer));
        },

        (Normal { count, .. }, Char('k'))
        | (Select { count, .. }, Char('k'))
        | (Normal { count, .. }, Key::Up)
        | (Select { count, .. }, Key::Up) => {
            state.with_mode(|mode| mode.with_count(None));
            state.with_cursor(|cur, buffer| cur.above(count.unwrap_or(1), buffer));
        },

        (Normal { count, .. }, Char('l'))
        | (Select { count, .. }, Char('l'))
        | (Normal { count, .. }, Key::Right)
        | (Select { count, .. }, Key::Right) => {
            state.with_mode(|mode| mode.with_count(None));
            state.with_cursor(|cur, buffer| cur.at_right(count.unwrap_or(1), buffer));
        },

        (Edit { .. }, Key::Left) => state.with_cursor(|cur, buffer| cur.at_left(1, buffer)),
        (Edit { .. }, Key::Down) => state.with_cursor(|cur, buffer| cur.below(1, buffer)),
        (Edit { .. }, Key::Up) => state.with_cursor(|cur, buffer| cur.above(1, buffer)),
        (Edit { .. }, Key::Right) => state.with_cursor(|cur, buffer| cur.at_right(1, buffer)),

        (Normal { count }, Char(ch @ '0'..='9'))
        | (Select { count, .. }, Char(ch @ '0'..='9'))
        | (Operator { count, .. }, Char(ch @ '0'..='9')) => {
            let delta = ch.to_digit(10).unwrap() as usize;
            let n = count.or(Some(0)).map(|count| count * 10 + delta);

            state.with_mode(|mode| mode.with_count(n))
        },

        (Operator { .. }, _) => state.with_mode(|_| Mode::default()),

        _ => (),
    };

    Ok(())
}
