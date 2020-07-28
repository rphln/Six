use std::error::Error;
use std::io;

use tui::backend::{Backend, TermionBackend};
use tui::text::Text;
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Terminal;

use termion::input::MouseTerminal;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use six::state::event_loop;
use six::state::State;

use six::event::{Event, Events};

fn draw<W>(state: &State, terminal: &mut Terminal<W>) -> Result<(), Box<dyn Error>>
where
    W: Backend + io::Write,
{
    if matches!(state.mode, six::state::Mode::Edit) {
        write!(terminal.backend_mut(), "{}", termion::cursor::SteadyBar)?;
    } else {
        write!(terminal.backend_mut(), "{}", termion::cursor::SteadyBlock)?;
    };

    terminal.draw(|frame| {
        use tui::layout::{Constraint, Layout};

        let chunks = Layout::default()
            .margin(1)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(frame.size());

        let body = chunks[0];

        let x = state.col() as u16;
        let y = state.row() as u16;

        let width = body.width.saturating_sub(1);
        let height = body.height.saturating_sub(1);

        let sx = x.saturating_sub(width);
        let sy = y.saturating_sub(height);

        let cx = body.x + x.min(width);
        let cy = body.y + y.min(height);

        let text = Text::from(state.buffer.as_str());
        let paragraph = Paragraph::new(text).scroll((sy, sx));

        frame.render_widget(paragraph, body);
        frame.set_cursor(cx, cy)
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
    let events = Events::new();

    loop {
        draw(&state, &mut terminal)?;

        if let Event::Input(input) = events.next()? {
            state = event_loop(state, input).ok_or("")?;
        }
    }
}
