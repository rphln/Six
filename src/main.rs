use std::error::Error;
use std::io;

use tui::{
    backend::{Backend, TermionBackend},
    layout::{Alignment, Rect},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::Paragraph,
    Frame, Terminal,
};

use termion::{input::MouseTerminal, input::TermRead, raw::IntoRawMode, screen::AlternateScreen};

use six::{
    state::{event_loop, Mode, State},
    ui::buffer_view::{TextEditState, TextEditView},
};

fn draw_status_line<B: Backend>(frame: &mut Frame<B>, area: Rect, state: &mut State) {
    let mode = match state.mode() {
        Mode::Normal { .. } => "Normal",
        Mode::Edit { .. } => "Edit",
        Mode::Select { .. } => "Select",
        Mode::Query { prompt, .. } => prompt.as_ref(),
        Mode::Operator { prompt, .. } => prompt.as_ref(),
    };

    let ruler = format!("{},{}", state.row(), state.col());

    let partial =
        if let Mode::Query { partial, .. } = state.mode() { partial.as_ref() } else { "" };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Length(mode.len() as u16 + 1),
            Constraint::Min(1),
            Constraint::Length(ruler.len() as u16 + 1),
        ])
        .split(area);

    let mut stat = TextEditState::new(partial, partial.len() as u16, 0);
    let view = TextEditView::new();

    if let Mode::Query { .. } = state.mode() {
        view.focus(chunks[1], frame, &stat);
    }

    let emphasis = Style::default().fg(Color::Green);

    let mode = Span::styled(mode, emphasis);
    let mode = Paragraph::new(mode);

    let ruler = Span::styled(ruler, emphasis);
    let ruler = Paragraph::new(ruler).alignment(Alignment::Right);

    frame.render_widget(mode, chunks[0]);
    frame.render_widget(ruler, chunks[2]);
    frame.render_stateful_widget(view, chunks[1], &mut stat);
}

fn draw<B>(terminal: &mut Terminal<B>, state: &mut State) -> Result<(), Box<dyn Error>>
where
    B: Backend + io::Write,
{
    match state.mode() {
        Mode::Edit { .. } => write!(terminal.backend_mut(), "{}", termion::cursor::SteadyBar)?,
        Mode::Query { .. } => write!(terminal.backend_mut(), "{}", termion::cursor::BlinkingBar)?,

        _ => write!(terminal.backend_mut(), "{}", termion::cursor::SteadyBlock)?,
    }

    terminal.draw(|frame| {
        let chunks = Layout::default()
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(frame.size());

        let mut stat = TextEditState::from(&state);
        let view = TextEditView::new();

        view.focus(chunks[0], frame, &stat);

        frame.render_stateful_widget(view, chunks[0], &mut stat);
        draw_status_line(frame, chunks[1], state);
    })?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);

    let backend = TermionBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let mut state = State::default();

    loop {
        draw(&mut terminal, &mut state)?;

        if let Some(key) = io::stdin().keys().next() {
            let key = key?;

            if matches!(key, termion::event::Key::Ctrl('d')) {
                break;
            }

            event_loop(&mut state, key);
        }
    }

    Ok(())
}
