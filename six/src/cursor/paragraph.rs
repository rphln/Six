use crate::buffer::Buffer;
use crate::cursor::{Codepoint, Cursor, Iter};

pub struct Paragraph<'a> {
    chars: Codepoint<'a>,
    buffer: &'a Buffer,
}

impl<'a> Iter<'a> for Paragraph<'a> {
    fn new(cursor: Cursor, buffer: &'a Buffer) -> Self {
        Self { buffer, chars: Codepoint::new(cursor, buffer) }
    }
}

impl Iterator for Paragraph<'_> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        // let buffer = self.buffer;
        // self.chars.find(|&cursor| {
        //     let p = cursor.index.checked_sub(2).and_then(|idx| buffer.get(idx..=idx));
        //     let q = cursor.index.checked_sub(1).and_then(|idx| buffer.get(idx..=idx));
        //     let r = buffer.get(cursor.index..=cursor.index);

        //     p.map_or(false, |ch| ch != '\n')
        //         && q.map_or(false, |ch| ch == '\n')
        //         && r.map_or(false, |ch| ch == '\n')
        // })
    }
}

impl DoubleEndedIterator for Paragraph<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        todo!()
        // let buffer = self.buffer;
        // self.chars.rfind(|&cursor| {
        //     let idx = cursor.to_index(buffer);

        //     let p = idx.checked_sub(1).and_then(|idx| buffer.get(idx));
        //     let q = buffer.get(idx);
        //     let r = buffer.get(idx + 1);

        //     p.map_or(true, |ch| ch == '\n')
        //         && q.map_or(false, |ch| ch == '\n')
        //         && r.map_or(false, |ch| ch != '\n')
        // })
    }
}
