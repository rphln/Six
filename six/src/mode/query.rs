use crate::buffer::Buffer;
use crate::cursor::{ Cursor};
use crate::event::{Event, Key, Modifiers};
use crate::mode::{Mode, Normal};
use crate::state::Context;

/// Queries the user for a text input and applies an operation.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Query<Callback>
where
    Callback: 'static + Send + Sync + FnOnce(&mut Context, &str) -> Box<dyn Mode>,
{
    /// The operation name.
    name: &'static str,

    /// The buffer of the query.
    buffer: Buffer,

    /// The maximum length of the input.
    length: Option<usize>,

    /// Function to be called after the input is submitted.
    #[derivative(Debug = "ignore")]
    and_then: Callback,
}

impl<Callback> Query<Callback>
where
    Callback: 'static + Send + Sync + FnOnce(&mut Context, &str) -> Box<dyn Mode>,
{
    pub fn new(name: &'static str, length: Option<usize>, and_then: Callback) -> Box<Self> {
        Box::new(Self { name, length, and_then, buffer: Buffer::default() })
    }
}

impl<Callback> Mode for Query<Callback>
where
    Callback: Send + Sync + FnOnce(&mut Context, &str) -> Box<dyn Mode>,
{
    fn name(&self) -> &str {
        self.name
    }

    fn advance(mut self: Box<Self>, context: &mut Context, event: Event) -> Box<dyn Mode> {
        unimplemented!()

        // match event {
        //     Event::Key(Key::Char(ch), Modifiers::NONE) => {
        //         self.buffer.append(ch);

        //         if ch == '\n' || self.length.map_or(false, |len| self.buffer.len() == len) {
        //             (self.and_then)(context, self.buffer.as_str())
        //         } else {
        //             self
        //         }
        //     },

        //     Event::Key(Key::Delete, Modifiers::NONE) => {
        //         let end = self
        //             .buffer
        //             .cursor()
        //             .forward::<Codepoint>(self.buffer.as_str())
        //             .unwrap_or_else(|| Cursor::eof(self.buffer.as_str()));
        //         self.buffer.edit("", self.buffer.cursor()..end);

        //         self
        //     },

        //     Event::Key(Key::Left, Modifiers::NONE) => {
        //         if self.buffer.backward::<Codepoint>().is_some() {
        //             self
        //         } else {
        //             Normal::new()
        //         }
        //     },

        //     _ => Normal::new(),
        // }
    }
}
