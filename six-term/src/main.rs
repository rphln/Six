//! Six - A Vi-like toy text editor.

#![deny(clippy::all, clippy::pedantic)]
#![feature(generator_trait)]

use std::error::Error;
use std::io;
use std::ops::Generator;
use std::pin::Pin;

use tui::{
    backend::{Backend, TermionBackend},
    layout::{Alignment, Rect},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Paragraph, Wrap},
    Frame, Terminal,
};

use termion::event::Key::{Char, Ctrl};
use termion::{input::MouseTerminal, input::TermRead, raw::IntoRawMode, screen::AlternateScreen};

use six::state::{Editor, Mode, World};

mod buffer_view;

use crate::buffer_view::{Overflow, TextEditState, TextEditView};

fn draw_edit_view<B: Backend>(frame: &mut Frame<B>, area: Rect, state: &Editor) {
    let mut stat = TextEditState::new(state.buffer(), state.cursor());
    let view = TextEditView::new(Overflow::Scroll);

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
    // let mode = match state.mode() {
    //     Mode::Normal { .. } => "Normal",
    //     Mode::Edit { .. } => "Edit",
    //     Mode::Select { .. } => "Select",
    //     Mode::Query { prompt, .. } => prompt,
    //     Mode::Operator { prompt, .. } => prompt,
    // };

    let mode = "Normal";
    let position = state.cursor().to_col(state.buffer()).to_string();

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

    // if let Mode::Query { buffer, cursor, .. } = state.mode() {
    //     let mut stat = TextEditState::new(buffer, *cursor);
    //     let view = TextEditView::new(Overflow::Scroll);

    //     view.focus(chunks[1], frame, &stat);
    //     frame.render_stateful_widget(view, chunks[1], &mut stat);
    // }
}

fn draw<B>(terminal: &mut Terminal<B>, state: &Editor) -> Result<(), Box<dyn Error>>
where
    B: Backend + io::Write,
{
    // match state.mode() {
        // Mode::Edit { .. } => write!(terminal.backend_mut(), "{}", termion::cursor::SteadyBar)?,
        // Mode::Query { .. } => write!(terminal.backend_mut(), "{}", termion::cursor::BlinkingBar)?,

        // _ => write!(terminal.backend_mut(), "{}", termion::cursor::SteadyBlock)?,
    // }

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

    let mut world = World::default();

    loop {
        draw(&mut terminal, world.editor())?;

        if let Some(key) = io::stdin().keys().next() {
            let key = key?;

            if matches!(key, Ctrl('d')) {
                break;
            }

            if let Char(ch) = key {
                world.advance(ch);
            }
        }
    }

    Ok(())
}
