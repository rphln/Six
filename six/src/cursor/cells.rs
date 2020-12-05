use crate::{Buffer, Cursor};

/// An iterator over the Unicode codepoint boundaries of a buffer.
pub struct Cells<'a> {
    cursor: Cursor,
    buffer: &'a Buffer,
}

impl<'a> Cells<'a> {
    pub fn new(cursor: Cursor, buffer: &'a Buffer) -> Self {
        Self { cursor, buffer }
    }
}

impl Iterator for Cells<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.col < self.buffer.line(self.cursor.row)?.len() {
            self.cursor.col += 1;
        } else {
            self.cursor.col = 0;
            self.cursor.row += 1;
        }

        Some(self.cursor)
    }
}

impl DoubleEndedIterator for Cells<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cursor.col == 0 {
            self.cursor.row = self.cursor.row.checked_sub(1)?;
            self.cursor.col = self.buffer.line(self.cursor.row)?.len();
        } else {
            self.cursor.col -= 1;
        }

        Some(self.cursor)
    }
}
