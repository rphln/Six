use crate::cursor::{Cursor, Iter};

/// An iterator over the Unicode codepoint boundaries of a text.
pub struct Codepoint<'s> {
    cursor: Cursor,
    text: &'s str,
}

impl<'s> Iter<'s> for Codepoint<'s> {
    fn new(cursor: Cursor, text: &'s str) -> Self {
        Self { cursor, text }
    }

    fn at(&self) -> Self::Item {
        self.cursor
    }
}

impl Iterator for Codepoint<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.offset == self.text.len() {
            None
        } else {
            self.cursor.offset += match self.text.as_bytes()[self.cursor.offset] {
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
        if self.cursor.offset == 0 {
            None
        } else {
            self.cursor.offset -= 1;

            while !self.text.is_char_boundary(self.cursor.offset) {
                self.cursor.offset -= 1;
            }

            Some(self.cursor)
        }
    }
}
