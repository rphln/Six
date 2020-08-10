use std::error::Error;
use std::ops::RangeBounds;
use std::sync::Arc;

use rlua::{Context, Lua, UserData, UserDataMethods};

use crate::buffer::{Buffer, View};
use crate::cursor::{Bounded, Char, Cursor, EndOfWord, Line, Points, Word};

// TODO: Replace with a trait alias once it stabilizes.
pub trait Callback<Arg>:
    Fn(&mut Editor, Arg) -> Result<(), Box<dyn Error>> + Send + Sync + 'static
{
}

#[derive(Debug, Default)]
pub struct Editor {
    /// Current content.
    content: Buffer,

    /// Cursor position in the content.
    cursor: Cursor,

    /// The editor operation mode.
    mode: Mode,
}

#[derive(Clone, Derivative)]
#[derivative(Debug, Default)]
pub enum Mode {
    /// The default editor mode.
    #[derivative(Default)]
    Normal,

    /// The text input mode.
    Edit,

    /// Queries the user for a text range.
    Select {
        /// The fixed point of the selection.
        anchor: Cursor,
    },

    /// Queries the user for a text object and applies an operation.
    Operator {
        /// The prompt displayed to the user.
        prompt: &'static str,

        /// Callback invoked once a text object has been provided.
        #[derivative(Debug = "ignore")]
        callback: Arc<dyn Callback<(Cursor, Cursor)>>,
    },

    /// Queries the user for a text input and applies an operation.
    Query {
        /// The prompt displayed to the user.
        prompt: &'static str,

        /// The maximum length of the queried string.
        length: Option<usize>,

        /// The content of the query.
        content: Buffer,

        /// Cursor position in the content.
        cursor: Cursor,

        /// Callback invoked once the input has finished.
        #[derivative(Debug = "ignore")]
        callback: Arc<dyn for<'r> Callback<&'r str>>,
    },
}

impl Editor {
    /// Returns a view to the text buffer.
    #[inline]
    #[must_use]
    pub fn content(&self) -> &impl View {
        // TODO: Return a reference.
        &self.content
    }

    /// Returns a copy of the cursor position.
    #[inline]
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Returns a reference to the mode.
    #[inline]
    #[must_use]
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Returns to the default mode.
    #[inline]
    pub fn escape(&mut self) {
        self.mode = Mode::default();
    }

    /// Enters the text edit mode.
    #[inline]
    pub fn edit(&mut self) {
        self.mode = Mode::Edit;
    }

    /// Enters the selection mode.
    #[inline]
    pub fn select(&mut self) {
        self.mode = Mode::Select { anchor: self.cursor }
    }

    /// Queries the user for a text input.
    #[inline]
    pub fn query(
        &mut self,
        prompt: &'static str,
        length: Option<usize>,
        callback: impl for<'r> Callback<&'r str>,
    ) {
        self.mode = Mode::Query {
            length,
            prompt,

            callback: Arc::new(callback),

            content: Buffer::default(),
            cursor: Cursor::default(),
        };
    }

    /// Returns a mutable reference to the query contents.
    ///
    /// # Panics
    ///
    /// Panics if the editor is not in the `Query` mode.
    #[must_use]
    pub fn query_content_mut(&mut self) -> &mut Buffer {
        if let Mode::Query { ref mut content, .. } = self.mode {
            content
        } else {
            panic!("Attempt to call `query_content_mut` in an incompatible mode.")
        }
    }

    /// Completes a text query.
    ///
    /// # Panics
    ///
    /// Panics if the editor is not in the `Query` mode.
    ///
    /// # Errors
    ///
    /// Forwards any errors from the callback.
    pub fn after_query(&mut self, text: &str) -> Result<(), Box<dyn Error>> {
        if let Mode::Query { ref callback, .. } = self.mode {
            callback.clone()(self, text)
        } else {
            panic!("Attempt to call `after_query` in an incompatible mode.")
        }
    }

    /// Queries the user for a text object.
    #[inline]
    pub fn operator(&mut self, prompt: &'static str, callback: impl Callback<(Cursor, Cursor)>) {
        self.mode = Mode::Operator { prompt, callback: Arc::new(callback) }
    }

