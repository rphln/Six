use std::ops::RangeBounds;

use crate::cursor::{Cells, Paragraphs};
use crate::Cursor;

pub type Content = Vec<Row>;

#[derive(Debug, Default)]
pub struct Row(Vec<char>);

impl Row {
    /// Returns the number of characters in the row.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the character at the specified position.
    pub fn get(&self, at: usize) -> Option<char> {
        self.0.get(at).copied()
    }

    /// Fills the row with the specified character until it meets the given length.
    pub fn pad(&mut self, chars: usize, padding: char) {
        self.0.resize(self.0.len().max(chars), padding)
    }

    pub fn to_string(&self) -> String {
        self.0.iter().collect::<String>()
    }

    /// Inserts a character at the specified position.
    ///
    /// If the position is past the end of the line, this function fills the line with spaces
    /// beforehand.
    pub fn insert(&mut self, position: usize, ch: char) {
        if position > self.len() {
            self.pad(position, ' ');
        }

        self.0.insert(position, ch)
    }
}

/// The mutable buffer of an editor.
#[derive(Debug, Default)]
pub struct Buffer {
    /// The text content.
    content: Content,

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

    /// Sets the cursor position, returning the old value.
    pub fn set_cursor(&mut self, cursor: Cursor) -> Cursor {
        std::mem::replace(&mut self.cursor, cursor)
    }

    /// Returns a reference to the buffer's content.
    pub fn content(&self) -> &Content {
        &self.content
    }

    /// Converts the buffer contents to a string.
    #[inline]
    #[must_use]
    pub fn to_string(&self) -> String {
        unimplemented!()
    }

    /// Returns the character at the specified position.
    pub fn get(&self, at: Cursor) -> Option<char> {
        self.line(at.row()).and_then(|row| row.get(at.col()))
    }

    /// Returns a reference to the specified line, if it exists.
    pub fn line(&self, idx: usize) -> Option<&Row> {
        self.content.get(idx)
    }

    /// Inserts a character at the specified cursor position.
    pub fn insert(&mut self, ch: char, at: Cursor) {
        self.content[at.row()].insert(at.col(), ch);
    }

    /// Replaces the text in a range.
    ///
    /// The length of the range can differ from the replacement's.
    pub fn edit(&mut self, text: &str, range: impl RangeBounds<Cursor>) {
        unimplemented!()
    }

    /// Attempts to move the cursor forward over a given metric.
    ///
    /// Returns the new position on success.
    pub fn forward<'a, It>(&'a mut self) -> Option<Cursor>
    where
        It: Iterator<Item = Cursor>,
    {
        unimplemented!()
    }

    /// Attempts to move the cursor backward over a given metric.
    ///
    /// Returns the new position on success.
    pub fn backward<'a, It>(&'a mut self) -> Option<Cursor>
    where
        It: DoubleEndedIterator<Item = Cursor>,
    {
        unimplemented!()
    }

    /// Returns an iterator over the cells of the buffer, starting at the specified position.
    pub fn cells(&self, cursor: Cursor) -> Cells<'_> {
        Cells::new(cursor, &self)
    }

    /// Returns an iterator over the paragraphs of the buffer, starting at the specified
    /// position.
    pub fn paragraphs(&self, cursor: Cursor) -> Paragraphs<'_> {
        Paragraphs::new(cursor, &self)
    }
}
