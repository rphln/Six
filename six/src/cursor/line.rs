use crate::cursor::{Cursor, Iter};

pub struct Line;

impl Iterator for Iter<'_, Line> {
    type Item = Cursor;

    /// Moves a `Cursor` downward.
    fn next(&mut self) -> Option<Self::Item> {
        let row = self.anchor.row();

        self.anchor = if row + 1 < self.buffer.rows() {
            Some(Cursor {
                row: row + 1,
                col: self.anchor.offset.min(self.buffer.cols_at(row + 1)),
                offset: self.anchor.offset,
            })
        } else {
            None
        }?;

        Some(self.anchor)
    }
}

impl DoubleEndedIterator for Iter<'_, Line> {
    /// Moves a `Cursor` upward.
    fn next_back(&mut self) -> Option<Self::Item> {
        let row = self.anchor.row();

        self.anchor = if row > 0 {
            Some(Cursor {
                row: row - 1,
                col: self.anchor.offset.min(self.buffer.cols_at(row - 1)),
                offset: self.anchor.offset,
            })
        } else {
            None
        }?;

        Some(self.anchor)
    }
}