    /// Completes an operator query.
    ///
    /// # Panics
    ///
    /// Panics if the editor is not in the `Operator` mode.
    ///
    /// # Errors
    ///
    /// Forwards any errors from the callback.
    pub fn after_operator(&mut self, p: Cursor, q: Cursor) -> Result<(), Box<dyn Error>> {
        if let Mode::Operator { ref callback, .. } = self.mode {
            callback.clone()(self, (p, q))
        } else {
            panic!("Attempt to call `after_operator` in an incompatible mode.")
        }
    }

    /// Sets the cursor position.
    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }

    /// Moves the cursor forward up to the specified units according to the specified unit.
    pub fn forward<'a, M>(&'a mut self, n: usize)
    where
        Points<'a, Buffer, M>: Iterator<Item = Cursor>,
    {
        self.cursor = self.cursor.iter::<_, M>(&self.content).take(n).last().unwrap_or(self.cursor);
    }

    /// Moves the cursor forward the last position according to the specified unit.
    pub fn last<'a, M>(&'a mut self)
    where
        Points<'a, Buffer, M>: Iterator<Item = Cursor>,
    {
        self.cursor = self.cursor.iter::<_, M>(&self.content).last().unwrap_or(self.cursor);
    }

    /// Moves the cursor backward up to the specified units according to the specified unit.
    pub fn backward<'a, M>(&'a mut self, n: usize)
    where
        Points<'a, Buffer, M>: DoubleEndedIterator<Item = Cursor>,
    {
        self.cursor =
            self.cursor.iter::<_, M>(&self.content).rev().take(n).last().unwrap_or(self.cursor);
    }

    /// Moves the cursor backward the last position according to the specified unit.
    pub fn first<'a, M>(&'a mut self)
    where
        Points<'a, Buffer, M>: DoubleEndedIterator<Item = Cursor>,
    {
        self.cursor = self.cursor.iter::<_, M>(&self.content).rev().last().unwrap_or(self.cursor);
    }

    /// Inserts a character at the specified position.
    pub fn insert(&mut self, at: Cursor, ch: char) {
        self.content.insert(at, ch);
    }

    /// Removes the specified range in the buffer, and replaces it with the given string.
    #[inline]
    pub fn replace_range(&mut self, range: impl RangeBounds<Cursor>, text: &str) {
        self.content.edit(range, text);
    }

    /// Evaluates a Lua expression.
    ///
    /// # Errors
    ///
    /// Forwards any errors produced by the upstream.
    pub fn eval<Output: UserData + Clone + 'static>(
        &mut self,
        context: &mut Context,
        program: &str,
    ) -> rlua::Result<Output> {
        context.scope(|scope| {
            context.globals().set("state", scope.create_nonstatic_userdata(self)?)?;
            context.load(program).eval()
        })
    }
}

impl UserData for Cursor {}

impl UserData for &mut Editor {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("insert_at_cursor", |_, state, text: String| {
            state.replace_range(state.cursor..state.cursor, text.as_str());
            Ok(())
        });

        methods.add_method("cursor", |_, state, ()| Ok(state.cursor));

        methods.add_method_mut("delete", |_, state, at: Cursor| {
            state.replace_range(at..=at, "");
            Ok(())
        })
    }
}

/// Built-in text objects.
#[derive(Derivative)]
#[derivative(Debug)]
pub enum TextObject {
    Word,
    Bol,
    Eow,
}

/// Built-in editor actions.
#[derive(Derivative)]
#[derivative(Debug)]
pub enum Action {
    /// Returns to the `Normal` mode.
    ///
    /// If `backward` is set, move the cursor backward by a character within a line.
    Escape { backward: bool },

    /// Enters the `Insert` mode.
    ///
    /// If `forward` is set, move the cursor forward by a character within a line.
    ToInsert { forward: bool },

    /// Moves the cursor to the end of the line and enters the `Insert` mode.
    ToInsertEol,

    /// Moves the cursor to the beginning of the line and enters the `Insert` mode.
    ToInsertBol,

    /// Begins a new line below the cursor and enters the `Insert` mode.
    ToInsertBelow,

