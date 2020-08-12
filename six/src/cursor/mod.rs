use unicode_width::UnicodeWidthStr;

use crate::buffer::Buffer;

pub mod bounded;
pub mod codepoint;
pub mod head;
pub mod line;
pub mod paragraph;
pub mod tail;

pub use crate::cursor::{
    bounded::Bounded, codepoint::Codepoint, head::Head, line::Line, paragraph::Paragraph,
    tail::Tail,
};

/// A text buffer coordinate.
#[derive(Debug, Clone, Copy, Default, Derivative)]
#[derivative(PartialEq, PartialOrd, Eq, Ord)]
pub struct Cursor {
    /// The cursor index.
    pub index: usize,

    /// The raw value of the cursor column.
    #[derivative(PartialEq = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub offset: Option<usize>,
}

impl Cursor {
    /// Creates a new cursor at the specified position.
    #[must_use]
    pub fn new(index: usize) -> Self {
        Self { index, offset: None }
    }

    /// Returns the horizontal position of this `Cursor`.
    #[must_use]
    pub fn to_col(self, buffer: &Buffer) -> usize {
        let start = buffer[..self.index].rfind('\n').map_or(0, |idx| idx + 1);
        buffer[start..self.index].width()
    }

    /// Returns the vertical position of this `Cursor`.
    #[must_use]
    pub fn to_row(self, buffer: &Buffer) -> usize {
        buffer[..self.index].split('\n').count() - 1
    }

    /// Returns an iterator over the positions of a given unit.
    #[inline]
    pub fn iter<'a, It: Iter<'a>>(self, buffer: &'a Buffer) -> It {
        It::new(self, buffer)
    }

    /// Returns the previous cursor position over a given unit.
    ///
    /// Shorthand for `cursor.iter::<U>(&buffer).next_back()`.
    #[inline]
    pub fn prev<'a, It: Iter<'a>>(self, buffer: &'a Buffer) -> Option<Self> {
        self.iter::<It>(buffer).next_back()
    }

    /// Returns the next cursor position over a given unit.
    ///
    /// Shorthand for `cursor.iter::<U>(&buffer).next()`.
    #[inline]
    pub fn next<'a, It: Iter<'a>>(self, buffer: &'a Buffer) -> Option<Self> {
        self.iter::<It>(buffer).next()
    }
}

/// An iterator over the positions of an unit.
pub trait Iter<'a>: Iterator<Item = Cursor> + DoubleEndedIterator {
    fn new(cursor: Cursor, buffer: &'a Buffer) -> Self;
}
