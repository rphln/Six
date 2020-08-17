use std::ops::{Bound, RangeBounds};

use rlua::Lua;

use crate::cursor::{Codepoint, Cursor, Iter};
use crate::event::Key;
use crate::mode::{Advance, Keymap, Mode, Operation};

/// An modal editor.
#[derive(Debug, Default)]
pub struct Editor {
    /// The current mode.
    mode: Mode,

    /// The mutable state of the editor.
    state: State,

    /// The editor context.
    context: Context,
}

#[derive(Default, Derivative)]
#[derivative(Debug)]
pub struct Context {
    /// Lua state.
    #[derivative(Debug = "ignore")]
    lua: Lua,
}

/// The mutable state of an editor.
#[derive(Debug, Default)]
pub struct State {
    /// The text buffer.
    buffer: String,

    /// The cursor position.
    cursor: Cursor,
}

impl State {
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

    /// Returns a reference to the buffer contents.
    #[inline]
    #[must_use]
    pub fn buffer(&self) -> &str {
        self.buffer.as_str()
    }

    /// Inserts a character at the specified cursor position.
    pub fn insert(&mut self, ch: char, at: Cursor) {
        self.buffer.insert(at.offset(), ch);
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
        let cursor = self.cursor.forward::<It>(self.buffer.as_str())?;
        std::mem::replace(&mut self.cursor, cursor).into()
    }

    /// Attempts to move the cursor backward over a given metric.
    ///
    /// Returns the previous position on success.
    pub fn backward<'a, It: Iter<'a>>(&'a mut self) -> Option<Cursor> {
        let cursor = self.cursor.backward::<It>(self.buffer.as_str())?;
        std::mem::replace(&mut self.cursor, cursor).into()
    }

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

        self.buffer.replace_range((start, end), text)
    }
}

impl Editor {
    /// Returns a reference to the state mode.
    #[must_use]
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Returns the cursor position.
    #[inline]
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.state.cursor()
    }

    /// Returns a reference to the buffer contents.
    #[inline]
    #[must_use]
    pub fn buffer(&self) -> &str {
        self.state.buffer()
    }

    /// Sets the cursor position.
    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.state.cursor = cursor
    }

    // TODO: Replace `Key` with `Operation`.
    pub fn handle_key(&mut self, key: Key) {
        // TODO: Replace with a dynamic keymap lookup.
        #[rustfmt::skip]
        let operations = match (self.mode().keymap(), key) {
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

            (keymap, Key::Esc) if keymap.intersects(!Keymap::INSERT)                         => vec![Operation::Escape],

            (keymap, Key::Backspace) if keymap.intersects(Keymap::INSERT)                    => vec![Operation::Delete, Operation::Left],
            (keymap, Key::Delete) if keymap.intersects(Keymap::INSERT)                       => vec![Operation::Delete],

            (keymap, Key::Esc) if keymap.intersects(Keymap::INSERT)                          => vec![Operation::Escape, Operation::Left],

            (keymap, Key::Delete) if keymap.intersects(Keymap::all())                        => vec![Operation::Delete],

            (keymap, Key::Down) if keymap.intersects(Keymap::all())                          => vec![Operation::Down],
            (keymap, Key::Left) if keymap.intersects(Keymap::all())                          => vec![Operation::Left],
            (keymap, Key::Right) if keymap.intersects(Keymap::all())                         => vec![Operation::Right],
            (keymap, Key::Up) if keymap.intersects(Keymap::all())                            => vec![Operation::Up],

            (keymap, Key::Char(ch)) if keymap.intersects(Keymap::INSERT | Keymap::QUERY)     => vec![Operation::Input(ch)],

            (_, _)                                                                           => vec![],
        };

        self.advance(operations.as_slice())
    }

    /// Advances the state state by handling events.
    pub fn advance(&mut self, operations: &[Operation]) {
        let mut mode = std::mem::take(&mut self.mode);

        for &operation in operations {
            match mode.advance(&mut self.state, operation) {
                Advance::Continue(next) => mode = next,
                Advance::Stop(next) => {
                    mode = next;
                    break;
                },
            }
        }

        self.mode = mode;
    }
}
