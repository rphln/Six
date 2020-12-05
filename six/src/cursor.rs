mod cells;
mod paragraphs;

pub use cells::Cells;
pub use paragraphs::Paragraphs;

/// A text text coordinate.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Cursor {
    /// The vertical position of the cursor.
    row: usize,

    /// The horizontal position of the cursor.
    col: usize,
}

impl Cursor {
    /// Creates a new cursor at the specified position.
    #[inline]
    #[must_use]
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// Creates a new cursor at the initial position of the buffer.
    #[inline]
    #[must_use]
    pub fn origin() -> Self {
        Self { row: 0, col: 0 }
    }

    /// Returns the horizontal position of this `Cursor`.
    #[inline]
    #[must_use]
    pub fn col(self) -> usize {
        self.col
    }

    /// Returns the vertical position of this `Cursor`.
    #[inline]
    #[must_use]
    pub fn row(self) -> usize {
        self.row
    }
}