    /// Begins a new line above the cursor and enters the `Insert` mode.
    ToInsertAbove,

    /// Queries the user to delete a text range.
    ToDelete,

    /// Queries the user to surround a text range.
    ToSurround,

    /// Queries the user for an expression to evaluate.
    ToEval,

    /// Enters the `Select` mode.
    ToSelect,

    /// Inserts the character at the cursor, and then moves the cursor forward.
    #[derivative(Debug = "transparent")]
    Input(char),

    /// Deletes the character at the cursor.
    ///
    /// If `backward` is set, moves the cursor backward.
    Delete { backward: bool },

    /// Uses a text object.
    #[derivative(Debug = "transparent")]
    TextObject(TextObject),
}

impl<F, Arg> Callback<Arg> for F where
    F: Fn(&mut Editor, Arg) -> Result<(), Box<dyn Error>> + Send + Sync + 'static
{
}

pub fn handle_key(state: &mut Editor, _: &mut Lua, action: Action) -> Result<(), Box<dyn Error>> {
    match action {
        Action::Escape { backward } => {
            state.escape();

            if backward {
                state.backward::<Bounded>(1);
            }
        },

        Action::ToInsert { forward } => {
            state.edit();

            if forward {
                state.forward::<Bounded>(1);
            }
        },

        Action::ToEval {} => state.query("Eval", None, |state: &mut Editor, command: &str| {
            state.escape();
            Ok(())
        }),

        Action::ToSurround { .. } => {
            let surround = |state: &mut Editor, (start, end): (Cursor, Cursor)| {
                // Replicates `vim-surround` by skipping any whitespaces at the end.
                let buf = state.content();
                let end = end
                    .iter::<_, EndOfWord>(buf)
                    .rev()
                    .next()
                    .and_then(|end| end.iter::<_, Char>(buf).next())
                    .unwrap_or(end);

                let surround = move |state: &mut Editor, sandwich: &str| {
                    let mut chars = sandwich.chars();

                    let prefix = chars.next().expect("prefix");
                    let suffix = chars.next().expect("suffix");

                    state.insert(end, suffix);
                    state.insert(start, prefix);

                    state.escape();
                    state.set_cursor(start);

                    Ok(())
                };

                state.query("Surround with", Some(2), surround);

                Ok(())
            };

            state.operator("Surround", surround);
        },

        Action::Input(ch) => {
            if let Mode::Query { length, mut content, cursor, .. } = state.mode.clone() {
                content.insert(cursor, ch);

                if ch == '\n' || length.map_or(false, |length| length == content.len()) {
                    state.after_query(content.as_str())?;
                } else {
                    let next_cursor = cursor.iter::<_, Char>(&content).next().expect("next");
                    let next_content = content;

                    if let Mode::Query { ref mut cursor, ref mut content, .. } = state.mode {
                        *cursor = next_cursor;
                        *content = next_content;
                    }
                }
            } else {
                state.insert(state.cursor, ch);
                state.forward::<Char>(1);
            }
        },

        Action::TextObject(TextObject::Bol) => {
            let end = state.cursor();
            state.first::<Bounded>();

            if let Mode::Operator { .. } = state.mode {
                state.after_operator(state.cursor(), end)?;
            }
        },

        Action::TextObject(TextObject::Word) => {
            let start = state.cursor();
            state.forward::<Word>(1);

            if let Mode::Operator { .. } = state.mode {
                state.after_operator(start, state.cursor())?;
            }
        },

        Action::TextObject(TextObject::Eow) => {
            let start = state.cursor();
            state.forward::<EndOfWord>(1);

            if let Mode::Operator { .. } = state.mode {
                state.after_operator(start, state.cursor())?;
            }
        },

        _ => todo!(),
    };

    Ok(())
}
// match (state.mode().clone(), event) {
//     (Edit, Esc) => {
//         state.try_with_cursor(|cur, buffer| cur.iter::<_, Bounded>(buffer).next_back());

//         state.with_mode(|_| Mode::normal(None));
//     },

//     (_, Esc) => state.with_mode(|_| Mode::normal(None)),

//     (Normal { .. }, Char('v')) => {
//         let anchor = state.cursor();
//         state.with_mode(|_| Select { anchor, repeat: None });
//     },

