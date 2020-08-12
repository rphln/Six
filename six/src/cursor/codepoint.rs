use crate::buffer::Buffer;
use crate::cursor::{Cursor, Iter};

pub struct Codepoint<'s> {
    cursor: Cursor,
    buffer: &'s Buffer,
}

impl<'s> Iter<'s> for Codepoint<'s> {
    fn new(cursor: Cursor, buffer: &'s Buffer) -> Self {
        Self { cursor, buffer }
    }
}

impl Iterator for Codepoint<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.index += self.buffer[self.cursor.index..].char_indices().skip(1).next()?.0;
        Some(self.cursor)
    }
}

impl DoubleEndedIterator for Codepoint<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.cursor.index = self.buffer[..self.cursor.index].char_indices().last()?.0;
        Some(self.cursor)
    }
}
