use std::ops::{Bound, RangeBounds};

use crate::cursor::Cursor;

#[derive(Debug, Default)]
pub struct Buf(String);

impl Buf {
    /// Creates a `Buf` from a string slice.
    pub fn from_str(text: &str) -> Self {
        Self(text.into())
    }

    /// Convers this `Buf` to a string.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// An iterator over the lines of a string, as string slices.
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.0.split('\n')
    }

    /// Gets the number of lines in the buffer.
    pub fn rows(&self) -> usize {
        self.lines().count()
    }

    /// Gets the number of characters in the specified line, excluding the line break.
    pub fn cols(&self, line: usize) -> usize {
        self.lines().nth(line).expect("Attempt to index past end of `Buf`").len()
    }

    /// Returns the `char` at `point`.
    pub fn get(&self, point: Cursor) -> Option<char> {
        self.0.chars().nth(to_offset(&self.0, point))
    }

    /// Replaces the specified range in the buffer with the given string.
    pub fn edit(&mut self, range: impl RangeBounds<Cursor>, text: &str) {
        let start = to_offset_bound(&self.0, range.start_bound());
        let end = to_offset_bound(&self.0, range.end_bound());

        self.0.replace_range((start, end), text.as_ref());
    }

    /// Inserts a character into this `Buf` at the specified position.
    pub fn insert(&mut self, point: Cursor, ch: char) {
        self.0.insert(to_offset(&self.0, point), ch);
    }

    /// Deletes the text in the specified range.
    pub fn delete(&mut self, range: impl RangeBounds<Cursor>) {
        self.edit(range, "")
    }
}

fn to_offset(buffer: &String, point: Cursor) -> usize {
    let offset = point.col();

    buffer
        .split('\n')
        .take(point.row())
        .fold(offset, |offset, line| offset + 1 + line.chars().count())
}

fn to_offset_bound(buffer: &String, bound: Bound<&Cursor>) -> Bound<usize> {
    use Bound::*;

    match bound {
        Unbounded => Unbounded,
        Included(&p) => Included(to_offset(buffer, p)),
        Excluded(&p) => Excluded(to_offset(buffer, p)),
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::buffer::*;
//     use crate::point::Point;

//     #[test]
//     fn test_to_offset() {
//         let buffer = String::from("lorem ipsum\ndolor sit amet");

//         assert_eq!(to_offset(&buffer, Point { y: 0, x: 0 }), 0);
//         assert_eq!(to_offset(&buffer, Point { y: 0, x: 5 }), 5);

//         assert_eq!(to_offset(&buffer, Point { y: 1, x: 0 }), 12);
//         assert_eq!(to_offset(&buffer, Point { y: 1, x: 5 }), 17);
//     }

//     #[test]
//     fn test_insert_at_start() {
//         let mut buffer = String::from("foo");
//         Buf::insert(&mut buffer, Point { x: 0, y: 0 }, 'f');

//         assert_eq!(buffer, "ffoo");
//     }

//     #[test]
//     fn test_insert_after_break() {
//         let mut buffer = String::from("foo\nbar");
//         let point = Point { x: 0, y: 1 };

//         Buf::insert(&mut buffer, point, 'b');

//         assert_eq!(buffer, "foo\nbbar");
//     }

//     #[test]
//     fn test_insert_before_break() {
//         let mut buffer = String::from("foo\nbar");
//         let point = Point { x: 0, y: 0 }.eol(&buffer).unwrap();

//         Buf::insert(&mut buffer, point, 'o');

//         assert_eq!(buffer, "fooo\nbar");
//     }

//     #[test]
//     fn test_insert_at_end() {
//         let mut buffer = String::from("foo\nbar");
//         let point = Point { x: 0, y: 1 }.eol(&buffer).unwrap();

//         Buf::insert(&mut buffer, point, 'r');

//         assert_eq!(buffer, "foo\nbarr");
//     }
// }