//     (Normal { .. }, Char('i')) => state.with_mode(|_| Edit),
//     (Normal { .. }, Char('a')) => {
//         state.with_mode(|_| Edit);
//         state.try_with_cursor(|cursor, buffer| cursor.iter::<_, Bounded>(buffer).next())
//     },

//     (Normal { .. }, Char('I')) => {
//         state.with_mode(|_| Edit);
//         state.try_with_cursor(|cursor, buffer| cursor.iter::<_,
// Bounded>(buffer).rev().last());     },
//     (Normal { .. }, Char('A')) => {
//         state.with_mode(|_| Edit);
//         state.try_with_cursor(|cursor, buffer| cursor.iter::<_, Bounded>(buffer).last());
//     },

//     (Normal { .. }, Char('o')) => {
//         let eol = state
//             .cursor()
//             .iter::<_, Bounded>(state.buffer())
//             .last()
//             .unwrap_or_else(|| state.cursor());
//         state.insert(eol, '\n');

//         state.try_with_cursor(|_, buffer| eol.iter::<_, Unbounded>(buffer).next());
//         state.with_mode(|_| Mode::Edit);
//     },

//     (Normal { .. }, Char('O')) => {
//         let bol = state
//             .cursor()
//             .iter::<_, Bounded>(state.buffer())
//             .rev()
//             .last()
//             .unwrap_or_else(|| state.cursor());
//         state.insert(bol, '\n');

//         state.try_with_cursor(|_, buffer| bol.iter::<_, Unbounded>(buffer).next_back());
//         state.with_mode(|_| Mode::Edit);
//     },

//     (Normal { .. }, Char('s')) => {
//         let surround = |state: &mut State, _lua: &mut Lua, (start, end): Range| {
//             // Replicates `vim-surround` by skipping any whitespaces at the end.
//             let buf = state.buffer();
//             let end = end
//                 .iter::<_, Unbounded>(buf)
//                 .rev()
//                 .find(|p| buf.get(p).map(|ch| !ch.is_whitespace()).unwrap_or(true))
//                 .unwrap_or(end);

//             let surround = move |state: &mut State, _lua: &mut Lua, sandwich: &str| {
//                 let mut chars = sandwich.chars();

//                 let prefix = chars.next().expect("prefix");
//                 let suffix = chars.next().expect("suffix");

//                 state.insert(end, suffix);
//                 state.insert(start, prefix);

//                 state.with_mode(|_| Mode::default());
//                 state.with_cursor(|_, _| start);

//                 Ok(())
//             };

//             state.with_mode(|_| Mode::query("Surround with", Some(2), surround));

//             Ok(())
//         };

//         state.with_mode(|_| Mode::operator("Surround", None, surround));
//     },

//     (Normal { repeat }, Char('d')) => {
//         state.with_mode(|_| {
//             Mode::operator(
//                 "Delete",
//                 repeat,
//                 |state: &mut State, _lua: &mut Lua, (start, end): Range| {
//                     state.with_cursor(|_, _| start);
//                     state.delete(start..=end);
//                     state.with_mode(|_| Mode::default());

//                     Ok(())
//                 },
//             )
//         });
//     },

//     (Normal { .. }, Char(';')) => {
//         state.with_mode(|_| {
//             Mode::query("Eval", None, |state: &mut State, lua: &mut Lua, program: &str| {
//                 state.with_mode(|_| Mode::default());

//                 lua.context(|ctx| {
//                     ctx.scope(|scope| {
//                         ctx.globals().set("state", scope.create_nonstatic_userdata(state)?)?;

//                         ctx.load(program).exec()
//                     })
//                 })?;

//                 Ok(())
//             })
//         });
//     },

//     (Normal { .. }, Char('u')) => state.with_mode(|_| {
//         Mode::query(
//             "Eval & Forward",
//             None,
//             |state: &mut State, lua: &mut Lua, program: &str| {
//                 state.with_mode(|_| Mode::default());

//                 state.try_with_cursor(|cursor, buffer| {
//                     lua.context::<_, Option<Cursor>>(|ctx| {
//                         ctx.scope(|scope| {
//                             let iter = cursor.iter::<_, Unbounded>(buffer);
//                             let iter = scope.create_nonstatic_userdata(iter);

