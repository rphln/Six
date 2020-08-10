use std::marker::PhantomData;
use std::ops::Not;

use crate::buffer::Buffer;

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

    /// Converts this point to a buffer character offset.
    #[must_use]
    pub fn to_offset(self, buffer: &Buffer) -> usize {
        buffer
            .lines()
            .take(self.row)
            .fold(self.col, |offset, line| offset + line.chars().count() + 1)
    }

    /// Iterates through each cursor position as specified by a unit.
    pub fn iter<'a, Unit>(self, buffer: &'a Buffer) -> Iter<'a, Unit> {
        Iter::new(self, buffer)
    }
}

#[derive(Clone)]
pub struct Iter<'a, Unit> {
    anchor: Cursor,
    buffer: &'a Buffer,

    unit: PhantomData<*const Unit>,
}

impl<'a, Unit> Iter<'a, Unit> {
    pub fn new(anchor: Cursor, buffer: &'a Buffer) -> Self {
        Self { anchor, buffer, unit: PhantomData::default() }
    }

    fn has_whitespace(&self, cursor: Cursor) -> Option<bool> {
        self.buffer.get(cursor.to_offset(self.buffer)).map(char::is_whitespace)
    }

    fn has_character(&self, cursor: Cursor) -> Option<bool> {
        self.has_whitespace(cursor).map(bool::not)
    }

    fn has_line_break(&self, cursor: Cursor) -> Option<bool> {
        self.buffer.get(cursor.to_offset(&self.buffer)).map(|ch| ch == '\n')
    }
}

pub struct Bounded;

impl Iterator for Iter<'_, Bounded> {
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

impl DoubleEndedIterator for Iter<'_, Bounded> {
    /// Moves a `Cursor` forwards inside a line by up to the specified amount.
    fn next_back(&mut self) -> Option<Self::Item> {
        let col = self.anchor.col();
        let row = self.anchor.row();

        self.anchor = if col > 0 { Some(Cursor::new(col - 1, row)) } else { None }?;

        Some(self.anchor)
    }
}

impl Iterator for Iter<'_, char> {
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

impl DoubleEndedIterator for Iter<'_, char> {
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

impl Iterator for Iter<'_, Line> {
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

impl DoubleEndedIterator for Iter<'_, Line> {
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

impl Iterator for Iter<'_, Word> {
    type Item = Cursor;

    /// Moves forward by a word unit.
    fn next(&mut self) -> Option<Self::Item> {
        self.anchor = self.anchor.iter::<char>(self.buffer).find(|&cursor| {
            let offset = cursor.to_offset(self.buffer);

            self.buffer.get(offset - 1).map_or(false, |ch| ch.is_whitespace())
                && self.buffer.get(offset).map_or(false, |ch| !ch.is_whitespace())
        })?;

        Some(self.anchor)
    }
}

impl DoubleEndedIterator for Iter<'_, Word> {
    /// Moves backward by a word unit.
    fn next_back(&mut self) -> Option<Self::Item> {
        // NOTE: In order to match Vim's behaviour, we want either:
        //  (1) A whitespace (at `previous`) followed by a character (at `current`).
        //  (2) The beginning of the buffer (`previous` is `None`) followed by anything.

        self.anchor = self.anchor.iter::<char>(self.buffer).rev().find(|&current| {
            match current.iter::<char>(self.buffer).next_back() {
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

impl Iterator for Iter<'_, EndOfWord> {
    type Item = Cursor;

    /// Moves forward to the end of word.
    fn next(&mut self) -> Option<Self::Item> {
        // NOTE: Read the note for `Word::next`.

        self.anchor = self.anchor.iter::<char>(self.buffer).find(|&cursor| {
            let offset = cursor.to_offset(self.buffer);

            self.buffer.get(offset - 1).map_or(false, |ch| !ch.is_whitespace())
                && self.buffer.get(offset).map_or(false, |ch| ch.is_whitespace())
        })?;

        Some(self.anchor)
    }
}

impl DoubleEndedIterator for Iter<'_, EndOfWord> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.anchor = self
            .anchor
            .iter::<char>(self.buffer)
            .rev()
            .skip_while(|&cursor| self.has_whitespace(cursor).unwrap_or(false))
            .take_while(|&cursor| self.has_character(cursor).unwrap_or(true))
            .next()?;

        Some(self.anchor)
    }
}

pub struct Paragraph;

impl Iterator for Iter<'_, Paragraph> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        self.anchor = self.anchor.iter::<char>(self.buffer).find(|&cursor| {
            match cursor.iter::<char>(self.buffer).next_back() {
                Some(q) => {
                    self.has_line_break(cursor).map_or(false, bool::not)
                        && self.has_line_break(q).unwrap_or(false)
                },
                None => false,
            }
        })?;

        Some(self.anchor)
    }
}

impl DoubleEndedIterator for Iter<'_, Paragraph> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.anchor = self.anchor.iter::<char>(self.buffer).rev().find(|&cursor| {
            match cursor.iter::<char>(self.buffer).next_back() {
                Some(q) => {
                    self.has_line_break(cursor).map_or(false, bool::not)
                        && self.has_line_break(q).unwrap_or(false)
                },
                None => false,
            }
        })?;

        Some(self.anchor)
    }
}
