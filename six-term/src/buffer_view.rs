use std::borrow::Borrow;
use std::marker::PhantomData;

use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, StatefulWidget, Widget, Wrap},
    Frame,
};

use six::buffer::View;
use six::Cursor;

pub struct Col(pub u16);
pub struct Row(pub u16);

pub struct TextEditState<'a> {
    buf: &'a str,

    col: u16,
    row: u16,
}

impl<'a> TextEditState<'a> {
    pub fn new(buffer: &'a impl View, cursor: Cursor) -> Self {
        let buf = buffer.as_str();

        let row = cursor.row() as u16;
        let col = cursor.col() as u16;

        Self { buf, row, col }
    }
}

pub enum Overflow {
    Wrap,
    Scroll,
}

pub struct TextEditView<'a> {
    overflow: Overflow,

    _phantom: PhantomData<&'a ()>,
}

impl<'a> TextEditView<'a> {
    pub fn new(overflow: Overflow) -> Self {
        Self { overflow, _phantom: PhantomData::default() }
    }

    pub fn scroll(&self, area: Rect, state: &TextEditState) -> (u16, u16) {
        let x = state.col.saturating_sub(area.width - 1);
        let y = state.row.saturating_sub(area.height - 1);

        (y, x)
    }

    pub fn focus<B: Backend>(&self, area: Rect, frame: &mut Frame<B>, state: &TextEditState) {
        let x = area.x + state.col.min(area.width - 1);
        let y = area.y + state.row.min(area.height - 1);

        frame.set_cursor(x, y);
    }
}

impl<'a> StatefulWidget for TextEditView<'a> {
    type State = TextEditState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let paragraph = Paragraph::new(state.buf);
        let paragraph = match self.overflow {
            Overflow::Wrap => paragraph.wrap(Wrap { trim: false }),
            Overflow::Scroll => paragraph.scroll(self.scroll(area, state)),
        };

        paragraph.render(area, buf)
    }
}
