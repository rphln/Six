use std::error::Error;
use std::io;

use tui::backend::{Backend, TermionBackend};
use tui::text::Text;

use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Terminal;

use termion::input::MouseTerminal;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use six::buffer::Buffer;
use six::cursor::Cursor;
use six::state::event_loop;
use six::state::{Mode, State};

use six::event::{Event, Events};

fn draw<W, B>(state: &State<B>, terminal: &mut Terminal<W>) -> Result<(), Box<dyn Error>>
where
    W: Backend + io::Write,
    B: Buffer + std::fmt::Debug,
{
    if matches!(state.mode(), six::state::Mode::Edit) {
        write!(terminal.backend_mut(), "{}", termion::cursor::SteadyBar)?;
    } else {
        write!(terminal.backend_mut(), "{}", termion::cursor::SteadyBlock)?;
    };

    terminal.draw(|frame| {
        use tui::layout::{Constraint, Layout};

        let chunks = Layout::default()
            .margin(1)
            .constraints([Constraint::Min(1), Constraint::Length(2)].as_ref())
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

        let spans: Vec<_> = state
            .lines()
            .enumerate()
            .map(|(row, line)| {
                if let Mode::Select { anchor, .. } = state.mode() {
                    let (begin, end) = match (state.cursor(), anchor) {
                        (&p, &q) if p < q => (p, q),
                        (&p, &q) if p > q => (q, p),

                        (_, _) => return Spans::from(line),
                    };

                    let mut prefix = 0;
                    let mut infix = 0;
                    let mut suffix = 0;

                    for (col, _) in line.chars().enumerate() {
                        let point = Cursor::new(col, row);

                        if point <= begin {
                            prefix += 1;
                        } else if point > end {
                            suffix += 1;
                        } else {
                            infix += 1;
                        }
                    }

                    infix += prefix;
                    suffix += infix;

                    let suffix = line[infix..suffix].to_string();
                    let infix = line[prefix..infix].to_string();
                    let prefix = line[..prefix].to_string();

                    return Spans::from(vec![
                        Span::raw(prefix),
                        Span::styled(infix, Style::default().bg(Color::Red)),
                        Span::raw(suffix),
                    ]);
                }

                Spans::from(line)
            })
            .collect();

        let paragraph = Paragraph::new(spans).scroll((sy, sx));

        let debug = format!("{:?}", state);
        let debug = debug.as_str();
        let debug = Paragraph::new(debug)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::Black));

        frame.render_widget(paragraph, body);
        frame.render_widget(debug, chunks[1]);
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

    let mut state = State::with_buffer(String::new());
    let events = Events::new();

    loop {
        draw(&state, &mut terminal)?;

        if let Event::Input(input) = events.next()? {
            event_loop(&mut state, input).ok_or("")?;
        }
    }
}
