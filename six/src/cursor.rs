use std::marker::PhantomData;
use std::ops::Not;

use crate::buffer::View;

/// A text buffer coordinate.
#[derive(Debug, Clone, Copy, Default, Derivative)]
#[derivative(PartialEq, PartialOrd, Eq, Ord)]
pub struct Cursor {
    /// The cursor row.
    row: usize,

    /// The cursor column, limited to the current line length.
    col: usize,

    /// The raw value of the cursor column.
    #[derivative(PartialEq = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    offset: usize,
}

impl Cursor {
    /// Creates a new cursor at the specified coordinates.
    #[must_use]
    pub fn new(col: usize, row: usize) -> Self {
        Self { row, col, offset: col }
    }

    /// Returns the horizontal position of this `Cursor`.
    #[must_use]
    pub fn col(self) -> usize {
        self.col
    }

    /// Returns the vertical position of this `Cursor`.
    #[must_use]
    pub fn row(self) -> usize {
        self.row
    }

    /// Iterates through each cursor position as specified by a unit.
    pub fn iter<B: View, Unit>(self, buffer: &B) -> Points<'_, B, Unit> {
        Points::new(self, buffer)
    }
}

#[derive(Clone)]
pub struct Points<'a, B: View, Unit> {
    anchor: Cursor,
    buffer: &'a B,

    unit: PhantomData<Unit>,
}

impl<'a, Unit, B: View> Points<'a, B, Unit> {
    pub fn new(anchor: Cursor, buffer: &'a B) -> Self {
        Self { anchor, buffer, unit: PhantomData::default() }
    }

    fn has_whitespace(&self, p: Cursor) -> Option<bool> {
        self.buffer.get(p).map(char::is_whitespace)
    }

    fn has_character(&self, p: Cursor) -> Option<bool> {
        self.has_whitespace(p).map(bool::not)
    }
}

pub struct Bounded;

impl<'a, B: View> Iterator for Points<'a, B, Bounded> {
    type Item = Cursor;

    /// Moves a `Cursor` forwards inside a line by up to the specified amount.
    fn next(&mut self) -> Option<Self::Item> {
        let col = self.anchor.col();
        let row = self.anchor.row();

        self.anchor =
            if col < self.buffer.cols_at(row) { Some(Cursor::new(col + 1, row)) } else { None }?;

        Some(self.anchor)
    }
}

impl<'a, B: View> DoubleEndedIterator for Points<'a, B, Bounded> {
    /// Moves a `Cursor` forwards inside a line by up to the specified amount.
    fn next_back(&mut self) -> Option<Self::Item> {
        let col = self.anchor.col();
        let row = self.anchor.row();

        self.anchor = if col > 0 { Some(Cursor::new(col - 1, row)) } else { None }?;

        Some(self.anchor)
    }
}

pub struct Char;

impl<'a, B: View> Iterator for Points<'a, B, Char> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        let col = self.anchor.col();
        let row = self.anchor.row();

        self.anchor = if col == self.buffer.cols_at(row) {
            if row + 1 < self.buffer.rows() {
                Some(Cursor::new(0, row + 1))
            } else {
                None
            }
        } else {
            Some(Cursor::new(col + 1, row))
        }?;

        Some(self.anchor)
    }
}

impl<'a, B: View> DoubleEndedIterator for Points<'a, B, Char> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let col = self.anchor.col();
        let row = self.anchor.row();

        self.anchor = if col == 0 {
            if row > 0 {
                Some(Cursor::new(0, row - 1))
            } else {
                None
            }
        } else {
            Some(Cursor::new(col - 1, row))
        }?;

        Some(self.anchor)
    }
}

pub struct Line;

impl<'a, B: View> Iterator for Points<'a, B, Line> {
    type Item = Cursor;

    /// Moves a `Cursor` downward.
    fn next(&mut self) -> Option<Self::Item> {
        let row = self.anchor.row();

        self.anchor = if row + 1 < self.buffer.rows() {
            Some(Cursor {
                row: row + 1,
                col: self.anchor.offset.min(self.buffer.cols_at(row + 1)),
                offset: self.anchor.offset,
            })
        } else {
            None
        }?;

        Some(self.anchor)
    }
}

