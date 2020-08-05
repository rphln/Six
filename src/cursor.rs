use std::error::Error;
use std::fmt;

use crate::buffer::Buf;

/// A text buffer coordinate.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Cursor {
    row: usize,
    col: usize,

    offset: usize,
}

/// The operation was interrupted too soon.
#[derive(Debug)]
pub struct Partial {
    /// Position at which the interruption occurred.
    pub at: Cursor,

    /// Remaining units to move.
    pub remaining: usize,
}

impl Cursor {
    pub fn new(col: usize, row: usize) -> Self {
        Self { row, col, offset: col }
    }

    /// Returns the horizontal position of this `Cursor`.
    pub fn col(self) -> usize {
        self.col
    }

    /// Returns the vertical position of this `Cursor`.
    pub fn row(self) -> usize {
        self.row
    }

    /// Returns the position at the last character of the current line.
    pub fn at_eol(mut self, buffer: &Buf) -> Self {
        self.col = buffer.cols(self.row);
        self
    }

    /// Returns the position at the first character of the current line.
    pub fn at_bol(mut self, _: &Buf) -> Self {
        self.col = 0;
        self
    }

    /// Moves a `Cursor` forwards inside a line by up to the specified amount.
    pub fn try_at_right(mut self, count: usize, buffer: &Buf) -> Result<Self, Partial> {
        let offset = count.min(buffer.cols(self.row) - self.col);

        self.col += offset;
        self.offset = self.col;

        if offset == count {
            Ok(self)
        } else {
            Err(Partial { at: self, remaining: count - offset })
        }
    }

    // TODO: Replace with a macro.
    pub fn at_right(self, count: usize, buffer: &Buf) -> Self {
        self.try_at_right(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    /// Moves a `Cursor` backwards inside a line by up to the specified amount.
    pub fn try_at_left(mut self, count: usize, _: &Buf) -> Result<Self, Partial> {
        let offset = count.min(self.col);

        self.col -= offset;
        self.offset = self.col;

        if offset == count {
            Ok(self)
        } else {
            Err(Partial { at: self, remaining: count - offset })
        }
    }

    // TODO: Replace with a macro.
    pub fn at_left(self, count: usize, buffer: &Buf) -> Self {
        self.try_at_left(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    /// Moves a `Cursor` downwards by up to the specified amount.
    pub fn try_below(mut self, count: usize, buffer: &Buf) -> Result<Self, Partial> {
        let offset = count.min(buffer.rows().saturating_sub(1) - self.row);

        self.row += offset;
        self.col = self.offset.min(buffer.cols(self.row));

        if offset == count {
            Ok(self)
        } else {
            Err(Partial { at: self, remaining: count - offset })
        }
    }

    // TODO: Replace with a macro.
    pub fn below(self, count: usize, buffer: &Buf) -> Self {
        self.try_below(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    /// Moves a `Cursor` upwards by up to the specified amount.
    pub fn try_above(mut self, count: usize, buffer: &Buf) -> Result<Self, Partial> {
        let offset = count.min(self.row);

        self.row -= offset;
        self.col = self.offset.min(buffer.cols(self.row));

        if offset == count {
            Ok(self)
        } else {
            Err(Partial { at: self, remaining: count - offset })
        }
    }

    // TODO: Replace with a macro.
    pub fn above(self, count: usize, buffer: &Buf) -> Self {
        self.try_above(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    /// Advances a point while a predicate matches.
    pub fn try_forward_while<P>(self, buffer: &Buf, predicate: P) -> Result<Self, Partial>
    where
        P: Fn(Self) -> bool,
    {
        if predicate(self) {
            self.try_forward(1, buffer)?.try_forward_while(buffer, predicate)
        } else {
            Ok(self)
        }
    }

    // TODO: Replace with a macro.
    pub fn forward_while<P>(self, buffer: &Buf, predicate: P) -> Self
    where
        P: Fn(Self) -> bool,
    {
        self.try_forward_while(buffer, predicate).unwrap_or_else(|partial| partial.at)
    }

    /// Advances a point while a predicate matches.
    pub fn try_backward_while<P>(self, buffer: &Buf, predicate: P) -> Result<Self, Partial>
    where
        P: Fn(Self) -> bool,
    {
        if predicate(self) {
            self.try_backward(1, buffer)?.try_backward_while(buffer, predicate)
        } else {
            Ok(self)
        }
    }

    // TODO: Replace with a macro.
    pub fn backward_while<P>(self, buffer: &Buf, predicate: P) -> Self
    where
        P: Fn(Self) -> bool,
    {
        self.try_backward_while(buffer, predicate).unwrap_or_else(|partial| partial.at)
    }

    /// Moves a `Cursor` forward.
    pub fn try_forward(self, count: usize, buffer: &Buf) -> Result<Self, Partial> {
        if count == 0 {
            Ok(self)
        } else {
            self.try_at_right(count, buffer).or_else(|Partial { at, remaining }| {
                at.try_below(1, buffer)
                    .or(Err(Partial { at, remaining }))?
                    .at_bol(buffer)
                    .try_forward(remaining - 1, buffer)
            })
        }
    }

    // TODO: Replace with a macro.
    pub fn forward(self, count: usize, buffer: &Buf) -> Self {
        self.try_forward(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    /// Moves a `Cursor` backwards.
    pub fn try_backward(self, count: usize, buffer: &Buf) -> Result<Self, Partial> {
        if count == 0 {
            Ok(self)
        } else {
            self.try_at_left(count, buffer).or_else(|Partial { at, remaining }| {
                at.try_above(1, buffer)
                    .or(Err(Partial { at, remaining }))?
                    .at_eol(buffer)
                    .try_backward(remaining - 1, buffer)
            })
        }
    }

    // TODO: Replace with a macro.
    pub fn backward(self, count: usize, buffer: &Buf) -> Self {
        self.try_backward(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    pub fn try_forward_words(self, count: usize, buffer: &Buf) -> Result<Self, Partial> {
        (1..=count).try_fold(self, |cursor, _| {
            cursor
                .try_forward_while(buffer, |p| {
                    buffer.get(p).map(|ch| !ch.is_whitespace()).unwrap_or(false)
                })?
                .try_forward_while(buffer, |p| {
                    buffer.get(p).map(|ch| ch.is_whitespace()).unwrap_or(false)
                })
        })
    }

    // TODO: Replace with a macro.
    pub fn forward_words(self, count: usize, buffer: &Buf) -> Self {
        self.try_forward_words(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    pub fn try_backward_words(self, count: usize, buffer: &Buf) -> Result<Self, Partial> {
        (1..=count).try_fold(self, |cursor, _| {
            cursor
                .try_at_left(1, buffer)?
                .try_backward_while(buffer, |p| {
                    buffer.get(p).map(|ch| ch.is_whitespace()).unwrap_or(false)
                })?
                .try_backward_while(buffer, |p| {
                    buffer.get(p).map(|ch| !ch.is_whitespace()).unwrap_or(false)
                })?
                .try_at_right(1, buffer)
        })
    }

    // TODO: Replace with a macro.
    pub fn backward_words(self, count: usize, buffer: &Buf) -> Self {
        self.try_backward_words(count, buffer).unwrap_or_else(|partial| partial.at)
    }
}

impl Error for Partial {}

impl std::fmt::Display for Partial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Attempt to move {} units past the allowed bounds", self.remaining)
    }
}
