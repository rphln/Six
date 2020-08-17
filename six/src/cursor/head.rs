use crate::cursor::{Codepoint, Cursor, Iter};

pub struct Head<'a> {
    iter: Codepoint<'a>,
    text: &'a str,
}

fn is_word_head(cursor: Cursor, text: &str) -> bool {
    if let Some(slice) = text.get(..=cursor.offset) {
        let mut chars = slice.chars();

        let p = chars.next_back();
        let q = chars.next_back();

        !p.map_or(true, char::is_whitespace) && q.map_or(true, char::is_whitespace)
    } else {
        false
    }
}

impl<'a> Iter<'a> for Head<'a> {
    fn new(cursor: Cursor, text: &'a str) -> Self {
        Self { text, iter: Codepoint::new(cursor, text) }
    }

    fn at(&self) -> Self::Item {
        self.iter.at()
    }
}

impl Iterator for Head<'_> {
    type Item = Cursor;

    /// Moves forward by a word unit.
    fn next(&mut self) -> Option<Self::Item> {
        let text = self.text;
        self.iter.find(|&cursor| is_word_head(cursor, text))
    }
}

impl DoubleEndedIterator for Head<'_> {
    /// Moves backward by a word unit.
    fn next_back(&mut self) -> Option<Self::Item> {
        let text = self.text;
        self.iter.rfind(|&cursor| is_word_head(cursor, text))
    }
}
