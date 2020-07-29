use std::ops::{Bound, RangeBounds};

use crate::cursor::Cursor;

// TODO: Check if this is alright.
pub type Lines<'a, 'b> = Box<dyn Iterator<Item = &'a str> + 'b>;

pub trait Buffer: Clone {
    /// Creates a `Buffer` from a string slice.
    fn from_str(text: &str) -> Self;

    /// Convers this `Buffer` to a string.
    fn to_string(self) -> String;

    /// An iterator over the lines of a string, as string slices.
    fn lines(&self) -> Box<dyn Iterator<Item = &str> + '_>;

    /// Gets the number of lines in the buffer.
    fn rows(&self) -> usize;

    /// Gets the number of characters in the specified line, excluding the line break.
    fn cols(&self, line: usize) -> usize;

    /// Returns the `char` at `point`.
    fn get(&self, point: Cursor) -> Option<char>;

    /// Replaces the specified range in the buffer with the given string.
    fn edit(&mut self, range: impl RangeBounds<Cursor>, text: &str);

    /// Inserts a character into this `Buffer` at the specified position.
    fn insert(&mut self, point: Cursor, ch: char) {
        self.edit(point..point, ch.to_string().as_str())
    }

    /// Deletes the text in the specified range.
    fn delete(&mut self, range: impl RangeBounds<Cursor>) {
        self.edit(range, "")
    }
}

impl Buffer for String {
    fn from_str(text: &str) -> Self {
        text.into()
    }

    fn to_string(self) -> String {
        self
    }

    fn lines(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(self.split('\n'))
    }

    fn rows(&self) -> usize {
        self.lines().count()
    }

    fn cols(&self, line: usize) -> usize {
        self.lines().nth(line).expect("Attempt to index past end of `Buffer`").len()
    }

    fn get(&self, point: Cursor) -> Option<char> {
        self.chars().nth(to_offset(self, point))
    }

    fn insert(&mut self, point: Cursor, ch: char) {
        self.insert(to_offset(self, point), ch);
    }

    fn edit(&mut self, range: impl RangeBounds<Cursor>, text: &str) {
        let start = to_offset_bound(self, range.start_bound());
        let end = to_offset_bound(self, range.end_bound());

        self.replace_range((start, end), text.as_ref());
    }
}

fn to_offset(buffer: &str, point: Cursor) -> usize {
    let offset = point.col(&buffer.to_owned());

    buffer
        .split('\n')
        .take(point.row(&buffer.to_owned()))
        .fold(offset, |offset, line| offset + 1 + line.chars().count())
}

fn to_offset_bound(buffer: &str, bound: Bound<&Cursor>) -> Bound<usize> {
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
//         Buffer::insert(&mut buffer, Point { x: 0, y: 0 }, 'f');

//         assert_eq!(buffer, "ffoo");
//     }

//     #[test]
//     fn test_insert_after_break() {
//         let mut buffer = String::from("foo\nbar");
//         let point = Point { x: 0, y: 1 };

//         Buffer::insert(&mut buffer, point, 'b');

//         assert_eq!(buffer, "foo\nbbar");
//     }

//     #[test]
//     fn test_insert_before_break() {
//         let mut buffer = String::from("foo\nbar");
//         let point = Point { x: 0, y: 0 }.eol(&buffer).unwrap();

//         Buffer::insert(&mut buffer, point, 'o');

//         assert_eq!(buffer, "fooo\nbar");
//     }

//     #[test]
//     fn test_insert_at_end() {
//         let mut buffer = String::from("foo\nbar");
//         let point = Point { x: 0, y: 1 }.eol(&buffer).unwrap();

//         Buffer::insert(&mut buffer, point, 'r');

//         assert_eq!(buffer, "foo\nbarr");
//     }
// }
