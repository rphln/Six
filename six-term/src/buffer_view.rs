use std::marker::PhantomData;

use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, StatefulWidget, Widget, Wrap},
    Frame,
};

pub struct TextEditState<'a> {
    /// The text content of the editor view.
    content: &'a str,

    /// The column of the editor view's cursor.
    col: u16,

    /// The row of the editor view's cursor.
    row: u16,
}

impl<'a> TextEditState<'a> {
    /// Initializes the editor view state from a string and a cursor.
    pub fn new(content: &'a str, cursor: six::Cursor) -> Self {
        let col = cursor.col() as u16;
        let row = cursor.row() as u16;

        Self { content, col, row }
    }
}

pub enum Overflow {
    Wrap,
    Scroll,
}

pub struct TextEditView<'a> {
    overflow: Overflow,

    phantom: PhantomData<&'a ()>,
}

impl TextEditView<'_> {
    pub fn new(overflow: Overflow) -> Self {
        Self { overflow, phantom: PhantomData::default() }
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
        let paragraph = Paragraph::new(state.content);
        let paragraph = match self.overflow {
            Overflow::Wrap => paragraph.wrap(Wrap { trim: false }),
            Overflow::Scroll => paragraph.scroll(self.scroll(area, state)),
        };

        paragraph.render(area, buf)
    }
}
