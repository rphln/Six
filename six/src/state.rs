use std::ops::{Bound, RangeBounds};

use rlua::{Lua, UserData, UserDataMethods};

use crate::cursor::{Codepoint, Cursor, Iter};
use crate::event::Key;
use crate::mode::{Advance, Context, Keymap, Mode, Operation};

/// An modal editor.
#[derive(Default, Derivative)]
#[derivative(Debug)]
pub struct Editor {
    /// The active mode.
    mode: Mode,

    /// The text buffer.
    buffer: Buffer,

    /// The editor message log.
    messages: Vec<String>,

    /// The Lua engine.
    #[derivative(Debug = "ignore")]
    lua: Lua,
}

/// The mutable buffer of an editor.
#[derive(Clone, Debug, Default)]
pub struct Buffer {
    /// The text content.
    content: String,

    /// The cursor position.
    cursor: Cursor,
}

impl Buffer {
    /// Returns the cursor position.
    #[inline]
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Sets the cursor position.
    ///
    /// Returns the old value.
    pub fn set_cursor(&mut self, cursor: Cursor) -> Cursor {
        std::mem::replace(&mut self.cursor, cursor)
    }

    /// Returns a reference to buffer contents.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.content.as_str()
    }

    /// Inserts a character at the specified cursor position.
    pub fn insert(&mut self, ch: char, at: Cursor) {
        self.content.insert(at.offset(), ch);
    }

    /// Inserts a character at the cursor position, and then moves the cursor forward.
    pub fn append(&mut self, ch: char) {
        self.insert(ch, self.cursor);
        self.forward::<Codepoint>().expect("forward");
    }

    /// Attempts to move the cursor forward over a given metric.
    ///
    /// Returns the previous position on success.
    pub fn forward<'a, It: Iter<'a>>(&'a mut self) -> Option<Cursor> {
        let cursor = self.cursor.forward::<It>(self.content.as_str())?;
        std::mem::replace(&mut self.cursor, cursor).into()
    }

    /// Attempts to move the cursor backward over a given metric.
    ///
    /// Returns the previous position on success.
    pub fn backward<'a, It: Iter<'a>>(&'a mut self) -> Option<Cursor> {
        let cursor = self.cursor.backward::<It>(self.content.as_str())?;
        std::mem::replace(&mut self.cursor, cursor).into()
    }

    /// Replaces the text in a range.
    ///
    /// The length of the range can differ from the replacement's.
    pub fn edit(&mut self, text: &str, range: impl RangeBounds<Cursor>) {
        use Bound::{Excluded, Included, Unbounded};

        let start = match range.start_bound() {
            Included(&s) => Included(s.offset()),
            Excluded(&s) => Excluded(s.offset()),
            Unbounded => Unbounded,
        };

        let end = match range.end_bound() {
            Included(&e) => Included(e.offset()),
            Excluded(&e) => Excluded(e.offset()),
            Unbounded => Unbounded,
        };

        self.content.replace_range((start, end), text)
    }
}

impl UserData for Cursor {}

impl UserData for &mut Buffer {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("insert_at_cursor", |_, state, text: String| {
            state.edit(text.as_str(), state.cursor..state.cursor);
            Ok(())
        });

        methods.add_method("cursor", |_, state, ()| Ok(state.cursor));

        methods.add_method_mut("delete", |_, state, at: Cursor| {
            state.edit("", at..=at);
            Ok(())
        })
    }
}

impl Editor {
    /// Returns a reference to the buffer mode.
    #[must_use]
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Returns the cursor position.
    #[inline]
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.buffer.cursor()
    }

    /// Returns a reference to the content contents.
    #[inline]
    #[must_use]
    pub fn content(&self) -> &str {
        self.buffer.as_str()
    }

    // TODO: Replace `Key` with `Operation`.
    pub fn handle_key(&mut self, key: Key) {
        // TODO: Replace with a dynamic keymap lookup.
        #[rustfmt::skip]
        let operations = match (self.mode().keymap(), key) {
            (keymap, Key::Char(ch)) if keymap.intersects(Keymap::INSERT | Keymap::QUERY)     => vec![Operation::Input(ch)],

            (keymap, Key::Char('h')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Left],
            (keymap, Key::Char('j')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Down],
            (keymap, Key::Char('k')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Up],
            (keymap, Key::Char('l')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Right],

            (keymap, Key::Char('W')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Head { reverse: true }],
            (keymap, Key::Char('w')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Head { reverse: false }],

            (keymap, Key::Char('E')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Tail { reverse: true }],
            (keymap, Key::Char('e')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Tail { reverse: false }],

            (keymap, Key::Char('^')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Bol],
            (keymap, Key::Char('$')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Eol],

            (keymap, Key::Char('a')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Insert, Operation::Right],
            (keymap, Key::Char('i')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Insert],

            (keymap, Key::Char(';')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Eval],

            (keymap, Key::Char('s')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Surround],
            (keymap, Key::Char('d')) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY)) => vec![Operation::Delete],

            (keymap, Key::Delete) if keymap.intersects(!(Keymap::INSERT | Keymap::QUERY))    => vec![Operation::Delete],

            (keymap, Key::Esc) if keymap.intersects(!Keymap::INSERT)                         => vec![Operation::Escape],

            (Keymap::INSERT, Key::Backspace)                                                 => vec![Operation::Delete, Operation::Left],
            (Keymap::INSERT, Key::Delete)                                                    => vec![Operation::Delete, Operation::Right],
            (Keymap::INSERT, Key::Esc)                                                       => vec![Operation::Escape, Operation::Left],

            (Keymap::QUERY, Key::Backspace)                                                  => vec![Operation::Left, Operation::Delete],
            (Keymap::QUERY, Key::Delete)                                                     => vec![Operation::Delete],

            (_, Key::Down)                                                                   => vec![Operation::Down],
            (_, Key::Left)                                                                   => vec![Operation::Left],
            (_, Key::Right)                                                                  => vec![Operation::Right],
            (_, Key::Up)                                                                     => vec![Operation::Up],

            (_, _)                                                                           => vec![],
        };

        self.advance(operations.as_slice())
    }

    /// Advances the buffer buffer by handling events.
    pub fn advance(&mut self, operations: &[Operation]) {
        let mut mode = std::mem::take(&mut self.mode);

        for &operation in operations {
            let context = Context::new(&mut self.buffer, &mut self.lua, &mut self.messages);

            match mode.advance(context, operation) {
                Advance::Continue(next) => mode = next,
                Advance::Halt(next) => {
                    mode = next;
                    break;
                }
            }
        }

        self.mode = mode;
    }
}