//                             ctx.load(program)
//                                 .eval::<Function>()
//                                 .unwrap()
//                                 .call::<_, Option<Cursor>>(iter)
//                                 .unwrap()
//                         })
//                     })
//                 });

//                 Ok(())
//             },
//         )
//     }),

//     (Normal { repeat }, Char('t')) => state.with_mode(|_| {
//         Mode::query("Jump to next", Some(1), move |state: &mut State, _: &mut Lua, ch: &str|
// {             let ch = ch.chars().next().expect("ch");

//             state.try_with_cursor(|cur, buf| {
//                 let repeat = repeat.unwrap_or(1);

//                 let mut it = cur.iter::<_, Bounded>(buf);
//                 (1..=repeat).fold(None, |_, _| {
//                     it.find(|p| {
//                         p.iter::<_, Bounded>(buf)
//                             .next()
//                             .map(|p| buf.get(p).map(|other| other == ch).expect("get"))
//                             .unwrap_or(false)
//                     })
//                 })
//             });

//             state.with_mode(|_| Normal { repeat: None });

//             Ok(())
//         })
//     }),

//     (Operator { ref callback, repeat, .. }, Char('W')) => {
//         let start = state.cursor();
//         let end = start
//             .iter::<_, Word>(state.buffer())
//             .take(repeat.unwrap_or(1))
//             .last()
//             .expect("end");

//         callback(state, lua, (start, end))?;
//     },

//     (Operator { callback, repeat, .. }, Char('B')) => {
//         let end = state.cursor();
//         let start = end
//             .iter::<_, Word>(state.buffer())
//             .rev()
//             .take(repeat.unwrap_or(1))
//             .last()
//             .expect("start");

//         callback(state, lua, (start, end))?;
//     },

//     (Query { mut partial, callback, length, .. }, Char(ch)) => {
//         let at = partial.cursor();
//         partial.buffer_mut().insert(at, ch);

//         partial.try_with_cursor(|cur, buffer| cur.iter::<_, Unbounded>(buffer).next());

//         match length {
//             _ if ch == '\n' => {
//                 callback(state, lua, partial.buffer().as_str())?;
//             },

//             Some(length) if length == partial.buffer().len() => {
//                 callback(state, lua, partial.buffer().as_str())?;
//             },

//             _ => {
//                 state.with_mode(|mode| mode.with_partial(|p| *p = partial));
//             },
//         };
//     },

//     (Query { .. }, Backspace) => {
//         state.with_mode(|mode| {
//             mode.with_partial(|partial| {
//                 partial.try_with_cursor(|cur, buf| cur.iter::<_,
// Unbounded>(buf).next_back());

//                 let cursor = partial.cursor();
//                 if partial.buffer().get(cursor).is_some() {
//                     partial.buffer_mut().delete(cursor..=cursor);
//                 }
//             })
//         });
//     },

//     (Edit, Char(ch)) => {
//         state.insert(state.cursor(), ch);
//         state.try_with_cursor(|cur, buffer| cur.iter::<_, Unbounded>(buffer).next());
//     },

//     (Edit, Backspace) => {
//         state.try_with_cursor(|cur, buffer| cur.iter::<_, Unbounded>(buffer).next_back());

//         let cursor = state.cursor();
//         if state.buffer().get(cursor).is_some() {
//             state.delete(cursor..=cursor)
//         }
//     },

//     (Edit, Delete) => {
//         let cursor = state.cursor();
//         if state.buffer().get(cursor).is_some() {
//             state.delete(cursor..=cursor)
//         }
//     },

//     (Edit, Ctrl('w')) => {
//         let anchor = state.cursor();

//         state.try_with_cursor(|cur, buffer| cur.iter::<_, Word>(buffer).next_back());
//         state.delete(state.cursor()..anchor);
//     },

//     (Normal { repeat, .. }, Char('W')) => {
//         state.with_mode(|mode| mode.with_count(None));
//         state.try_with_cursor(|cur, buffer| {
//             cur.iter::<_, Word>(buffer).take(repeat.unwrap_or(1)).last()
//         });
//     },

