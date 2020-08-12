

use crate::cursor::{Cursor, Iter};

pub struct Paragraph;

impl Iterator for Iter<'_, Paragraph> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        self.anchor = self.anchor.iter::<char>(self.buffer).find(|&cursor| {
            let idx = cursor.to_index(self.buffer);

            let p = idx.checked_sub(2).and_then(|idx| self.buffer.get(idx));
            let q = idx.checked_sub(1).and_then(|idx| self.buffer.get(idx));
            let r = self.buffer.get(idx);

            p.map_or(false, |ch| ch != '\n')
                && q.map_or(false, |ch| ch == '\n')
                && r.map_or(false, |ch| ch == '\n')
        })?;

        Some(self.anchor)
    }
}

impl DoubleEndedIterator for Iter<'_, Paragraph> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.anchor = self.anchor.iter::<char>(self.buffer).rev().find(|&cursor| {
            let idx = cursor.to_index(self.buffer);

            let p = idx.checked_sub(1).and_then(|idx| self.buffer.get(idx));
            let q = self.buffer.get(idx);
            let r = self.buffer.get(idx + 1);

            p.map_or(true, |ch| ch == '\n')
                && q.map_or(false, |ch| ch == '\n')
                && r.map_or(false, |ch| ch != '\n')
        })?;

        Some(self.anchor)
    }
}
