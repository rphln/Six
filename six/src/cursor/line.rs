use crate::buffer::Buffer;
use crate::cursor::{Cursor, Iter};

pub struct Line<'a> {
    cursor: Cursor,
    buffer: &'a Buffer,
}

impl<'a> Iter<'a> for Line<'a> {
    fn new(cursor: Cursor, buffer: &'a Buffer) -> Self {
        Self { cursor, buffer }
    }
}

impl Iterator for Line<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl DoubleEndedIterator for Line<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
