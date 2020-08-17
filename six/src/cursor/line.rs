use crate::cursor::{Bounded, Cursor, Iter};

pub struct Line<'a> {
    cursor: Cursor,
    column: usize,
    text: &'a str,
}

impl<'a> Iter<'a> for Line<'a> {
    fn new(cursor: Cursor, text: &'a str) -> Self {
        Self { text, cursor, column: cursor.to_col(text) }
    }

    fn at(&self) -> Self::Item {
        self.cursor
    }
}

impl Iterator for Line<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.offset += self.text[self.cursor.offset..].find('\n')? + 1;

        self.cursor = self
            .cursor
            .iter::<Bounded>(self.text)
            .take_while(|cursor| cursor.to_col(self.text) <= self.column)
            .last()
            .unwrap_or(self.cursor);

        Some(self.cursor)
    }
}

impl DoubleEndedIterator for Line<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.cursor.offset = self.text[..self.cursor.offset].rfind('\n')?;

        self.cursor = self
            .cursor
            .iter::<Bounded>(self.text)
            .rev()
            .find(|cursor| cursor.to_col(self.text) == self.column)
            .unwrap_or(self.cursor);

        Some(self.cursor)
    }
}
