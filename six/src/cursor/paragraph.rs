use crate::cursor::{Codepoint, Cursor, Iter};

pub struct Paragraph<'a> {
    iter: Codepoint<'a>,
    text: &'a str,
}

impl<'a> Iter<'a> for Paragraph<'a> {
    fn new(cursor: Cursor, text: &'a str) -> Self {
        Self { text, iter: Codepoint::new(cursor, text) }
    }

    fn at(&self) -> Self::Item {
        self.iter.at()
    }
}

impl Iterator for Paragraph<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        let text = self.text;

        self.iter.find(|&cursor| {
            let mut chars = text[cursor.offset..].chars();

            let p = chars.next();
            let q = chars.next();
            let r = chars.next();

            !p.map_or(true, |ch| ch == '\n')
                && q.map_or(true, |ch| ch == '\n')
                && r.map_or(true, |ch| ch == '\n')
        })
    }
}

impl DoubleEndedIterator for Paragraph<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let text = self.text;

        self.iter.rfind(|&cursor| {
            if let Some(slice) = text.get(..=cursor.offset) {
                let mut chars = slice.chars();

                let p = chars.next_back();
                let q = chars.next_back();
                let r = chars.next_back();

                !p.map_or(true, |ch| ch == '\n')
                    && q.map_or(true, |ch| ch == '\n')
                    && r.map_or(true, |ch| ch == '\n')
            } else {
                false
            }
        })
    }
}
