use std::ops::{Deref, RangeBounds};

#[derive(Debug, Clone, Default)]
pub struct Buffer(String);

impl Buffer {
    /// An iterator over the lines of a string, as string slices.
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.0.split('\n')
    }

    /// Replaces the specified range in the buffer with the given string.
    pub fn replace_range(&mut self, range: impl RangeBounds<usize>, text: &str) {
        self.0.replace_range(range, text.as_ref());
    }

    /// Inserts a character into this `Buffer` at the specified position.
    pub fn insert(&mut self, idx: usize, ch: char) {
        self.0.insert(idx, ch);
    }

    /// Deletes the text in the specified range.
    pub fn delete(&mut self, range: impl RangeBounds<usize>) {
        self.replace_range(range, "")
    }

    /// Convers this `Buffer` to a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.deref()
    }

    /// Returns the number of characters in the buffer.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the `char` at `idx`.
    #[must_use]
    pub fn get(&self, idx: usize) -> Option<char> {
        self[idx..].chars().next()
    }
}

impl From<&str> for Buffer {
    fn from(text: &str) -> Self {
        Self(text.into())
    }
}

impl Deref for Buffer {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.0.deref()
    }
}
