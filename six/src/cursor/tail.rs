use crate::cursor::{Cursor, Iter};

pub struct Tail;

impl Iter<'_, Tail> {
    fn is_word_tail(&self, cursor: Cursor) -> bool {
        let idx = cursor.to_index(self.buffer);

        self.buffer.get(idx).map_or(false, |ch| !ch.is_whitespace())
            && self.buffer.get(idx + 1).map_or(true, |ch| ch.is_whitespace())
    }
}

impl Iterator for Iter<'_, Tail> {
    type Item = Cursor;

    /// Moves forward to the end of word.
    fn next(&mut self) -> Option<Self::Item> {
        self.anchor =
            self.anchor.iter::<char>(self.buffer).find(|&cursor| self.is_word_tail(cursor))?;

        Some(self.anchor)
    }
}

impl DoubleEndedIterator for Iter<'_, Tail> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.anchor = self
            .anchor
            .iter::<char>(self.buffer)
            .rev()
            .find(|&cursor| self.is_word_tail(cursor))?;

        Some(self.anchor)
    }
}

#[cfg(test)]
mod tests {
    use super::Tail;
    use crate::{
        cursor::{Col, Row},
        Buffer, Cursor,
    };

    static LOREM: &str = include_str!("../../assets/lorem.txt");

    #[test]
    fn test_next() {
        let buffer = Buffer::from(LOREM);
        let mut iter = Cursor::new(Row(0), Col(0)).iter::<Tail>(&buffer);

        assert_eq!(iter.next().expect("next"), Cursor::new(Row(0), Col(2)));
        assert_eq!(iter.next().expect("next"), Cursor::new(Row(0), Col(6)));
        assert_eq!(iter.next().expect("next"), Cursor::new(Row(0), Col(10)));
        assert_eq!(iter.next().expect("next"), Cursor::new(Row(1), Col(2)));
        assert_eq!(iter.next().expect("next"), Cursor::new(Row(1), Col(7)));
        assert_eq!(iter.next().expect("next"), Cursor::new(Row(1), Col(12)));

        assert_eq!(iter.next().expect("next"), Cursor::new(Row(3), Col(4)));

        let mut iter = Cursor::new(Row(8), Col(0)).iter::<Tail>(&buffer);

        assert_eq!(iter.next().expect("next"), Cursor::new(Row(9), Col(6)));
        assert_eq!(iter.next().expect("next"), Cursor::new(Row(9), Col(12)));
        assert_eq!(iter.next().expect("next"), Cursor::new(Row(10), Col(5)));

        let mut iter = Cursor::new(Row(15), Col(38)).iter::<Tail>(&buffer);

        assert_eq!(iter.next().expect("next"), Cursor::new(Row(15), Col(47)));
        assert_eq!(iter.next(), None);
    }
}
