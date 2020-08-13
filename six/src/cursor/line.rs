use crate::buffer::Buffer;
use crate::cursor::{Bounded, Cursor, Iter};

pub struct Line<'a> {
    cursor: Cursor,
    buffer: &'a Buffer,

    column: usize,
}

impl<'a> Iter<'a> for Line<'a> {
    fn new(cursor: Cursor, buffer: &'a Buffer) -> Self {
        Self { cursor, buffer, column: cursor.to_col(buffer) }
    }

    fn at(&self) -> Self::Item {
        self.cursor
    }
}

impl Iterator for Line<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.index += self.buffer[self.cursor.index..].find('\n')? + 1;
        self.cursor
            .iter::<Bounded>(self.buffer)
            .take_while(|cursor| cursor.to_col(self.buffer) <= self.column)
            .last()
    }
}

impl DoubleEndedIterator for Line<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.cursor.index = self.buffer[..self.cursor.index].rfind('\n')?;
        self.cursor
            .iter::<Bounded>(self.buffer)
            .rev()
            .take_while(|cursor| cursor.to_col(self.buffer) >= self.column)
            .last()
    }
}