impl<'a, B: View> DoubleEndedIterator for Points<'a, B, Line> {
    /// Moves a `Cursor` upward.
    fn next_back(&mut self) -> Option<Self::Item> {
        let row = self.anchor.row();

        self.anchor = if row > 0 {
            Some(Cursor {
                row: row - 1,
                col: self.anchor.offset.min(self.buffer.cols_at(row - 1)),
                offset: self.anchor.offset,
            })
        } else {
            None
        }?;

        Some(self.anchor)
    }
}

pub struct Word;

impl<'a, B: View> Iterator for Points<'a, B, Word> {
    type Item = Cursor;

    /// Moves forward by a word unit.
    fn next(&mut self) -> Option<Self::Item> {
        // NOTE: This is more like a hack than the proper behaviour; ideally, we'd search for a
        // pair that matches `has_whitespace` into `has_character` in order to be consistent with
        // the `SemanticWord` and `Paragraph` implementations.
        self.anchor = self
            .anchor
            .iter::<_, Char>(self.buffer)
            .skip_while(|&p| self.has_character(p).unwrap_or(false))
            .find(|&p| self.has_character(p).unwrap_or(true))?;

        Some(self.anchor)
    }
}

impl<'a, B: View> DoubleEndedIterator for Points<'a, B, Word> {
    /// Moves backward by a word unit.
    fn next_back(&mut self) -> Option<Self::Item> {
        // NOTE: In order to match Vim's behaviour, we want either:
        //  (1) A whitespace (at `previous`) followed by a character (at `current`).
        //  (2) The beginning of the buffer (`previous` is `None`) followed by anything.

        self.anchor = self.anchor.iter::<_, Char>(self.buffer).rev().find(|&current| {
            match current.iter::<_, Char>(self.buffer).next_back() {
                Some(previous) => {
                    self.has_whitespace(previous).expect("has_whitespace")
                        && self.has_character(current).expect("has_character")
                },

                // We have (2).
                None => true,
            }
        })?;

        Some(self.anchor)
    }
}

pub struct EndOfWord;

impl<'a, B: View> Iterator for Points<'a, B, EndOfWord> {
    type Item = Cursor;

    /// Moves forward to the end of word.
    fn next(&mut self) -> Option<Self::Item> {
        // NOTE: Read the note for `Word::next`.

        self.anchor = self
            .anchor
            .iter::<_, Char>(self.buffer)
            .skip_while(|&p| self.has_whitespace(p).unwrap_or(false))
            .take_while(|&p| self.has_character(p).unwrap_or(true))
            .last()?;

        Some(self.anchor)
    }
}

impl<'a, B: View> DoubleEndedIterator for Points<'a, B, EndOfWord> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.anchor = self
            .anchor
            .iter::<_, Char>(self.buffer)
            .rev()
            .skip_while(|&p| self.has_whitespace(p).unwrap_or(false))
            .take_while(|&p| self.has_character(p).unwrap_or(true))
            .next()?;

        Some(self.anchor)
    }
}

pub struct Paragraph;

impl<'a, B: View> Points<'a, B, Paragraph> {
    fn has_line_break(&self, p: Cursor) -> Option<bool> {
        self.buffer.get(p).map(|ch| ch == '\n')
    }
}

impl<'a, B: View> Iterator for Points<'a, B, Paragraph> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        self.anchor = self.anchor.iter::<_, Char>(self.buffer).find(|&p| {
            match p.iter::<_, Char>(self.buffer).next_back() {
                Some(q) => {
                    self.has_line_break(p).map_or(false, bool::not)
                        && self.has_line_break(q).unwrap_or(false)
                },
                None => false,
            }
        })?;

        Some(self.anchor)
    }
}

impl<'a, B: View> DoubleEndedIterator for Points<'a, B, Paragraph> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.anchor = self.anchor.iter::<_, Char>(self.buffer).rev().find(|&p| {
            match p.iter::<_, Char>(self.buffer).next_back() {
                Some(q) => {
                    self.has_line_break(p).map_or(false, bool::not)
                        && self.has_line_break(q).unwrap_or(false)
                },
                None => false,
            }
        })?;

        Some(self.anchor)
    }
}
