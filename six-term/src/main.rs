//! Six - A Vi-like toy text editor.

#![deny(clippy::all, clippy::pedantic)]

use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{cursor, execute, queue, style, terminal, Result};

use six::{Editor, Event as Ev, Key, Modifiers};

fn draw(stdout: &mut impl Write, state: &Editor) -> Result<()> {
    let (_cols, rows) = terminal::size()?;

    queue!(stdout, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;

    state.buffer().content().iter().try_for_each(|row| {
        queue!(stdout, style::Print(row.to_string()), cursor::MoveToNextLine(1))
    })?;

    queue!(stdout, style::Print(format!("{:?}", state)))?;

    queue!(stdout, cursor::MoveTo(0, rows))?;
    queue!(stdout, style::Print(state.mode()))?;

    let col = state.cursor().col() as u16;
    let row = state.cursor().row() as u16;

    queue!(stdout, cursor::MoveTo(col, row))?;

    stdout.flush()?;

    Ok(())
}

fn main() -> Result<()> {
    let mut editor = Editor::new();
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen)?;

    draw(&mut stdout, &editor)?;

    loop {
        match event::read()? {
            Event::Key(KeyEvent { code: KeyCode::Char('d'), modifiers: KeyModifiers::CONTROL }) => {
                break;
            },

            Event::Key(KeyEvent { code, modifiers }) => {
                let code = match code {
                    KeyCode::Esc => Key::Esc,

                    KeyCode::Char(ch) => Key::Char(ch),

                    KeyCode::Left => Key::Left,
                    KeyCode::Up => Key::Up,
                    KeyCode::Down => Key::Down,
                    KeyCode::Right => Key::Right,

                    KeyCode::Backspace => Key::Backspace,
                    KeyCode::Delete => Key::Delete,

                    KeyCode::Home => Key::Home,
                    KeyCode::End => Key::End,

                    KeyCode::Enter => Key::Char('\n'),

                    _ => Key::Char('\0'),
                };

                let modifiers = match modifiers {
                    KeyModifiers::NONE => Modifiers::NONE,
                    KeyModifiers::CONTROL => Modifiers::CTRL,

                    KeyModifiers::SHIFT => Modifiers::NONE,

                    _ => todo!(),
                };

                editor.advance(&[Ev::Key(code, modifiers)])
            },

            _ => continue,
        }

        draw(&mut stdout, &editor)?;
    }

    terminal::disable_raw_mode()?;
    execute!(stdout, terminal::LeaveAlternateScreen)
}
