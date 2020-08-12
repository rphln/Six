use crate::buffer::Buffer;
use crate::cursor::{Codepoint, Cursor, Iter};

pub struct Tail<'a> {
    chars: Codepoint<'a>,
    buffer: &'a Buffer,
}

fn is_word_tail(cursor: Cursor, buffer: &Buffer) -> bool {
    buffer.get(cursor.index).map_or(false, |ch| !ch.is_whitespace())
        && cursor
            .next::<Codepoint>(buffer)
            .and_then(|cursor| buffer.get(cursor.index))
            .map_or(true, |ch| ch.is_whitespace())
}

impl<'a> Iter<'a> for Tail<'a> {
    fn new(cursor: Cursor, buffer: &'a Buffer) -> Self {
        Self { buffer, chars: Codepoint::new(cursor, buffer) }
    }
}

impl Iterator for Tail<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        let buffer = self.buffer;
        self.chars.find(|&cursor| is_word_tail(cursor, buffer))
    }
}

impl DoubleEndedIterator for Tail<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let buffer = self.buffer;
        self.chars.rfind(|&cursor| is_word_tail(cursor, buffer))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::Tail;
//     use crate::{
//         cursor::{Col, Row},
//         Buffer, Cursor,
//     };

//     static LOREM: &Buffer = include_str!("../../assets/lorem.txt");

//     #[test]
//     fn test_next() {
//         let buffer = Buffer::from(LOREM);
//         let mut iter = Cursor::new(Row(0), Col(0)).iter::<Tail>(&buffer);

//         assert_eq!(iter.next().expect("next"), Cursor::new(Row(0), Col(2)));
//         assert_eq!(iter.next().expect("next"), Cursor::new(Row(0), Col(6)));
//         assert_eq!(iter.next().expect("next"), Cursor::new(Row(0), Col(10)));
//         assert_eq!(iter.next().expect("next"), Cursor::new(Row(1), Col(2)));
//         assert_eq!(iter.next().expect("next"), Cursor::new(Row(1), Col(7)));
//         assert_eq!(iter.next().expect("next"), Cursor::new(Row(1), Col(12)));

//         assert_eq!(iter.next().expect("next"), Cursor::new(Row(3), Col(4)));

//         let mut iter = Cursor::new(Row(8), Col(0)).iter::<Tail>(&buffer);

//         assert_eq!(iter.next().expect("next"), Cursor::new(Row(9), Col(6)));
//         assert_eq!(iter.next().expect("next"), Cursor::new(Row(9), Col(12)));
//         assert_eq!(iter.next().expect("next"), Cursor::new(Row(10), Col(5)));

//         let mut iter = Cursor::new(Row(15), Col(38)).iter::<Tail>(&buffer);

//         assert_eq!(iter.next().expect("next"), Cursor::new(Row(15), Col(47)));
//         assert_eq!(iter.next(), None);
//     }
// }
