use std::error::Error;
use std::io;

use tui::backend::{Backend, TermionBackend};
use tui::text::Text;

use tui::style::{Color, Modifier, Style};
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
            .constraints(
                [Constraint::Min(1), Constraint::Length(1), Constraint::Length(2)].as_ref(),
            )
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
                    let begin = *state.cursor().min(anchor);
                    let end = *state.cursor().max(anchor);

                    let mut prefix: usize = 0;
                    let mut infix: usize = 0;
                    let mut suffix: usize = 0;

                    for (col, _) in line.chars().enumerate() {
                        let point = Cursor::new(col, row);

                        if point < begin {
                            prefix += 1;
                        } else if point > end {
                            suffix += 1;
                        } else {
                            infix += 1;
                        }
                    }

                    infix += prefix;
                    suffix += infix;

                    let default = Style::default();
                    let colored = default.add_modifier(Modifier::UNDERLINED);

                    return Spans::from(vec![
                        Span::raw(line[..prefix].to_string()),
                        Span::styled(line[prefix..infix].to_string(), colored),
                        Span::raw(line[infix..suffix].to_string()),
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

        let mode = match state.mode().clone() {
            Mode::Normal { .. } => "Normal".to_string(),
            Mode::Edit { .. } => "Edit".to_string(),
            Mode::Select { .. } => "Select".to_string(),
            Mode::Query { prompt, .. } => prompt.unwrap_or("Query".to_string()),
            Mode::Operator { prompt, .. } => prompt.unwrap_or("Operator".to_string()),
        };

        let partial = if let Mode::Query { partial, .. } = state.mode() {
            let partial = format!(" {}", partial);
            let x = (mode.len() + partial.len()) as u16;
            frame.set_cursor(chunks[1].x + x, chunks[1].y);
            partial
        } else {
            frame.set_cursor(cx, cy);
            String::default()
        };

        let modeline = Spans::from(vec![
            Span::styled(mode, Style::default().add_modifier(Modifier::BOLD).fg(Color::Green)),
            Span::raw(partial),
        ]);

        let modeline = Paragraph::new(modeline);

        frame.render_widget(paragraph, chunks[0]);
        frame.render_widget(modeline, chunks[1]);
        frame.render_widget(debug, chunks[2]);
    })?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);

    let backend = TermionBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let mut state = State::<String>::default();
    let events = Events::new();

    loop {
        draw(&state, &mut terminal)?;

        if let Event::Input(input) = events.next()? {
            if matches!(input, termion::event::Key::Ctrl('d')) {
                break;
            }

            event_loop(&mut state, input);
        }
    }

    Ok(())
}
