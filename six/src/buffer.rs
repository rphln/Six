use std::ops::RangeBounds;

use crate::cursor::{to_offset_bound, Codepoint, Cursor, Iter};

/// The mutable buffer of an editor.
#[derive(Debug, Default)]
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
        let start = to_offset_bound(range.start_bound(), self.content.as_str());
        let end = to_offset_bound(range.end_bound(), self.content.as_str());

        self.content.replace_range((start, end), text)
    }
}
