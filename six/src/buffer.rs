use std::borrow::Borrow;
use std::ops::{Bound, RangeBounds};

use crate::cursor::Cursor;

#[derive(Debug, Clone, Default)]
pub struct Buffer(String);

pub trait View {
    /// Returns the number of characters in the buffer.
    fn len(&self) -> usize;

    /// Returns whether the buffer is empty.
    fn is_empty(&self) -> bool;

    /// Returns the number of lines in the buffer.
    fn rows(&self) -> usize;

    /// Returns the number of characters in the specified line, excluding the line break.
    fn cols_at(&self, line: usize) -> usize;

    /// Returns the point at which this view was created.
    fn origin(&self) -> Cursor;

    /// Returns the `char` at `point`.
    fn get(&self, point: Cursor) -> Option<char>;

    /// Convers this `Buffer` to a string.
    fn as_str(&self) -> &str;
}

impl Buffer {
    /// An iterator over the lines of a string, as string slices.
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.0.split('\n')
    }

    /// Replaces the specified range in the buffer with the given string.
    pub fn edit(&mut self, range: impl RangeBounds<Cursor>, text: &str) {
        let start = to_offset_bound(self.0.as_ref(), range.start_bound());
        let end = to_offset_bound(self.0.as_ref(), range.end_bound());

        self.0.replace_range((start, end), text.as_ref());
    }

    /// Inserts a character into this `Buffer` at the specified position.
    pub fn insert(&mut self, point: Cursor, ch: char) {
        self.0.insert(to_offset(self.0.as_ref(), point.borrow()), ch);
    }

    /// Deletes the text in the specified range.
    pub fn delete(&mut self, range: impl RangeBounds<Cursor>) {
        self.edit(range, "")
    }
}

impl From<&str> for Buffer {
    /// Creates a `Buffer` from a string slice.
    fn from(text: &str) -> Self {
        Self(text.into())
    }
}

// TODO: Implement `View` for `Buffer` slices, so we can call `Cursor` methods using slices.

impl View for Buffer {
    /// Convers this `Buffer` to a string.
    fn as_str(&self) -> &str {
        self.0.as_str()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn origin(&self) -> Cursor {
        Cursor::new(0, 0)
    }

    fn rows(&self) -> usize {
        self.lines().count()
    }

    fn cols_at(&self, line: usize) -> usize {
        self.lines().nth(line).expect("Attempt to index past end of `Buffer`").len()
    }

    fn get(&self, point: Cursor) -> Option<char> {
        self.0.chars().nth(to_offset(self.0.as_ref(), point.borrow()))
    }
}

fn to_offset(buffer: &str, point: &Cursor) -> usize {
    let offset = point.col();

    buffer
        .split('\n')
        .take(point.row())
        .fold(offset, |offset, line| offset + 1 + line.chars().count())
}

fn to_offset_bound(buffer: &str, bound: Bound<&Cursor>) -> Bound<usize> {
    use Bound::{Excluded, Included, Unbounded};

    match bound {
        Unbounded => Unbounded,
        Included(p) => Included(to_offset(buffer, p)),
        Excluded(p) => Excluded(to_offset(buffer, p)),
    }
}
