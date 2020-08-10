use std::error::Error;
use std::ops::RangeBounds;
use std::sync::Arc;

use rlua::{Context, Lua, UserData, UserDataMethods};

use crate::buffer::Buffer;
use crate::cursor::{Bounded, Cursor, EndOfWord, Iter, Line, Word};

// TODO: Replace with a trait alias once it stabilizes.
pub trait Callback<Arg>:
    Fn(&mut Editor, &mut Lua, Arg) -> Result<(), Box<dyn Error>> + Send + Sync + 'static
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
    pub fn content(&self) -> &Buffer {
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
    pub fn after_query(&mut self, lua: &mut Lua, text: &str) -> Result<(), Box<dyn Error>> {
        if let Mode::Query { ref callback, .. } = self.mode {
            callback.clone()(self, lua, text)
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
    pub fn after_operator(
        &mut self,
        lua: &mut Lua,
        start: Cursor,
        end: Cursor,
    ) -> Result<(), Box<dyn Error>> {
        if let Mode::Operator { ref callback, .. } = self.mode {
            callback.clone()(self, lua, (start, end))
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
        Iter<'a, M>: Iterator<Item = Cursor>,
    {
        self.cursor = self
            .cursor
            .iter::<M>(&self.content)
            .take(n)
            .last()
            .or_else(|| {
                // FIXME: This is, of course, horribly inefficient.
                // Also, we probably should accept this lambda as an argument instead.
                self.cursor.iter::<char>(&self.content).last()
            })
            .unwrap_or(self.cursor);
    }

    /// Moves the cursor forward the last position according to the specified unit.
    pub fn last<'a, M>(&'a mut self)
    where
        Iter<'a, M>: Iterator<Item = Cursor>,
    {
        self.cursor = self
            .cursor
            .iter::<M>(&self.content)
            .last()
            .or_else(|| {
                // FIXME: This is, of course, horribly inefficient.
                // Also, we probably should accept this lambda as an argument instead.
                self.cursor.iter::<char>(&self.content).last()
            })
            .unwrap_or(self.cursor);
    }

    /// Moves the cursor backward up to the specified units according to the specified unit.
    pub fn backward<'a, M>(&'a mut self, n: usize)
    where
        Iter<'a, M>: DoubleEndedIterator<Item = Cursor>,
    {
        self.cursor = self
            .cursor
            .iter::<M>(&self.content)
            .rev()
            .take(n)
            .last()
            .or_else(|| {
                // FIXME: This is, of course, horribly inefficient.
                // Also, we probably should accept this lambda as an argument instead.
                self.cursor.iter::<char>(&self.content).rev().last()
            })
            .unwrap_or(self.cursor);
    }

    /// Moves the cursor backward the last position according to the specified unit.
    pub fn first<'a, M>(&'a mut self)
    where
        Iter<'a, M>: DoubleEndedIterator<Item = Cursor>,
    {
        self.cursor = self
            .cursor
            .iter::<M>(&self.content)
            .rev()
            .last()
            .or_else(|| {
                // FIXME: This is, of course, horribly inefficient.
                // Also, we probably should accept this lambda as an argument instead.
                self.cursor.iter::<char>(&self.content).rev().last()
            })
            .unwrap_or(self.cursor);
    }

    /// Inserts a character at the specified position.
    pub fn insert(&mut self, at: Cursor, ch: char) {
        self.content.insert(at.to_offset(&self.content), ch);
    }

    /// Removes the specified range in the buffer, and replaces it with the given string.
    #[inline]
    pub fn replace_range(&mut self, range: impl RangeBounds<Cursor>, text: &str) {
        use std::ops::Bound::{Excluded, Included, Unbounded};

        let start = match range.start_bound() {
            Included(ref start) => Included(start.to_offset(&self.content)),
            Excluded(ref start) => Excluded(start.to_offset(&self.content)),
            Unbounded => Unbounded,
        };

        let end = match range.start_bound() {
            Included(ref end) => Included(end.to_offset(&self.content)),
            Excluded(ref end) => Excluded(end.to_offset(&self.content)),
            Unbounded => Unbounded,
        };

        self.content.edit((start, end), text);
    }

    /// Executes a Lua program.
    ///
    /// # Errors
    ///
    /// Forwards any errors produced by the upstream.
    pub fn exec(&mut self, lua: &mut Lua, program: &str) -> rlua::Result<()> {
        lua.context(|context| {
            context.scope(|scope| {
                context.globals().set("state", scope.create_nonstatic_userdata(self)?)?;
                context.load(program).exec()
            })
        })
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

    Up,
    Down,
    Left,
    Right,
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

    /// Queries the user to surround a text range with a format string.
    ToSandwich,

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
    F: Fn(&mut Editor, &mut Lua, Arg) -> Result<(), Box<dyn Error>> + Send + Sync + 'static
{
}

// TODO: Split this into a bunch of functions.
pub fn handle_key(state: &mut Editor, lua: &mut Lua, action: Action) -> Result<(), Box<dyn Error>> {
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

        Action::ToEval {} => {
            state.query("Exec", None, |state: &mut Editor, lua: &mut Lua, program: &str| {
                state.escape();
                state.exec(lua, program)?;

                Ok(())
            });
        },

        Action::ToSurround { .. } => {
            let surround = |state: &mut Editor, _: &mut Lua, (start, end): (Cursor, Cursor)| {
                // Replicates `vim-surround` by skipping any spaces at the end.
                let buf = state.content();
                let end = end
                    .iter::<EndOfWord>(buf)
                    .rev()
                    .next()
                    .and_then(|end| end.iter::<char>(buf).next())
                    .unwrap_or(end);

                let surround = move |state: &mut Editor, _: &mut Lua, sandwich: &str| {
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
                content.insert(cursor.to_offset(&content), ch);

                if ch == '\n' || length.map_or(false, |length| length == content.len()) {
                    state.after_query(lua, content.as_str())?;
                } else {
                    let next_cursor = cursor.iter::<char>(&content).next().expect("next");
                    let next_content = content;

                    if let Mode::Query { ref mut cursor, ref mut content, .. } = state.mode {
                        *cursor = next_cursor;
                        *content = next_content;
                    }
                }
            } else {
                state.insert(state.cursor, ch);
                state.forward::<char>(1);
            }
        },

        Action::TextObject(TextObject::Bol) => {
            let end = state.cursor();
            state.first::<Bounded>();

            if let Mode::Operator { .. } = state.mode {
                state.after_operator(lua, state.cursor(), end)?;
            }
        },

        Action::TextObject(TextObject::Word) => {
            let start = state.cursor();
            state.forward::<Word>(1);

            if let Mode::Operator { .. } = state.mode {
                state.after_operator(lua, start, state.cursor())?;
            }
        },

        Action::TextObject(TextObject::Eow) => {
            let start = state.cursor();
            state.forward::<EndOfWord>(1);

            if let Mode::Operator { .. } = state.mode {
                state.after_operator(lua, start, state.cursor())?;
            }
        },

        Action::TextObject(TextObject::Left) => {
            let end = state.cursor();
            state.backward::<Bounded>(1);

            if let Mode::Operator { .. } = state.mode {
                state.after_operator(lua, state.cursor(), end)?;
            }
        },

        Action::TextObject(TextObject::Up) => {
            let end = state.cursor();
            state.backward::<Line>(1);

            if let Mode::Operator { .. } = state.mode {
                state.after_operator(lua, state.cursor(), end)?;
            }
        },

        Action::TextObject(TextObject::Down) => {
            let start = state.cursor();
            state.forward::<Line>(1);

            if let Mode::Operator { .. } = state.mode {
                state.after_operator(lua, start, state.cursor())?;
            }
        },

        Action::TextObject(TextObject::Right) => {
            let start = state.cursor();
            state.forward::<Bounded>(1);

            if let Mode::Operator { .. } = state.mode {
                state.after_operator(lua, start, state.cursor())?;
            }
        },

        _ => todo!(),
    };

    Ok(())
}
