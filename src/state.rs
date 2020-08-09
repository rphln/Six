use std::borrow::Borrow;
use std::fmt;
use std::ops::RangeBounds;
use std::sync::Arc;

use std::error::Error;
use termion::event::Key;

use rlua::{
    Function,
    // MetaMethod,
    // Variadic,
    Lua,
    UserData,
    UserDataMethods,
};

use crate::buffer::{Buf, BufferView};
use crate::cursor::{Bounded, Cursor, CursorIterator, Line, Paragraph, Unbounded, Word};

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
    #[inline]
    pub fn with_cursor(&mut self, map: impl FnOnce(Cursor, &Buf) -> Cursor) {
        self.cursor = map(self.cursor(), self.buffer());
    }

    /// Attempts to update the cursor position by applying a function over its previous value.
    ///
    /// Maintains the original position on failure.
    #[inline]
    pub fn try_with_cursor(&mut self, map: impl FnOnce(Cursor, &Buf) -> Option<Cursor>) {
        self.cursor = map(self.cursor(), self.buffer()).unwrap_or(self.cursor);
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
    Normal {
        /// The number of times to repeat the next operation.
        repeat: Option<usize>,
    },

    /// The text input mode.
    Edit,

    /// Queries the user for a text range.
    Select {
        /// The fixed point of the selection.
        anchor: Cursor,

        /// The number of times to repeat the next operation.
        repeat: Option<usize>,
    },

    /// Queries the user for a text object and applies an operation.
    Operator {
        /// The prompt displayed to the user.
        prompt: String,

        /// The number of times to repeat the provided text object.
        repeat: Option<usize>,

        /// Callback invoked once a text object has been provided.
        callback: Arc<dyn Callback<Range>>,
    },

    /// Queries the user for a text input and applies an operation.
    Query {
        /// The prompt displayed to the user.
        prompt: String,

        /// The maximum length of the queried string.
        length: Option<usize>,

        /// Partial buffer for the query.
        partial: EditView,

        /// Callback invoked once the input has finished.
        callback: Arc<dyn for<'c> Callback<&'c str>>,
    },
}

impl UserData for &Buf {}
impl UserData for &Cursor {}

impl<'a, B: 'a + BufferView> UserData for CursorIterator<'a, B, Unbounded> {
    fn add_methods<'lua, U: UserDataMethods<'lua, Self>>(methods: &mut U) {
        methods.add_method_mut("next", |_, iter, ()| Ok(iter.next()))
    }
}

impl UserData for Cursor {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("forward", |_, &cursor, (repeat, buffer): (usize, &Buf)| {
            Ok(cursor.iter::<_, Unbounded>(buffer).take(repeat).last().unwrap_or(cursor))
        })
    }
}

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

    /// Attempts to update the cursor position by applying a function over its previous value.
    ///
    /// Maintains the original position on failure.
    #[inline]
    pub fn try_with_cursor(&mut self, map: impl FnOnce(Cursor, &Buf) -> Option<Cursor>) {
        self.view.try_with_cursor(map)
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
        Self::Normal { repeat: None }
    }
}

