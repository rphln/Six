use crate::buffer::Buffer;

#[derive(Debug, Clone, Copy, Default, PartialOrd, PartialEq)]
pub struct Cursor {
    y: usize,
    x: usize,

    w: usize,
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
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y, w: 0 }
    }

    /// Returns the horizontal position of this `Point`.
    pub fn col(self) -> usize {
        self.x
    }

    /// Returns the vertical position of this `Point`.
    pub fn row(self) -> usize {
        self.y
    }

    /// Returns the position at the last character of the current line.
    pub fn at_eol(mut self, buffer: &impl Buffer) -> Self {
        self.x = buffer.cols(self.y);
        self.w = self.x;
        self
    }

    /// Returns the position at the first character of the current line.
    pub fn at_bol(mut self, _: &impl Buffer) -> Self {
        self.x = 0;
        self.w = self.x;
        self
    }

    /// Moves a `Point` forwards inside a line by up to the specified amount.
    pub fn at_right(mut self, count: usize, buffer: &impl Buffer) -> Result<Self, Partial> {
        let offset = count.min(buffer.cols(self.y) - self.x);

        self.x += offset;
        self.w = self.x;

        if offset == count {
            Ok(self)
        } else {
            Err(Partial {
                at: self,
                remaining: count - offset,
            })
        }
    }

    /// Moves a `Point` backwards inside a line by up to the specified amount.
    pub fn at_left(mut self, count: usize, _: &impl Buffer) -> Result<Self, Partial> {
        let offset = count.min(self.x);

        self.x -= offset;
        self.w = self.x;

        if offset == count {
            Ok(self)
        } else {
            Err(Partial {
                at: self,
                remaining: count - offset,
            })
        }
    }

    /// Moves a `Point` downwards by up to the specified amount.
    pub fn below(mut self, count: usize, buffer: &impl Buffer) -> Result<Self, Partial> {
        let offset = count.min(buffer.rows().saturating_sub(1) - self.y);

        self.y += offset;
        self.x = self.w.min(buffer.cols(self.y));

        if offset == count {
            Ok(self)
        } else {
            Err(Partial {
                at: self,
                remaining: count - offset,
            })
        }
    }

    /// Moves a `Point` upwards by up to the specified amount.
    pub fn above(mut self, count: usize, buffer: &impl Buffer) -> Result<Self, Partial> {
        let offset = count.min(self.y);

        self.y -= offset;
        self.x = self.w.min(buffer.cols(self.y));

        if offset == count {
            Ok(self)
        } else {
            Err(Partial {
                at: self,
                remaining: count - offset,
            })
        }
    }

    /// Advances a point while a predicate matches.
    pub fn forward_while<B, P>(self, buffer: &B, predicate: P) -> Result<Self, Partial>
    where
        B: Buffer,
        P: Fn(Self) -> bool,
    {
        if predicate(self) {
            self.forward(1, buffer)?.forward_while(buffer, predicate)
        } else {
            Ok(self)
        }
    }

    /// Advances a point while a predicate matches.
    pub fn backward_while<B, P>(self, buffer: &B, predicate: P) -> Result<Self, Partial>
    where
        B: Buffer,
        P: Fn(Self) -> bool,
    {
        if predicate(self) {
            self.backward(1, buffer)?.backward_while(buffer, predicate)
        } else {
            Ok(self)
        }
    }

    /// Moves a `Point` forward.
    pub fn forward<B>(self, count: usize, buffer: &B) -> Result<Self, Partial>
    where
        B: Buffer,
    {
        if count == 0 {
            Ok(self)
        } else {
            self.at_right(count, buffer)
                .or_else(|Partial { at, remaining }| {
                    at.below(1, buffer)
                        .or(Err(Partial { at, remaining }))?
                        .at_bol(buffer)
                        .forward(remaining - 1, buffer)
                })
        }
    }

    /// Moves a `Point` backwards.
    pub fn backward<B>(self, count: usize, buffer: &B) -> Result<Self, Partial>
    where
        B: Buffer,
    {
        if count == 0 {
            Ok(self)
        } else {
            self.at_left(count, buffer)
                .or_else(|Partial { at, remaining }| {
                    at.above(1, buffer)
                        .or(Err(Partial { at, remaining }))?
                        .at_eol(buffer)
                        .backward(remaining - 1, buffer)
                })
        }
    }

    pub fn forward_words<B>(self, count: usize, buffer: &B) -> Result<Self, Partial>
    where
        B: Buffer,
    {
        (1..=count).try_fold(self, |cursor, _| {
            cursor
                .forward_while(buffer, |p| {
                    buffer
                        .get(&p)
                        .map(|ch| !ch.is_whitespace())
                        .unwrap_or(false)
                })?
                .forward_while(buffer, |p| {
                    buffer.get(&p).map(|ch| ch.is_whitespace()).unwrap_or(false)
                })
        })
    }

    pub fn backward_words<B>(self, count: usize, buffer: &B) -> Result<Self, Partial>
    where
        B: Buffer,
    {
        (1..=count).try_fold(self, |cursor, _| {
            cursor
                .at_left(1, buffer)?
                .backward_while(buffer, |p| {
                    buffer.get(&p).map(|ch| ch.is_whitespace()).unwrap_or(false)
                })?
                .backward_while(buffer, |p| {
                    buffer
                        .get(&p)
                        .map(|ch| !ch.is_whitespace())
                        .unwrap_or(false)
                })?
                .at_right(1, buffer)
        })
    }
}

#[cfg(test)]
mod tests {}
