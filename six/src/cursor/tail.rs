use crate::buffer::Buffer;
use crate::cursor::{Codepoint, Cursor, Iter};

pub struct Tail<'a> {
    iter: Codepoint<'a>,
    buffer: &'a Buffer,
}

fn is_word_tail(cursor: Cursor, buffer: &Buffer) -> bool {
    buffer.get(cursor.index).map_or(false, |ch| !ch.is_whitespace())
        && cursor
            .iter::<Codepoint>(buffer)
            .next()
            .and_then(|cursor| buffer.get(cursor.index))
            .map_or(true, char::is_whitespace)
}

impl<'a> Iter<'a> for Tail<'a> {
    fn new(cursor: Cursor, buffer: &'a Buffer) -> Self {
        Self { buffer, iter: Codepoint::new(cursor, buffer) }
    }

    fn at(&self) -> Self::Item {
        self.iter.at()
    }
}

impl Iterator for Tail<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        let buffer = self.buffer;
        self.iter.find(|&cursor| is_word_tail(cursor, buffer))
    }
}

impl DoubleEndedIterator for Tail<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let buffer = self.buffer;
        self.iter.rfind(|&cursor| is_word_tail(cursor, buffer))
    }
}

#[cfg(test)]
mod tests {
    use super::Tail;
    use crate::{Buffer, Cursor};

    static LOREM: &str = include_str!("../../assets/lorem.txt");

    #[test]
    fn test_iter() {
        let buffer = Buffer::from(LOREM);
        let codepoints = Cursor::new(0).iter::<Tail>(&buffer).collect::<Vec<_>>();

        assert_eq!(codepoints, vec![]);
    }
}
