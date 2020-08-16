use crate::buffer::Buffer;
use crate::cursor::{Codepoint, Cursor, Iter};

/// An iterator over the cursor positions within a line.
pub struct Bounded<'a> {
    codepoints: Codepoint<'a>,
    buffer: &'a Buffer,
}

impl Bounded<'_> {
    fn is_eol(&self, cursor: Cursor) -> bool {
        self.buffer.get(cursor.index).map_or(true, |ch| ch == '\n')
    }
}

impl<'a> Iter<'a> for Bounded<'a> {
    fn new(cursor: Cursor, buffer: &'a Buffer) -> Self {
        Self { buffer, codepoints: Codepoint::new(cursor, buffer) }
    }

    fn at(&self) -> Self::Item {
        self.codepoints.at()
    }
}

impl Iterator for Bounded<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_eol(self.at()) {
            None
        } else {
            self.codepoints.next()
        }
    }
}

impl DoubleEndedIterator for Bounded<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.codepoints.next_back()? {
            cursor if self.is_eol(cursor) => None,
            cursor => Some(cursor),
        }
    }
}