impl Mode {
    pub fn with_count(mut self, next: Option<usize>) -> Self {
        match self {
            Mode::Operator { ref mut repeat, .. }
            | Mode::Normal { ref mut repeat, .. }
            | Mode::Select { ref mut repeat, .. } => *repeat = next,

            _ => panic!("Attempt to set repeat for an incompatible mode"),
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

    pub fn normal(repeat: Option<usize>) -> Self {
        Self::Normal { repeat }
    }

    pub fn operator(
        prompt: impl Into<String>,
        repeat: Option<usize>,
        callback: impl Callback<Range>,
    ) -> Self {
        Self::Operator { repeat, prompt: prompt.into(), callback: Arc::new(callback) }
    }

    pub fn query(
        prompt: impl Into<String>,
        length: Option<usize>,
        callback: impl for<'r> Callback<&'r str>,
    ) -> Self {
        Self::Query {
            length,
            prompt: prompt.into(),
            partial: EditView::default(),
            callback: Arc::new(callback),
        }
    }
}

pub fn event_loop(state: &mut State, lua: &mut Lua, event: Key) -> Result<(), Box<dyn Error>> {
    use termion::event::Key::{Backspace, Char, Ctrl, Delete, Esc};
    use Mode::{Edit, Normal, Operator, Query, Select};

    match (state.mode().clone(), event) {
        (Edit, Esc) => {
            state.try_with_cursor(|cur, buffer| cur.iter::<_, Bounded>(buffer).next_back());

            state.with_mode(|_| Mode::normal(None));
        },

        (_, Esc) => state.with_mode(|_| Mode::normal(None)),

        (Normal { .. }, Char('v')) => {
            let anchor = state.cursor();
            state.with_mode(|_| Select { anchor, repeat: None });
        },

        (Normal { .. }, Char('i')) => state.with_mode(|_| Edit),
        (Normal { .. }, Char('a')) => {
            state.with_mode(|_| Edit);
            state.try_with_cursor(|cursor, buffer| cursor.iter::<_, Bounded>(buffer).next())
        },

        (Normal { .. }, Char('I')) => {
            state.with_mode(|_| Edit);
            state.try_with_cursor(|cursor, buffer| cursor.iter::<_, Bounded>(buffer).rev().last());
        },
        (Normal { .. }, Char('A')) => {
            state.with_mode(|_| Edit);
            state.try_with_cursor(|cursor, buffer| cursor.iter::<_, Bounded>(buffer).last());
        },

        (Normal { .. }, Char('o')) => {
            let eol = state
                .cursor()
                .iter::<_, Bounded>(state.buffer())
                .last()
                .unwrap_or_else(|| state.cursor());
            state.insert(eol, '\n');

            state.try_with_cursor(|_, buffer| eol.iter::<_, Unbounded>(buffer).next());
            state.with_mode(|_| Mode::Edit);
        },

        (Normal { .. }, Char('O')) => {
            let bol = state
                .cursor()
                .iter::<_, Bounded>(state.buffer())
                .rev()
                .last()
                .unwrap_or_else(|| state.cursor());
            state.insert(bol, '\n');

            state.try_with_cursor(|_, buffer| bol.iter::<_, Unbounded>(buffer).next_back());
            state.with_mode(|_| Mode::Edit);
        },

        (Normal { .. }, Char('s')) => {
            let surround = |state: &mut State, _lua: &mut Lua, (start, end): Range| {
                // Replicates `vim-surround` by skipping any whitespaces at the end.
                let buf = state.buffer();
                let end = end
                    .iter::<_, Unbounded>(buf)
                    .rev()
                    .find(|p| buf.get(p).map(|ch| !ch.is_whitespace()).unwrap_or(true))
                    .unwrap_or(end);

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

        (Normal { repeat }, Char('d')) => {
            state.with_mode(|_| {
                Mode::operator(
                    "Delete",
                    repeat,
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

        (Normal { .. }, Char('u')) => state.with_mode(|_| {
            Mode::query(
                "Eval & Forward",
                None,
                |state: &mut State, lua: &mut Lua, program: &str| {
                    state.with_mode(|_| Mode::default());

                    state.try_with_cursor(|cursor, buffer| {
                        lua.context::<_, Option<Cursor>>(|ctx| {
                            ctx.scope(|scope| {
                                let iter = cursor.iter::<_, Unbounded>(buffer);
                                let iter = scope.create_nonstatic_userdata(iter);

                                ctx.load(program)
                                    .eval::<Function>()
                                    .unwrap()
                                    .call::<_, Option<Cursor>>(iter)
                                    .unwrap()
                            })
                        })
                    });

                    Ok(())
                },
            )
        }),

        (Normal { repeat }, Char('t')) => state.with_mode(|_| {
            Mode::query("Jump to next", Some(1), move |state: &mut State, _: &mut Lua, ch: &str| {
                let ch = ch.chars().next().expect("ch");

                state.try_with_cursor(|cur, buf| {
                    let repeat = repeat.unwrap_or(1);

                    let mut it = cur.iter::<_, Bounded>(buf);
                    (1..=repeat).fold(None, |_, _| {
                        it.find(|p| {
                            p.iter::<_, Bounded>(buf)
                                .next()
                                .map(|p| buf.get(p).map(|other| other == ch).expect("get"))
                                .unwrap_or(false)
                        })
                    })
                });

                state.with_mode(|_| Normal { repeat: None });

                Ok(())
            })
        }),

        (Operator { ref callback, repeat, .. }, Char('W')) => {
            let start = state.cursor();
            let end = start
                .iter::<_, Word>(state.buffer())
                .take(repeat.unwrap_or(1))
                .last()
                .expect("end");

            callback(state, lua, (start, end))?;
        },

        (Operator { callback, repeat, .. }, Char('B')) => {
            let end = state.cursor();
            let start = end
                .iter::<_, Word>(state.buffer())
                .rev()
                .take(repeat.unwrap_or(1))
                .last()
                .expect("start");

            callback(state, lua, (start, end))?;
        },

        (Query { mut partial, callback, length, .. }, Char(ch)) => {
            let at = partial.cursor();
            partial.buffer_mut().insert(at, ch);

            partial.try_with_cursor(|cur, buffer| cur.iter::<_, Unbounded>(buffer).next());

            match length {
                _ if ch == '\n' => {
                    callback(state, lua, partial.buffer().as_str())?;
                },

                Some(length) if length == partial.buffer().len() => {
                    callback(state, lua, partial.buffer().as_str())?;
                },

                _ => {
                    state.with_mode(|mode| mode.with_partial(|p| *p = partial));
                },
            };
        },

        (Query { .. }, Backspace) => {
            state.with_mode(|mode| {
                mode.with_partial(|partial| {
                    partial.try_with_cursor(|cur, buf| cur.iter::<_, Unbounded>(buf).next_back());

                    let cursor = partial.cursor();
                    if partial.buffer().get(cursor).is_some() {
                        partial.buffer_mut().delete(cursor..=cursor);
                    }
                })
            });
        },

        (Edit, Char(ch)) => {
            state.insert(state.cursor(), ch);
            state.try_with_cursor(|cur, buffer| cur.iter::<_, Unbounded>(buffer).next());
        },

        (Edit, Backspace) => {
            state.try_with_cursor(|cur, buffer| cur.iter::<_, Unbounded>(buffer).next_back());

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

            state.try_with_cursor(|cur, buffer| cur.iter::<_, Word>(buffer).next_back());
            state.delete(state.cursor()..anchor);
        },

        (Normal { repeat, .. }, Char('W')) => {
            state.with_mode(|mode| mode.with_count(None));
            state.try_with_cursor(|cur, buffer| {
                cur.iter::<_, Word>(buffer).take(repeat.unwrap_or(1)).last()
            });
        },

        (Normal { repeat, .. }, Char('B')) => {
            state.with_mode(|mode| mode.with_count(None));
            state.try_with_cursor({
                |cur, buffer| cur.iter::<_, Word>(buffer).rev().take(repeat.unwrap_or(1)).last()
            });
        },

        (Normal { repeat, .. }, Char('h'))
        | (Select { repeat, .. }, Char('h'))
        | (Normal { repeat, .. }, Key::Left)
        | (Select { repeat, .. }, Key::Left) => {
            state.with_mode(|mode| mode.with_count(None));
            state.try_with_cursor({
                |cur, buffer| cur.iter::<_, Bounded>(buffer).rev().take(repeat.unwrap_or(1)).last()
            });
        },

        (Normal { repeat, .. }, Char('j'))
        | (Select { repeat, .. }, Char('j'))
        | (Normal { repeat, .. }, Key::Down)
        | (Select { repeat, .. }, Key::Down) => {
            state.with_mode(|mode| mode.with_count(None));
            state.try_with_cursor({
                |cur, buffer| cur.iter::<_, Line>(buffer).take(repeat.unwrap_or(1)).last()
            });
        },

        (Normal { repeat, .. }, Char('k'))
        | (Select { repeat, .. }, Char('k'))
        | (Normal { repeat, .. }, Key::Up)
        | (Select { repeat, .. }, Key::Up) => {
            state.with_mode(|mode| mode.with_count(None));
            state.try_with_cursor(|cur, buffer| {
                cur.iter::<_, Line>(buffer).rev().take(repeat.unwrap_or(1)).last()
            });
        },

        (Normal { repeat, .. }, Char('l'))
        | (Select { repeat, .. }, Char('l'))
        | (Normal { repeat, .. }, Key::Right)
        | (Select { repeat, .. }, Key::Right) => {
            state.with_mode(|mode| mode.with_count(None));
            state.try_with_cursor({
                |cur, buffer| cur.iter::<_, Bounded>(buffer).take(repeat.unwrap_or(1)).last()
            });
        },

        (Normal { repeat, .. }, Char('{')) | (Select { repeat, .. }, Char('{')) => {
            state.with_mode(|mode| mode.with_count(None));
            state.try_with_cursor(|cur, buffer| {
                cur.iter::<_, Paragraph>(buffer).rev().take(repeat.unwrap_or(1)).last()
            });
        },

        (Normal { repeat, .. }, Char('}')) | (Select { repeat, .. }, Char('}')) => {
            state.with_mode(|mode| mode.with_count(None));
            state.try_with_cursor({
                |cur, buffer| cur.iter::<_, Paragraph>(buffer).take(repeat.unwrap_or(1)).last()
            });
        },

        (Edit { .. }, Key::Left) => {
            state.try_with_cursor(|cur, buffer| cur.iter::<_, Bounded>(buffer).next())
        },
        (Edit { .. }, Key::Up) => {
            state.try_with_cursor(|cur, buffer| cur.iter::<_, Line>(buffer).next_back())
        },

        (Edit { .. }, Key::Down) => {
            state.try_with_cursor(|cur, buffer| cur.iter::<_, Line>(buffer).next())
        },
        (Edit { .. }, Key::Right) => {
            state.try_with_cursor(|cur, buffer| cur.iter::<_, Bounded>(buffer).next_back())
        },

        (Normal { repeat }, Char(ch @ '0'..='9'))
        | (Select { repeat, .. }, Char(ch @ '0'..='9'))
        | (Operator { repeat, .. }, Char(ch @ '0'..='9')) => {
            let delta = ch.to_digit(10).unwrap() as usize;
            let n = repeat.or(Some(0)).map(|repeat| repeat * 10 + delta);

            state.with_mode(|mode| mode.with_count(n))
        },

        (Operator { .. }, _) => state.with_mode(|_| Mode::default()),

        _ => (),
    };

    Ok(())
}
