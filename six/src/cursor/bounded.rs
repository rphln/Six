use crate::buffer::Buffer;
use crate::cursor::{Codepoint, Cursor, Iter};

pub struct Bounded<'a> {
    chars: Codepoint<'a>,
    buffer: &'a Buffer,
}

fn is_boundary(cursor: Cursor, buffer: &Buffer) -> bool {
    buffer.get(cursor.index).map_or(true, |ch| ch == '\n')
}

impl<'a> Iter<'a> for Bounded<'a> {
    fn new(cursor: Cursor, buffer: &'a Buffer) -> Self {
        Self { buffer, chars: Codepoint::new(cursor, buffer) }
    }

    fn at(&self) -> Self::Item {
        self.chars.at()
    }
}

impl Iterator for Bounded<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        if is_boundary(self.at(), self.buffer) {
            None
        } else {
            self.chars.next().or_else(|| Some(Cursor::eof(self.buffer)))
        }
    }
}

impl DoubleEndedIterator for Bounded<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.chars.next_back()? {
            cursor if is_boundary(cursor, self.buffer) => None,
            cursor => Some(cursor),
        }
    }
}
