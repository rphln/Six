use crate::buffer::Buffer;
use crate::cursor::{Codepoint, Cursor, Iter};

pub struct Head<'a> {
    iter: Codepoint<'a>,
    buffer: &'a Buffer,
}

fn is_word_head(cursor: Cursor, buffer: &Buffer) -> bool {
    cursor
        .prev::<Codepoint>(buffer)
        .and_then(|cursor| buffer.get(cursor.index))
        .map_or(true, char::is_whitespace)
        && buffer.get(cursor.index).map_or(false, |ch| !ch.is_whitespace())
}

impl<'a> Iter<'a> for Head<'a> {
    fn new(cursor: Cursor, buffer: &'a Buffer) -> Self {
        Self { buffer, iter: Codepoint::new(cursor, buffer) }
    }

    fn at(&self) -> Self::Item {
        self.iter.at()
    }
}

impl Iterator for Head<'_> {
    type Item = Cursor;

    /// Moves forward by a word unit.
    fn next(&mut self) -> Option<Self::Item> {
        let buffer = self.buffer;
        self.iter.find(|&cursor| is_word_head(cursor, buffer))
    }
}

impl DoubleEndedIterator for Head<'_> {
    /// Moves backward by a word unit.
    fn next_back(&mut self) -> Option<Self::Item> {
        let buffer = self.buffer;
        self.iter.rfind(|&cursor| is_word_head(cursor, buffer))
    }
}
