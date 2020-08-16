use crate::buffer::Buffer;
use crate::cursor::{Cursor, Iter};

/// An iterator over the Unicode codepoint boundaries of a buffer.
pub struct Codepoint<'s> {
    cursor: Cursor,
    buffer: &'s str,
}

impl<'s> Iter<'s> for Codepoint<'s> {
    fn new(cursor: Cursor, buffer: &'s Buffer) -> Self {
        Self { cursor, buffer: buffer.as_str() }
    }

    fn at(&self) -> Self::Item {
        self.cursor
    }
}

impl Iterator for Codepoint<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.index == self.buffer.len() {
            None
        } else {
            self.cursor.index += match self.buffer.as_bytes()[self.cursor.index] {
                b if b < 0x80 => 1,
                b if b < 0xe0 => 2,
                b if b < 0xf0 => 3,
                _ => 4,
            };

            Some(self.cursor)
        }
    }
}

impl DoubleEndedIterator for Codepoint<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cursor.index == 0 {
            None
        } else {
            self.cursor.index -= 1;

            while !self.buffer.is_char_boundary(self.cursor.index) {
                self.cursor.index -= 1;
            }

            Some(self.cursor)
        }
    }
}
