use crate::cursor::{Cursor, Iter};

pub struct Head;

impl Iter<'_, Head> {
    fn is_word_head(&self, cursor: Cursor) -> bool {
        let idx = cursor.to_index(self.buffer);

        idx.checked_sub(1)
            .and_then(|idx| self.buffer.get(idx))
            .map_or(true, |ch| ch.is_whitespace())
            && self.buffer.get(idx).map_or(false, |ch| !ch.is_whitespace())
    }
}

impl Iterator for Iter<'_, Head> {
    type Item = Cursor;

    /// Moves forward by a word unit.
    fn next(&mut self) -> Option<Self::Item> {
        self.anchor =
            self.anchor.iter::<char>(self.buffer).find(|&cursor| self.is_word_head(cursor))?;

        Some(self.anchor)
    }
}

impl DoubleEndedIterator for Iter<'_, Head> {
    /// Moves backward by a word unit.
    fn next_back(&mut self) -> Option<Self::Item> {
        self.anchor = self
            .anchor
            .iter::<char>(self.buffer)
            .rev()
            .find(|&cursor| self.is_word_head(cursor))?;

        Some(self.anchor)
    }
}
