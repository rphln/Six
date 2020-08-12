use std::ops::{Index, RangeBounds};

#[derive(Debug, Clone, Default)]
pub struct Buffer(String);

impl Buffer {
    /// An iterator over the lines of a string, as string slices.
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.0.split('\n')
    }

    /// Replaces the specified range in the buffer with the given string.
    pub fn edit(&mut self, range: impl RangeBounds<usize>, text: &str) {
        self.0.replace_range(range, text.as_ref());
    }

    /// Inserts a character into this `Buffer` at the specified position.
    pub fn insert(&mut self, idx: usize, ch: char) {
        self.0.insert(idx, ch);
    }

    /// Deletes the text in the specified range.
    pub fn delete(&mut self, range: impl RangeBounds<usize>) {
        self.edit(range, "")
    }

    /// Convers this `Buffer` to a string.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the number of characters in the buffer.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the `char` at `idx`.
    pub fn get(&self, idx: usize) -> Option<char> {
        self.0[idx..].chars().next()
    }

    /// Returns the number of lines in the buffer.
    pub fn rows(&self) -> usize {
        self.lines().count()
    }

    /// Returns the number of characters in the specified line, excluding the line break.
    pub fn cols_at(&self, line: usize) -> usize {
        self.lines().nth(line).expect("Attempt to index past end of `Buffer`").chars().count()
    }
}

impl From<&str> for Buffer {
    fn from(text: &str) -> Self {
        Self(text.into())
    }
}

impl<R: RangeBounds<usize>> Index<R> for Buffer {
    type Output = str;

    fn index(&self, index: R) -> &str {
        use std::ops::Bound::{Excluded, Included, Unbounded};

        let buf = self.as_str();

        match (index.start_bound(), index.end_bound()) {
            (Included(&p), Included(&q)) => &buf[p..=q],
            (Included(&p), Excluded(&q)) => &buf[p..q],
            (Included(&p), Unbounded) => &buf[p..],

            (Excluded(&p), Included(&q)) => &buf[(p + 1)..=q],
            (Excluded(&p), Excluded(&q)) => &buf[(p + 1)..q],
            (Excluded(&p), Unbounded) => &buf[(p + 1)..],

            (Unbounded, Included(&q)) => &buf[..=q],
            (Unbounded, Excluded(&q)) => &buf[..q],
            (Unbounded, Unbounded) => &buf[..],
        }
    }
}
