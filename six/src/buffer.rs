use std::ops::RangeBounds;

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
    pub fn insert(&mut self, offset: usize, ch: char) {
        self.0.insert(offset, ch);
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

    /// Returns the `char` at `offset`.
    pub fn get(&self, offset: usize) -> Option<char> {
        self.0.chars().nth(offset)
    }

    /// Returns the number of lines in the buffer.
    pub fn rows(&self) -> usize {
        self.lines().count()
    }

    /// Returns the number of characters in the specified line, excluding the line break.
    pub fn cols_at(&self, line: usize) -> usize {
        self.lines().nth(line).expect("Attempt to index past end of `Buffer`").len()
    }
}

impl From<&str> for Buffer {
    fn from(text: &str) -> Self {
        Self(text.into())
    }
}
