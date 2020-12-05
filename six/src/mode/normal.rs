use std::ops::Bound;

use crate::cursor::Cursor;
use crate::event::{Event, Key, Modifiers};
use crate::mode::{Insert, Mode, Operator, Query};
use crate::state::Context;

/// The default editor mode.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Normal;

impl Normal {
    /// Returns a new instance of this mode.
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

// fn surround(start: Bound<Cursor>, end: Bound<Cursor>) -> Box<dyn Mode> {
//     // use Bound::{Excluded, Included, Unbounded};

//     let surround = move |context: &mut Context, sandwich: &str| -> Box<dyn Mode> {
//         let mut sandwich = sandwich.chars();

//         let prefix = sandwich.next().expect("prefix");
//         let suffix = sandwich.next().expect("suffix");

//         // context.buffer.insert(suffix, end);
//         // context.buffer.insert(prefix, start);

//         // context.buffer.set_cursor(start);

//         Normal::new()
//     };

//     Query::new("Surround", Some(2), surround)
// }

impl Mode for Normal {
    fn name(&self) -> &str {
        "Normal"
    }

    fn advance(self: Box<Self>, context: &mut Context, event: Event) -> Box<dyn Mode> {
        // use crate::cursor::{Head, Line, Tail};

        match event {
            Event::Key(Key::Char('i'), Modifiers::NONE) => Insert::new(),
            Event::Key(Key::Char('a'), Modifiers::NONE) => Insert::new(),

        //     Event::Key(Key::Char('h'), Modifiers::NONE)
        //     | Event::Key(Key::Left, Modifiers::NONE) => {
        //         context.buffer.backward::<Codepoint>();
        //         self
        //     }

        //     Event::Key(Key::Char('k'), Modifiers::NONE) | Event::Key(Key::Up, Modifiers::NONE) =>
        // {         context.buffer.backward::<Line>();
        //         self
        //     }

        //     Event::Key(Key::Char('W'), Modifiers::NONE) => {
        //         context.buffer.backward::<Head>();
        //         self
        //     }

        //     Event::Key(Key::Char('E'), Modifiers::NONE) => {
        //         context.buffer.backward::<Tail>();
        //         self
        //     }

        //     Event::Key(Key::Char('l'), Modifiers::NONE)
        //     | Event::Key(Key::Right, Modifiers::NONE) => {
        //         context.buffer.forward::<Codepoint>();
        //         self
        //     }

        //     Event::Key(Key::Char('j'), Modifiers::NONE)
        //     | Event::Key(Key::Down, Modifiers::NONE) => {
        //         context.buffer.forward::<Line>();
        //         self
        //     }

        //     Event::Key(Key::Char('w'), Modifiers::NONE) => {
        //         context.buffer.forward::<Head>();
        //         self
        //     }

        //     Event::Key(Key::Char('e'), Modifiers::NONE) => {
        //         context.buffer.forward::<Tail>();
        //         self
        //     }

        //     Event::Key(Key::Char('s'), Modifiers::NONE) => {
        //         Operator::new("Surround", |_, start, end| surround(start, end))
        //     }

            _ => self,
        }
    }
}