//     (Normal { repeat, .. }, Char('B')) => {
//         state.with_mode(|mode| mode.with_count(None));
//         state.try_with_cursor({
//             |cur, buffer| cur.iter::<_, Word>(buffer).rev().take(repeat.unwrap_or(1)).last()
//         });
//     },

//     (Normal { repeat, .. }, Char('h'))
//     | (Select { repeat, .. }, Char('h'))
//     | (Normal { repeat, .. }, Key::Left)
//     | (Select { repeat, .. }, Key::Left) => {
//         state.with_mode(|mode| mode.with_count(None));
//         state.try_with_cursor({
//             |cur, buffer| cur.iter::<_,
// Bounded>(buffer).rev().take(repeat.unwrap_or(1)).last()         });
//     },

//     (Normal { repeat, .. }, Char('j'))
//     | (Select { repeat, .. }, Char('j'))
//     | (Normal { repeat, .. }, Key::Down)
//     | (Select { repeat, .. }, Key::Down) => {
//         state.with_mode(|mode| mode.with_count(None));
//         state.try_with_cursor({
//             |cur, buffer| cur.iter::<_, Line>(buffer).take(repeat.unwrap_or(1)).last()
//         });
//     },

//     (Normal { repeat, .. }, Char('k'))
//     | (Select { repeat, .. }, Char('k'))
//     | (Normal { repeat, .. }, Key::Up)
//     | (Select { repeat, .. }, Key::Up) => {
//         state.with_mode(|mode| mode.with_count(None));
//         state.try_with_cursor(|cur, buffer| {
//             cur.iter::<_, Line>(buffer).rev().take(repeat.unwrap_or(1)).last()
//         });
//     },

//     (Normal { repeat, .. }, Char('l'))
//     | (Select { repeat, .. }, Char('l'))
//     | (Normal { repeat, .. }, Key::Right)
//     | (Select { repeat, .. }, Key::Right) => {
//         state.with_mode(|mode| mode.with_count(None));
//         state.try_with_cursor({
//             |cur, buffer| cur.iter::<_, Bounded>(buffer).take(repeat.unwrap_or(1)).last()
//         });
//     },

//     (Normal { repeat, .. }, Char('{')) | (Select { repeat, .. }, Char('{')) => {
//         state.with_mode(|mode| mode.with_count(None));
//         state.try_with_cursor(|cur, buffer| {
//             cur.iter::<_, Paragraph>(buffer).rev().take(repeat.unwrap_or(1)).last()
//         });
//     },

//     (Normal { repeat, .. }, Char('}')) | (Select { repeat, .. }, Char('}')) => {
//         state.with_mode(|mode| mode.with_count(None));
//         state.try_with_cursor({
//             |cur, buffer| cur.iter::<_, Paragraph>(buffer).take(repeat.unwrap_or(1)).last()
//         });
//     },

//     (Edit { .. }, Key::Left) => {
//         state.try_with_cursor(|cur, buffer| cur.iter::<_, Bounded>(buffer).next())
//     },
//     (Edit { .. }, Key::Up) => {
//         state.try_with_cursor(|cur, buffer| cur.iter::<_, Line>(buffer).next_back())
//     },

//     (Edit { .. }, Key::Down) => {
//         state.try_with_cursor(|cur, buffer| cur.iter::<_, Line>(buffer).next())
//     },
//     (Edit { .. }, Key::Right) => {
//         state.try_with_cursor(|cur, buffer| cur.iter::<_, Bounded>(buffer).next_back())
//     },

//     (Normal { repeat }, Char(ch @ '0'..='9'))
//     | (Select { repeat, .. }, Char(ch @ '0'..='9'))
//     | (Operator { repeat, .. }, Char(ch @ '0'..='9')) => {
//         let delta = ch.to_digit(10).unwrap() as usize;
//         let n = repeat.or(Some(0)).map(|repeat| repeat * 10 + delta);

//         state.with_mode(|mode| mode.with_count(n))
//     },

//     (Operator { .. }, _) => state.with_mode(|_| Mode::default()),

//     _ => (),
// };

//     Ok(())
// }
