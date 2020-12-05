use crate::cursor::Cells;
use crate::{Buffer, Cursor};

pub struct Paragraphs<'a> {
    buffer: &'a Buffer,
    cursor: Cursor,
}

impl<'b> Paragraphs<'b> {
    pub fn new(cursor: Cursor, buffer: &'b Buffer) -> Self {
        Self { buffer, cursor }
    }
}

fn find(buffer: &Buffer, cells: impl Iterator<Item = Cursor>) -> Option<Cursor> {
    let mut lookahead = {
        let mut cells = itertools::multipeek(cells);

        std::iter::from_fn(move || {
            let p = cells.next()?;
            let q = cells.peek().and_then(|&q| buffer.get(q));
            let r = cells.peek().and_then(|&r| buffer.get(r));

            Some((p, q, r))
        })
    };

    let point = lookahead.find(|&(p, q, r)| {
        let p = buffer.get(p);

        !p.map_or(true, |ch| ch == '\n')
            && q.map_or(true, |ch| ch == '\n')
            && r.map_or(true, |ch| ch == '\n')
    })?;

    Some(point.0)
}

impl Iterator for Paragraphs<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        find(self.buffer, Cells::new(self.cursor, self.buffer))
    }
}

impl DoubleEndedIterator for Paragraphs<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        find(self.buffer, Cells::new(self.cursor, self.buffer).rev())
    }
}
