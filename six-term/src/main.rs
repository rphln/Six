//! Six - A Vi-like toy text editor.

#![deny(clippy::all, clippy::pedantic)]

use std::error::Error;
use std::io;
use std::marker::PhantomData;

use tui::backend::{Backend, TermionBackend};
use tui::buffer::Buffer;
use tui::layout::{Alignment, Rect};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Paragraph, StatefulWidget, Widget, Wrap};
use tui::{Frame, Terminal};

use termion::event::Key;
use termion::{input::MouseTerminal, input::TermRead, raw::IntoRawMode, screen::AlternateScreen};

use six::{Editor, Event as Ev, Mode};

#[derive(Default)]
pub struct TextEditState<'a> {
    buffer: &'a str,

    col: u16,
    row: u16,
}

impl<'a> TextEditState<'a> {
    fn new(buffer: &'a six::Buffer, cursor: six::Cursor) -> Self {
        let col = cursor.to_col(buffer) as u16;
        let row = cursor.to_row(buffer) as u16;

        Self { col, row, buffer: buffer.as_str() }
    }
}

impl<'a> From<&'a Editor> for TextEditState<'a> {
    fn from(state: &'a Editor) -> TextEditState<'a> {
        TextEditState::new(state.state().buffer(), state.state().cursor())
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

        (x, y)
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

fn draw_debug_view<B: Backend>(frame: &mut Frame<B>, area: Rect, state: &Editor) {
    let debug = format!("{:#?}", state);
    let debug = Paragraph::new(debug.as_ref())
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::Black));

    frame.render_widget(debug, area);
}

fn draw_status_line<B: Backend>(frame: &mut Frame<B>, area: Rect, state: &Editor) {
    let mode = state.mode().name();
    let position = state.state().cursor().to_col(state.state().buffer()).to_string();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Length(mode.len() as u16 + 1),
            Constraint::Min(1),
            Constraint::Length(position.len() as u16 + 1),
        ])
        .split(area);

    let emphasis = Style::default().fg(Color::Green);

    let mode = Span::styled(mode, emphasis);
    let mode = Paragraph::new(mode);

    let position = Span::styled(position, emphasis);
    let position = Paragraph::new(position).alignment(Alignment::Right);

    frame.render_widget(mode, chunks[0]);
    frame.render_widget(position, chunks[2]);

    if let Mode::Query { buffer, cursor, .. } = state.mode() {
        let mut stat = TextEditState::new(buffer, *cursor);
        let view = TextEditView::default();

        view.focus(chunks[1], frame, &stat);
        frame.render_stateful_widget(view, chunks[1], &mut stat);
    }
}

fn draw<B>(terminal: &mut Terminal<B>, state: &Editor) -> Result<(), Box<dyn Error>>
where
    B: Backend + io::Write,
{
    match state.mode() {
        Mode::Insert { .. } => write!(terminal.backend_mut(), "{}", termion::cursor::SteadyBar)?,
        // Mode::Query { .. } => write!(terminal.backend_mut(), "{}",
        // termion::cursor::BlinkingBar)?,
        _ => write!(terminal.backend_mut(), "{}", termion::cursor::SteadyBlock)?,
    }

    terminal.draw(|frame| {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)])
            .split(frame.size());

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(horizontal[0]);

        draw_edit_view(frame, vertical[0], state);
        draw_status_line(frame, vertical[1], state);

        draw_debug_view(frame, horizontal[1], state);
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

    loop {
        draw(&mut terminal, &editor)?;

        if let Some(event) = io::stdin().keys().next() {
            match event? {
                Key::Ctrl('d') => break,

                Key::Char(ch) => editor.advance(Ev::Char(ch)),
                Key::Esc => editor.advance(Ev::Esc),

                _ => continue,
            }
        }
    }

    Ok(())
}
