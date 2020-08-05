use crate::state::State;
use std::borrow::Borrow;
use std::marker::PhantomData;
use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, StatefulWidget, Widget},
    Frame,
};

pub struct TextEditState<'a> {
    buf: &'a str,

    col: u16,
    row: u16,
}

impl<'a, S: Borrow<State>> From<&'a S> for TextEditState<'a> {
    fn from(state: &'a S) -> Self {
        let state = state.borrow();

        let buf = state.buf().as_str();

        let row = state.row() as u16;
        let col = state.col() as u16;

        Self { buf, row, col }
    }
}

impl<'a> TextEditState<'a> {
    pub fn new(buf: &'a str, col: u16, row: u16) -> Self {
        Self { buf, col, row }
    }
}

#[derive(Default)]
pub struct TextEditView<'a> {
    _phantom: PhantomData<&'a ()>,
}

impl<'a> TextEditView<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    fn area(&self, area: Rect) -> (u16, u16) {
        let width = area.width.saturating_sub(1);
        let height = area.height.saturating_sub(1);

        (width, height)
    }

    fn scroll(&self, area: Rect, state: &TextEditState) -> (u16, u16) {
        let (width, height) = self.area(area);

        let x = state.col.saturating_sub(width);
        let y = state.row.saturating_sub(height);

        (y, x)
    }

    pub fn focus<B: Backend>(&self, area: Rect, frame: &mut Frame<B>, state: &TextEditState) {
        let (width, height) = self.area(area);

        let x = area.x + state.col.min(width);
        let y = area.y + state.row.min(height);

        frame.set_cursor(x, y);
    }
}

impl<'a> StatefulWidget for TextEditView<'a> {
    type State = TextEditState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Paragraph::new(state.buf).scroll(self.scroll(area, state)).render(area, buf)
    }
}
