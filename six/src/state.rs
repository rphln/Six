use rlua::Lua;

use crate::buffer::Buffer;
use crate::mode::{Mode, Normal};
use crate::Cursor;
use crate::Event;

/// An modal editor.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Editor {
    /// The current mode.
    mode: Box<dyn Mode>,

    /// The editor context.
    context: Context,

    /// The scripting engine.
    #[derivative(Debug = "ignore")]
    interpreter: Lua,
}

/// Editor context.
#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct Context {
    /// The text buffer.
    pub buffer: Buffer,
}

impl Editor {
    pub fn new() -> Self {
        Self { context: Context::default(), interpreter: Lua::default(), mode: Normal::new() }
    }

    /// Returns a reference to the text buffer.
    pub fn buffer(&self) -> &Buffer {
        &self.context.buffer
    }

    /// Returns the name of the active mode.
    pub fn mode(&self) -> &str {
        self.mode.name()
    }

    /// Returns the cursor position.
    pub fn cursor(&self) -> Cursor {
        self.context.buffer.cursor()
    }

    /// Advances the state by handling events.
    pub fn advance(&mut self, events: &[Event]) {
        self.mode =
            events.iter().fold(std::mem::replace(&mut self.mode, Normal::new()), |mode, &event| {
                mode.advance(&mut self.context, event)
            });
    }
}