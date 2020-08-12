use crate::buffer::Buffer;
use crate::cursor::{Codepoint, Cursor, Iter};

pub struct Bounded<'a> {
    chars: Codepoint<'a>,
    buffer: &'a Buffer,
}

fn has_line_break(cursor: Cursor, buffer: &Buffer) -> bool {
    buffer.get(cursor.index).map_or(false, |ch| ch == '\n')
}

impl<'a> Iter<'a> for Bounded<'a> {
    fn new(cursor: Cursor, buffer: &'a Buffer) -> Self {
        Self { buffer, chars: Codepoint::new(cursor, buffer) }
    }
}

impl Iterator for Bounded<'_> {
    type Item = Cursor;

    /// Moves a `Cursor` forwards inside a line by up to the specified amount.
    fn next(&mut self) -> Option<Self::Item> {
        match self.chars.next()? {
            cursor if has_line_break(cursor, self.buffer) => None,
            cursor => Some(cursor),
        }
    }
}

impl DoubleEndedIterator for Bounded<'_> {
    /// Moves a `Cursor` forwards inside a line by up to the specified amount.
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.chars.next_back()? {
            cursor if has_line_break(cursor, self.buffer) => None,
            cursor => Some(cursor),
        }
    }
}
