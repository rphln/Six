use crate::buffer::Buffer;

#[derive(Debug, Clone, Copy, Default, PartialOrd, PartialEq)]
pub struct Cursor {
    y: usize,
    x: usize,

    w: usize,
}

pub enum ErrorKind {
    /// The operation was interrupted too soon.
    Interrupted {
        /// Position at which the interruption occurred.
        at: Cursor,

        /// Remaining units to move.
        remaining: usize,
    },
}

impl Cursor {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y, w: 0 }
    }

    /// Returns the horizontal position of this `Point`.
    pub fn col(self, _: &impl Buffer) -> usize {
        self.x
    }

    /// Returns the vertical position of this `Point`.
    pub fn row(self, _: &impl Buffer) -> usize {
        self.y
    }

    /// Returns the position at the last character of the current line.
    pub fn eol(mut self, buffer: &impl Buffer) -> Self {
        self.x = buffer.cols(self.y);
        self.w = self.x;

        self
    }

    /// Returns the position at the first character of the current line.
    pub fn bol(mut self, _: &impl Buffer) -> Self {
        self.x = 0;
        self.w = self.x;

        self
    }

    /// Moves a `Point` forwards inside a line by up to the specified amount.
    pub fn right(mut self, count: usize, buffer: &impl Buffer) -> Result<Self, ErrorKind> {
        let offset = count.min(buffer.cols(self.y) - self.x);

        self.x += offset;
        self.w = self.x;

        if offset == count {
            Ok(self)
        } else {
            Err(ErrorKind::Interrupted {
                at: self,
                remaining: count - offset,
            })
        }
    }

    /// Moves a `Point` backwards inside a line by up to the specified amount.
    pub fn left(mut self, count: usize, _: &impl Buffer) -> Result<Self, ErrorKind> {
        let offset = count.min(self.x);

        self.x -= offset;
        self.w = self.x;

        if offset == count {
            Ok(self)
        } else {
            Err(ErrorKind::Interrupted {
                at: self,
                remaining: count - offset,
            })
        }
    }

    /// Moves a `Point` downwards by up to the specified amount.
    pub fn down(mut self, count: usize, buffer: &impl Buffer) -> Result<Self, ErrorKind> {
        let offset = count.min(buffer.rows().saturating_sub(1));

        self.y += offset;
        self.x = self.w.min(buffer.cols(self.y));

        if offset == count {
            Ok(self)
        } else {
            Err(ErrorKind::Interrupted {
                at: self,
                remaining: count - offset,
            })
        }
    }

    /// Moves a `Point` upwards by up to the specified amount.
    pub fn up(mut self, count: usize, buffer: &impl Buffer) -> Result<Self, ErrorKind> {
        let offset = count.min(self.y);

        self.y -= offset;
        self.x = self.w.min(buffer.cols(self.y));

        if offset == count {
            Ok(self)
        } else {
            Err(ErrorKind::Interrupted {
                at: self,
                remaining: count - offset,
            })
        }
    }

    /// Advances a point while a predicate matches.
    pub fn forward_while<B, P>(self, buffer: &B, predicate: P) -> Result<Self, ErrorKind>
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
    pub fn backward_while<B, P>(self, buffer: &B, predicate: P) -> Result<Self, ErrorKind>
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
    pub fn forward<B>(self, count: usize, buffer: &B) -> Result<Self, ErrorKind>
    where
        B: Buffer,
    {
        if count == 0 {
            Ok(self)
        } else {
            self.right(count, buffer).or_else(|err| match err {
                ErrorKind::Interrupted { at, remaining } => at
                    .down(1, buffer)
                    .or(Err(err))?
                    .bol(buffer)
                    .forward(remaining - 1, buffer),
            })
        }
    }

    /// Moves a `Point` backwards.
    pub fn backward<B>(self, count: usize, buffer: &B) -> Result<Self, ErrorKind>
    where
        B: Buffer,
    {
        if count == 0 {
            Ok(self)
        } else {
            self.left(count, buffer).or_else(|err| match err {
                ErrorKind::Interrupted { at, remaining } => at
                    .up(1, buffer)
                    .or(Err(err))?
                    .eol(buffer)
                    .backward(remaining - 1, buffer),
            })
        }
    }

    pub fn forward_words<B>(self, count: usize, buffer: &B) -> Result<Cursor, ErrorKind>
    where
        B: Buffer,
    {
        (1..=count).try_fold(self, |cursor, _| {
            cursor
                .forward_while(buffer, |p| {
                    buffer.get(p).map(|ch| !ch.is_whitespace()).unwrap_or(false)
                })?
                .forward_while(buffer, |p| {
                    buffer.get(p).map(|ch| ch.is_whitespace()).unwrap_or(false)
                })
        })
    }

    pub fn backward_words<B>(self, count: usize, buffer: &B) -> Result<Cursor, ErrorKind>
    where
        B: Buffer,
    {
        (1..=count).try_fold(self, |cursor, _| {
            cursor
                .left(1, buffer)?
                .backward_while(buffer, |p| {
                    buffer.get(p).map(|ch| ch.is_whitespace()).unwrap_or(false)
                })?
                .backward_while(buffer, |p| {
                    buffer.get(p).map(|ch| !ch.is_whitespace()).unwrap_or(false)
                })?
                .right(1, buffer)
        })
    }
}

pub fn unwrap(result: Result<Cursor, ErrorKind>) -> Cursor {
    result.unwrap_or_else(|err| match err {
        ErrorKind::Interrupted { at, .. } => at,
    })
}

#[cfg(test)]
mod tests {}
