use crate::event::{Event, Key, Modifiers};
use crate::mode::Mode;
use crate::state::Context;

use crate::mode::normal::Normal;

/// The text insertion mode.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Insert;

impl Insert {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl Mode for Insert {
    fn name(&self) -> &str {
        "Insert"
    }

    fn advance(self: Box<Self>, context: &mut Context, event: Event) -> Box<dyn Mode> {
        unimplemented!()
        // match event {
        //     Event::Key(Key::Esc, _) => Normal::new(),

        //     Event::Key(Key::Char(ch), Modifiers::NONE) => {
        //         context.buffer.append(ch);
        //         self
        //     },

        //     Event::Key(Key::Left, Modifiers::NONE) => {
        //         context.buffer.backward::<Codepoint>();
        //         self
        //     },

        //     Event::Key(Key::Up, Modifiers::NONE) => {
        //         context.buffer.backward::<Line>();
        //         self
        //     },

        //     Event::Key(Key::Left, Modifiers::CTRL) => {
        //         context.buffer.backward::<Head>();
        //         self
        //     },

        //     Event::Key(Key::Right, Modifiers::NONE) => {
        //         context.buffer.forward::<Codepoint>();
        //         self
        //     },

        //     Event::Key(Key::Down, Modifiers::NONE) => {
        //         context.buffer.forward::<Line>();
        //         self
        //     },

        //     Event::Key(Key::Right, Modifiers::CTRL) => {
        //         context.buffer.forward::<Head>();
        //         self
        //     },

        //     _ => self,
        // }
    }
}
