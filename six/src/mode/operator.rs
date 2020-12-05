use std::ops::Bound;

use crate::cursor::Cursor;
use crate::event::{Event, Key, Modifiers};
use crate::mode::Mode;
use crate::state::Context;

/// Queries the user for a text object and applies an operation.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Operator<Callback>
where
    Callback:
        'static + Send + Sync + FnOnce(&mut Context, Bound<Cursor>, Bound<Cursor>) -> Box<dyn Mode>,
{
    /// The operator name.
    name: &'static str,

    /// Operator to be executed.
    #[derivative(Debug = "ignore")]
    and_then: Callback,
}

impl<Callback> Operator<Callback>
where
    Callback:
        'static + Send + Sync + FnOnce(&mut Context, Bound<Cursor>, Bound<Cursor>) -> Box<dyn Mode>,
{
    pub fn new(name: &'static str, and_then: Callback) -> Box<Self> {
        Box::new(Self { name, and_then })
    }
}

impl<Callback> Mode for Operator<Callback>
where
    Callback:
        'static + Send + Sync + FnOnce(&mut Context, Bound<Cursor>, Bound<Cursor>) -> Box<dyn Mode>,
{
    fn name(&self) -> &str {
        self.name
    }

    fn advance(self: Box<Self>, context: &mut Context, event: Event) -> Box<dyn Mode> {
        unimplemented!()
        // use crate::cursor::{Codepoint, Head, Line, Tail};
        // use Bound::{Excluded, Included, Unbounded};

        // match event {
        //     Event::Key(Key::Char('h'), Modifiers::NONE)
        //     | Event::Key(Key::Left, Modifiers::NONE) => {
        //         let end = Excluded(context.buffer.cursor());
        //         let start = context.buffer.backward::<Codepoint>().map_or(Unbounded, Included);

        //         (self.and_then)(context, start, end)
        //     },

        //     Event::Key(Key::Char('k'), Modifiers::NONE) | Event::Key(Key::Up, Modifiers::NONE) => {
        //         let end = context.buffer.cursor();
        //         if let Some(start) = context.buffer.backward::<Line>() {
        //             (self.and_then)(context, Included(start), Excluded(end))
        //         } else {
        //             self
        //         }
        //     },

        //     Event::Key(Key::Char('W'), Modifiers::NONE) => {
        //         let end = context.buffer.cursor();
        //         if let Some(start) = context.buffer.backward::<Head>() {
        //             (self.and_then)(context, Included(start), Excluded(end))
        //         } else {
        //             self
        //         }
        //     },

        //     Event::Key(Key::Char('E'), Modifiers::NONE) => {
        //         let end = context.buffer.cursor();
        //         if let Some(start) = context.buffer.backward::<Tail>() {
        //             (self.and_then)(context, Included(start), Excluded(end))
        //         } else {
        //             self
        //         }
        //     },

        //     Event::Key(Key::Char('l'), Modifiers::NONE)
        //     | Event::Key(Key::Right, Modifiers::NONE) => {
        //         let end = context.buffer.cursor();
        //         if let Some(start) = context.buffer.forward::<Codepoint>() {
        //             (self.and_then)(context, Included(start), Excluded(end))
        //         } else {
        //             self
        //         }
        //     },

        //     Event::Key(Key::Char('j'), Modifiers::NONE)
        //     | Event::Key(Key::Down, Modifiers::NONE) => {
        //         let end = context.buffer.cursor();
        //         if let Some(start) = context.buffer.forward::<Line>() {
        //             (self.and_then)(context, Included(start), Excluded(end))
        //         } else {
        //             self
        //         }
        //     },

        //     Event::Key(Key::Char('w'), Modifiers::NONE) => {
        //         let start = context.buffer.cursor();
        //         if let Some(end) = context.buffer.forward::<Head>() {
        //             (self.and_then)(context, Included(start), Excluded(end))
        //         } else {
        //             self
        //         }
        //     },

        //     Event::Key(Key::Char('e'), Modifiers::NONE) => {
        //         let start = context.buffer.cursor();
        //         if let Some(end) = context.buffer.forward::<Tail>() {
        //             (self.and_then)(context, Included(start), Included(end))
        //         } else {
        //             self
        //         }
        //     },

        //     _ => self,
        // }
    }
}
