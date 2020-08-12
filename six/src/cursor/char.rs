use crate::cursor::{Col, Cursor, Iter, Row};

impl Iterator for Iter<'_, char> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        let col = self.anchor.col();
        let row = self.anchor.row();

        self.anchor = if col == self.buffer.cols_at(row) {
            if row + 1 < self.buffer.rows() {
                Some(Cursor::new(Row(row + 1), Col(0)))
            } else {
                None
            }
        } else {
            Some(Cursor::new(Row(row), Col(col + 1)))
        }?;

        Some(self.anchor)
    }
}

impl DoubleEndedIterator for Iter<'_, char> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let col = self.anchor.col();
        let row = self.anchor.row();

        self.anchor = if col == 0 {
            if row > 0 {
                Some(Cursor::new(Row(row - 1), Col(self.buffer.cols_at(row - 1))))
            } else {
                None
            }
        } else {
            Some(Cursor::new(Row(row), Col(col - 1)))
        }?;

        Some(self.anchor)
    }
}
