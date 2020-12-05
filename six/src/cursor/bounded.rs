use crate::{Buffer, Cursor};

/// An iterator over the cursor positions within a line.
pub struct Bounded<'a> {
    anchor: Cursor,
    buffer: &'a Buffer,
}

impl Iterator for Bounded<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.line(self.anchor.row)?.len() < self.anchor.col {
            self.anchor.col += 1;
            Some(self.anchor)
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for Bounded<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.anchor.col >= 1 {
            self.anchor.col -= 1;
            Some(self.anchor)
        } else {
            None
        }
    }
}
