use crate::cursor::{Cursor, Iter};

pub struct Bounded;

impl Iterator for Iter<'_, Bounded> {
    type Item = Cursor;

    /// Moves a `Cursor` forwards inside a line by up to the specified amount.
    fn next(&mut self) -> Option<Self::Item> {
        if self.has_line_break(self.anchor).unwrap_or(false) {
            None
        } else {
            self.anchor = self.anchor.iter::<char>(self.buffer).next()?;
            Some(self.anchor)
        }
    }
}

impl DoubleEndedIterator for Iter<'_, Bounded> {
    /// Moves a `Cursor` forwards inside a line by up to the specified amount.
    fn next_back(&mut self) -> Option<Self::Item> {
        let next = self.anchor.iter::<char>(self.buffer).next_back()?;

        if self.has_line_break(next).expect("has_line_break") {
            None
        } else {
            self.anchor = next;
            Some(self.anchor)
        }
    }
}
