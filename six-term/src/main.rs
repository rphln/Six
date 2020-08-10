//! Six - A Vi-like toy text editor.

#![deny(clippy::all, clippy::pedantic)]

use std::error::Error;
use std::io;

use tui::{
    backend::{Backend, TermionBackend},
    layout::{Alignment, Rect},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Paragraph, Wrap},
    Frame, Terminal,
};

use termion::{input::MouseTerminal, input::TermRead, raw::IntoRawMode, screen::AlternateScreen};

use six::{
    buffer::View,
    state::{handle_key, Action, Editor, Mode, TextObject},
    Lua,
};

mod buffer_view;

use crate::buffer_view::{Overflow, TextEditState, TextEditView};

fn draw_edit_view<B: Backend>(frame: &mut Frame<B>, area: Rect, state: &Editor) {
    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(4), Constraint::Min(1)])
        .split(area);

    let ruler = area[0];
    let body = area[1];

    let mut stat = TextEditState::new(state.content(), state.cursor());
    let view = TextEditView::new(Overflow::Scroll);

    let (y, _) = view.scroll(body, &stat);

    // TODO: Don't pointlessy render all markers.
    let markers: Vec<_> = (1..=state.content().rows())
        .map(|n| {
            let style = if n == state.cursor().row() + 1 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            Spans::from(vec![Span::styled(n.to_string(), style), Span::raw(" ")])
        })
        .collect();
    let markers = Paragraph::new(markers)
        .alignment(Alignment::Right)
        .scroll((y, 0))
        .style(Style::default().fg(Color::Black));

    view.focus(body, frame, &stat);

    frame.render_widget(markers, ruler);
    frame.render_stateful_widget(view, body, &mut stat);
}

fn draw_debug_view<B: Backend>(frame: &mut Frame<B>, area: Rect, state: &Editor) {
    let debug = format!("{:#?}", state);
    let debug = Paragraph::new(debug.as_ref())
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::Black));

    frame.render_widget(debug, area);
}

fn draw_status_line<B: Backend>(frame: &mut Frame<B>, area: Rect, state: &Editor) {
    let mode = match state.mode() {
        Mode::Normal { .. } => "Normal",
        Mode::Edit { .. } => "Edit",
        Mode::Select { .. } => "Select",
        Mode::Query { prompt, .. } => prompt,
        Mode::Operator { prompt, .. } => prompt,
    };

    let position = state.cursor().col().to_string();

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

    if let Mode::Query { content, cursor, .. } = state.mode() {
        let mut stat = TextEditState::new(content, *cursor);
        let view = TextEditView::new(Overflow::Scroll);

        view.focus(chunks[1], frame, &stat);
        frame.render_stateful_widget(view, chunks[1], &mut stat);
    }
}

fn draw<B>(terminal: &mut Terminal<B>, state: &Editor) -> Result<(), Box<dyn Error>>
where
    B: Backend + io::Write,
{
    match state.mode() {
        Mode::Edit { .. } => write!(terminal.backend_mut(), "{}", termion::cursor::SteadyBar)?,
        Mode::Query { .. } => write!(terminal.backend_mut(), "{}", termion::cursor::BlinkingBar)?,

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

    let mut state = Editor::default();
    let mut lua = Lua::new();

    loop {
        draw(&mut terminal, &state)?;

        if let Some(key) = io::stdin().keys().next() {
            use termion::event::Key::{Char, Ctrl, Esc};
            use Mode::{Edit, Normal, Query};

            let key = key?;

            if matches!(key, Ctrl('d')) {
                break;
            }

            let action = match (state.mode(), key) {
                (Edit { .. }, Char(ch)) => Action::Input(ch),
                (Edit { .. }, Esc) => Action::Escape { backward: true },

                (Query { .. }, Char(ch)) => Action::Input(ch),
                (Query { .. }, Esc) => Action::Escape { backward: false },

                (Normal { .. }, Char('i')) => Action::ToInsert { forward: false },
                (Normal { .. }, Char('a')) => Action::ToInsert { forward: true },

                (Normal { .. }, Char(';')) => Action::ToEval {},

                (_, Char('s')) => Action::ToSurround {},

                (_, Char('0')) => Action::TextObject(TextObject::Bol),
                (_, Char('W')) => Action::TextObject(TextObject::Word),
                (_, Char('E')) => Action::TextObject(TextObject::Eow),

                _ => todo!(),
            };

            handle_key(&mut state, &mut lua, action)?;
        }
    }

    Ok(())
}