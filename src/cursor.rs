use std::error::Error;
use std::fmt;

use crate::buffer::BufferView;

/// A text buffer coordinate.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Cursor {
    row: usize,
    col: usize,

    offset: usize,
}

/// The operation was interrupted too soon.
#[derive(Debug, Clone, Copy)]
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
    pub fn at_eol(mut self, buffer: &impl BufferView) -> Self {
        self.col = buffer.cols_at(self.row);
        self
    }

    /// Returns the position at the first character of the current line.
    pub fn at_bol(mut self, _: &impl BufferView) -> Self {
        self.col = 0;
        self
    }

    /// Moves a `Cursor` forwards inside a line by up to the specified amount.
    pub fn try_at_right(mut self, count: usize, buffer: &impl BufferView) -> Result<Self, Partial> {
        let offset = count.min(buffer.cols_at(self.row) - self.col);

        self.col += offset;
        self.offset = self.col;

        if offset == count {
            Ok(self)
        } else {
            Err(Partial { at: self, remaining: count - offset })
        }
    }

    pub fn at_right(self, count: usize, buffer: &impl BufferView) -> Self {
        self.try_at_right(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    /// Moves a `Cursor` backwards inside a line by up to the specified amount.
    pub fn try_at_left(mut self, count: usize, _: &impl BufferView) -> Result<Self, Partial> {
        let offset = count.min(self.col);

        self.col -= offset;
        self.offset = self.col;

        if offset == count {
            Ok(self)
        } else {
            Err(Partial { at: self, remaining: count - offset })
        }
    }

    pub fn at_left(self, count: usize, buffer: &impl BufferView) -> Self {
        self.try_at_left(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    /// Moves a `Cursor` downwards by up to the specified amount.
    pub fn try_below(mut self, count: usize, buffer: &impl BufferView) -> Result<Self, Partial> {
        let offset = count.min(buffer.rows().saturating_sub(1) - self.row);

        self.row += offset;
        self.col = self.offset.min(buffer.cols_at(self.row));

        if offset == count {
            Ok(self)
        } else {
            Err(Partial { at: self, remaining: count - offset })
        }
    }

    pub fn below(self, count: usize, buffer: &impl BufferView) -> Self {
        self.try_below(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    /// Moves a `Cursor` upwards by up to the specified amount.
    pub fn try_above(mut self, count: usize, buffer: &impl BufferView) -> Result<Self, Partial> {
        let offset = count.min(self.row);

        self.row -= offset;
        self.col = self.offset.min(buffer.cols_at(self.row));

        if offset == count {
            Ok(self)
        } else {
            Err(Partial { at: self, remaining: count - offset })
        }
    }

    pub fn above(self, count: usize, buffer: &impl BufferView) -> Self {
        self.try_above(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    /// Advances a cursor while a predicate matches.
    pub fn try_forward_while<P>(
        self,
        buffer: &impl BufferView,
        predicate: P,
    ) -> Result<Self, Partial>
    where
        P: Fn(Self) -> Option<bool>,
    {
        match predicate(self) {
            Some(true) => self.try_forward(1, buffer)?.try_forward_while(buffer, predicate),
            Some(false) => Ok(self),

            None => Err(Partial { at: self, remaining: 1 }),
        }
    }

    pub fn forward_while<P>(self, buffer: &impl BufferView, predicate: P) -> Self
    where
        P: Fn(Self) -> Option<bool>,
    {
        self.try_forward_while(buffer, predicate).unwrap_or_else(|partial| partial.at)
    }

    /// Advances a cursor while a predicate matches.
    pub fn try_backward_while<P>(
        self,
        buffer: &impl BufferView,
        predicate: P,
    ) -> Result<Self, Partial>
    where
        P: Fn(Self) -> Option<bool>,
    {
        match predicate(self) {
            Some(true) => self.try_backward(1, buffer)?.try_backward_while(buffer, predicate),
            Some(false) => Ok(self),

            None => Err(Partial { at: self, remaining: 1 }),
        }
    }

    pub fn backward_while<P>(self, buffer: &impl BufferView, predicate: P) -> Self
    where
        P: Fn(Self) -> Option<bool>,
    {
        self.try_backward_while(buffer, predicate).unwrap_or_else(|partial| partial.at)
    }

    // TODO: Instead of having methods for moving inside lines (`at_left`, `at_right`), across
    // lines (`above`, `below`) and freely (`forward`, `backward`) as `Cursor` members, do
    // something similar to what `xi_rope` [1][2] does:
    //
    // ```
    // pub trait Metric {
    //  fn next(cursor: Cursor, count: usize, buffer: &impl BufferView) -> Result<Cursor, Partial>;
    //  fn prev(cursor: Cursor, count: usize, buffer: &impl BufferView) -> Result<Cursor, Partial>;
    // }
    //
    // struct Characters;
    // struct Lines;
    // struct Words;
    // struct Paragraphs;
    //
    // impl Cursor {
    //     fn try_forward<M: Metric>(self, count: usize, buffer: &impl BufferView) -> Result<Cursor, Partial> {
    //       M::forward(self, count, usize, buffer)
    //     }
    //
    //     fn forward<M: Metric>(self, count: usize, buffer: &impl BufferView) -> Cursor {
    //       M::forward(self, count, buffer).unwrap_or_else(|Partial { at, .. }| at)
    //     }
    //
    //     [...]
    // }
    // ```
    //
    // And maybe even do something like:
    //
    // ```
    // fn iter<M: Metric>(self, buffer: &impl BufferView) -> impl Iterator<Item = BufferView> {
    //   [...]
    // }
    // ```
    //
    // Then, `forward(n, buf)` would become `forward::<Characters>(n, buf)`, `at_left(n, buf)`
    // would become `forward::<Characters>(n, buf.line(row))` or something, etc.
    //
    // That'd make easier to extend with new metrics (i.e., text objects) without doing all
    // boilerplate inside of `Cursor`.
    //
    // We could also have `Metric` be a regular parameter in order to hold state to make defining
    // metrics from Lua easier.
    //
    // [1]: https://docs.rs/xi-rope/0.3.0/xi_rope/tree/struct.Cursor.html
    // [2]: https://docs.rs/xi-rope/0.3.0/xi_rope/tree/trait.Metric.html

    /// Moves a `Cursor` forward.
    pub fn try_forward(self, count: usize, buffer: &impl BufferView) -> Result<Self, Partial> {
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

    pub fn forward(self, count: usize, buffer: &impl BufferView) -> Self {
        self.try_forward(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    /// Moves a `Cursor` backwards.
    pub fn try_backward(self, count: usize, buffer: &impl BufferView) -> Result<Self, Partial> {
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

    pub fn backward(self, count: usize, buffer: &impl BufferView) -> Self {
        self.try_backward(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    pub fn try_forward_words(
        self,
        count: usize,
        buffer: &impl BufferView,
    ) -> Result<Self, Partial> {
        (1..=count).try_fold(self, |cursor, _| {
            cursor
                .try_forward_while(buffer, |p| Some(!buffer.get(p)?.is_whitespace()))?
                .try_forward_while(buffer, |p| Some(buffer.get(p)?.is_whitespace()))
        })
    }

    pub fn forward_words(self, count: usize, buffer: &impl BufferView) -> Self {
        self.try_forward_words(count, buffer).unwrap_or_else(|partial| partial.at)
    }

    pub fn try_backward_words(
        self,
        count: usize,
        buffer: &impl BufferView,
    ) -> Result<Self, Partial> {
        (1..=count).try_fold(self, |cursor, _| {
            cursor
                .try_backward(1, buffer)?
                .try_backward_while(buffer, |p| Some(buffer.get(p)?.is_whitespace()))?
                .try_backward_while(buffer, |p| Some(!buffer.get(p)?.is_whitespace()))?
                .try_forward(1, buffer)
        })
    }

    pub fn backward_words(self, count: usize, buffer: &impl BufferView) -> Self {
        self.try_backward_words(count, buffer).unwrap_or_else(|partial| partial.at)
    }
}

impl Error for Partial {}

impl std::fmt::Display for Partial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Attempt to move {} units past the allowed bounds", self.remaining)
    }
}
