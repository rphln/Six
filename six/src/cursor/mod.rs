use std::marker::PhantomData;

use crate::buffer::Buffer;

pub mod bounded;
pub mod char;
pub mod head;
pub mod line;
pub mod paragraph;
pub mod tail;

pub use crate::cursor::{
    bounded::Bounded, head::Head, line::Line, paragraph::Paragraph, tail::Tail,
};

/// Newtype to avoid mixing arguments by accident.
pub struct Col(usize);

/// Newtype to avoid mixing arguments by accident.
pub struct Row(usize);

/// A text buffer coordinate.
#[derive(Debug, Clone, Copy, Default, Derivative)]
#[derivative(PartialEq, PartialOrd, Eq, Ord)]
pub struct Cursor {
    /// The cursor row.
    row: usize,

    /// The cursor column, limited to the current line length.
    col: usize,

    /// The raw value of the cursor column.
    #[derivative(PartialEq = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    offset: usize,
}

impl Cursor {
    /// Creates a new cursor at the specified coordinates.
    #[must_use]
    pub fn new(row: Row, col: Col) -> Self {
        Self { row: row.0, col: col.0, offset: col.0 }
    }

    /// Returns the horizontal position of this `Cursor`.
    #[must_use]
    pub fn col(self) -> usize {
        self.col
    }

    /// Returns the vertical position of this `Cursor`.
    #[must_use]
    pub fn row(self) -> usize {
        self.row
    }

    /// Converts this point to a buffer character index.
    #[must_use]
    pub fn to_index(self, buffer: &Buffer) -> usize {
        // TODO: Use raw indices in `Cursor`. Doing this conversion is too expensive given how
        // frequently we need it.

        buffer.lines().take(self.row).fold(self.col, |idx, line| idx + line.chars().count() + 1)
    }

    /// Returns an iterator over the positions of a given unit.
    pub fn iter<'a, Unit>(self, buffer: &'a Buffer) -> Iter<'a, Unit> {
        Iter::new(self, buffer)
    }
}

/// An iterator over the positions of an unit.
#[derive(Clone)]
pub struct Iter<'a, Unit> {
    anchor: Cursor,
    buffer: &'a Buffer,

    unit: PhantomData<*const Unit>,
}

impl<'a, Unit> Iter<'a, Unit> {
    pub fn new(anchor: Cursor, buffer: &'a Buffer) -> Self {
        Self { anchor, buffer, unit: PhantomData::default() }
    }

    fn has_whitespace(&self, cursor: Cursor) -> Option<bool> {
        self.buffer.get(cursor.to_index(self.buffer)).map(char::is_whitespace)
    }

    fn has_character(&self, cursor: Cursor) -> Option<bool> {
        self.has_whitespace(cursor).map(|r| !r)
    }

    fn has_line_break(&self, cursor: Cursor) -> Option<bool> {
        self.buffer.get(cursor.to_index(&self.buffer)).map(|ch| ch == '\n')
    }
}
