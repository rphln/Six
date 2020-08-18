use std::ops::Bound;

use unicode_width::UnicodeWidthStr;

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

/// A text text coordinate.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Cursor {
    /// The cursor offset.
    offset: usize,
}

impl Cursor {
    /// Creates a new cursor at the specified position.
    #[inline]
    #[must_use]
    pub fn new(offset: usize) -> Self {
        Self { offset }
    }

    /// Creates a new cursor at the final position of a text.
    #[inline]
    #[must_use]
    pub fn eof(text: &str) -> Self {
        Self { offset: text.len() }
    }

    /// Returns the codepoint offset for this cursor.
    #[inline]
    #[must_use]
    pub fn offset(self) -> usize {
        self.offset
    }

    /// Returns the horizontal position of this `Cursor`.
    #[must_use]
    pub fn to_col(self, text: &str) -> usize {
        let start = text[..self.offset].rfind('\n').map_or(0, |idx| idx + 1);
        text[start..self.offset].width()
    }

    /// Returns the vertical position of this `Cursor`.
    #[must_use]
    pub fn to_row(self, text: &str) -> usize {
        text[..self.offset].split('\n').count() - 1
    }

    /// Returns an iterator over the positions of a given unit.
    #[inline]
    #[must_use]
    pub fn iter<'a, It: Iter<'a>>(self, text: &'a str) -> It {
        It::new(self, text)
    }

    /// Returns the previous cursor position over a given unit.
    #[inline]
    #[must_use]
    pub fn backward<'a, It: Iter<'a>>(self, text: &'a str) -> Option<Self> {
        self.iter::<It>(text).next_back()
    }

    /// Returns the next cursor position over a given unit.
    #[inline]
    #[must_use]
    pub fn forward<'a, It: Iter<'a>>(self, text: &'a str) -> Option<Self> {
        self.iter::<It>(text).next()
    }
}

/// An iterator over the positions of an unit.
pub trait Iter<'a>: Iterator<Item = Cursor> + DoubleEndedIterator {
    /// Creates a new iterator.
    fn new(cursor: Cursor, text: &'a str) -> Self;

    /// Returns the current position of this iterator.
    fn at(&self) -> Self::Item;
}

/// Converts a `Cursor` bound to an offset bound.
#[must_use]
pub fn to_offset_bound(bound: Bound<&Cursor>, buffer: &str) -> Bound<usize> {
    use Bound::{Excluded, Included, Unbounded};

    match bound {
        Unbounded => Unbounded,
        Excluded(cursor) => Excluded(cursor.offset()),
        Included(cursor) => {
            if cursor.offset() == buffer.len() {
                Included(cursor.offset())
            } else {
                Included(cursor.forward::<Codepoint>(buffer).expect("next").offset() - 1)
            }
        },
    }
}
