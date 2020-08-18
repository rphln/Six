//! Six - A Vi-like toy text editor.

#![deny(clippy::all, clippy::pedantic)]

use std::error::Error;
use std::io;
use std::marker::PhantomData;

use tui::backend::{Backend, TermionBackend};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Paragraph, StatefulWidget, Widget};
use tui::{Frame, Terminal};

use termion::event;
use termion::{input::MouseTerminal, input::TermRead, raw::IntoRawMode, screen::AlternateScreen};

use six::{Cursor, Editor, Key, Mode};

#[derive(Default)]
pub struct TextEditState<'a> {
    buffer: &'a str,

    col: u16,
    row: u16,
}

impl<'a> TextEditState<'a> {
    fn new(buffer: &'a str, cursor: Cursor) -> Self {
        let col = cursor.to_col(buffer) as u16;
        let row = cursor.to_row(buffer) as u16;

        Self { col, row, buffer }
    }
}

impl<'a> From<&'a Editor> for TextEditState<'a> {
    fn from(state: &'a Editor) -> TextEditState<'a> {
        TextEditState::new(state.content(), state.cursor())
    }
}

#[derive(Default)]
pub struct TextEditView<'a> {
    phantom: PhantomData<&'a ()>,
}

impl TextEditView<'_> {
    #[must_use]
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
        Paragraph::new(state.buffer).scroll(self.scroll(area, state)).render(area, buf)
    }
}

fn draw_edit_view<B: Backend>(frame: &mut Frame<B>, area: Rect, state: &Editor) {
    let mut stat = TextEditState::from(state);
    let view = TextEditView::default();

    view.focus(area, frame, &stat);
    frame.render_stateful_widget(view, area, &mut stat);
}

fn draw_status_line<B: Backend>(frame: &mut Frame<B>, area: Rect, state: &Editor) {
    let mode = state.mode().name();
    let position = state.cursor().to_col(state.content()).to_string();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Length(mode.len() as u16 + 1),
            Constraint::Min(1),
            Constraint::Length(position.len() as u16 + 1),
        ])
        .split(area);

    let mode = Paragraph::new(mode);
    let position = Paragraph::new(position.as_ref());

    frame.render_widget(mode, chunks[0]);
    frame.render_widget(position, chunks[2]);

    if let Mode::Query { buffer, .. } = state.mode() {
        let mut stat = TextEditState::new(buffer.as_str(), buffer.cursor());
        let view = TextEditView::default();

        view.focus(chunks[1], frame, &stat);
        frame.render_stateful_widget(view, chunks[1], &mut stat);
    }
}

fn draw<B: Backend>(terminal: &mut Terminal<B>, state: &Editor) -> Result<(), Box<dyn Error>> {
    terminal.draw(|frame| {
        let parts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(frame.size());

        draw_edit_view(frame, parts[0], state);
        draw_status_line(frame, parts[1], state);
    })?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;

    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);

    let backend = TermionBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    let mut editor = Editor::default();

    draw(&mut terminal, &editor)?;

    for event in io::stdin().keys() {
        match event? {
            event::Key::Ctrl('d') => break,

            event::Key::Char(ch) => editor.handle_key(Key::Char(ch)),
            event::Key::Esc => editor.handle_key(Key::Esc),

            event::Key::Delete => editor.handle_key(Key::Delete),
            event::Key::Backspace => editor.handle_key(Key::Backspace),

            event::Key::Left => editor.handle_key(Key::Left),
            event::Key::Right => editor.handle_key(Key::Right),
            event::Key::Up => editor.handle_key(Key::Up),
            event::Key::Down => editor.handle_key(Key::Down),

            _ => continue,
        }

        draw(&mut terminal, &editor)?;
    }

    Ok(())
}
