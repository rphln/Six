use std::fmt::Debug;

use crate::buffer::Buffer;
use crate::cursor::{Bounded, Cursor, Iter};

type Event = char;

/// The editor.
#[derive(Debug, Default)]
pub struct World {
    /// The shared state.
    editor: Editor,

    /// The editor mode.
    mode: Option<Box<dyn Mode>>,
}

/// The editor state.
#[derive(Debug, Default)]
pub struct Editor {
    /// The editor buffer.
    buffer: Buffer,

    /// The cursor position.
    cursor: Cursor,
}

/// An editor mode.
pub trait Mode: Debug {
    /// Advances the editor state by handling an event.
    fn advance(&self, editor: &mut Editor, event: Event) -> Box<dyn Mode>;
}

#[derive(Debug)]
pub struct Normal;

impl World {
    /// Returns a reference to the editor state.
    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    /// Advances the editor state by handling an event.
    pub fn advance(&mut self, event: Event) {
        self.mode = self.mode.take().expect("mode").advance(&mut self.editor, event).into();
    }
}

impl Editor {
    /// Returns the cursor position.
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Returns a reference to the buffer.
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Moves the cursor forward up to the specified units according to the specified unit.
    pub fn forward<'a, M: Iter<'a>>(&'a mut self, n: usize) {
        self.cursor = self.cursor.iter::<M>(&self.buffer).take(n).last().unwrap_or(self.cursor);
    }

    /// Moves the cursor forward the last position according to the specified unit.
    pub fn last<'a, M: Iter<'a>>(&'a mut self) {
        self.cursor = self.cursor.iter::<M>(&self.buffer).last().unwrap_or(self.cursor);
    }

    /// Moves the cursor backward up to the specified units according to the specified unit.
    pub fn backward<'a, M: Iter<'a>>(&'a mut self, n: usize) {
        self.cursor =
            self.cursor.iter::<M>(&self.buffer).rev().take(n).last().unwrap_or(self.cursor);
    }

    /// Moves the cursor backward the last position according to the specified unit.
    pub fn first<'a, M: Iter<'a>>(&'a mut self) {
        self.cursor = self.cursor.iter::<M>(&self.buffer).rev().last().unwrap_or(self.cursor);
    }
}

impl Mode for Normal {
    fn advance(&self, editor: &mut Editor, event: Event) -> Box<dyn Mode> {
        todo!()
    }
}

// #[derive(Derivative)]
// #[derivative(Debug, Default)]
// pub enum Mode {
//     /// The default editor mode.
//     #[derivative(Default)]
//     Normal,

//     /// The text input mode.
//     Edit,

//     /// Queries the user for a text range.
//     Select {
//         /// The fixed point of the selection.
//         anchor: Cursor,
//     },

//     /// Queries the user for a text object and applies an operation.
//     Pending {
//         /// The prompt displayed to the user.
//         prompt: &'static str,

//         /// Operator to be executed.
//         name: &'static str,
//     },

//     /// Applies an operation over a queried text object.
//     Operator {
//         /// Initial position of the text object.
//         start: Cursor,

//         /// Final position of the text object.
//         end: Cursor,

//         /// Operator to be executed.
//         name: &'static str,
//     },

//     /// Queries the user for a text input and applies an operation.
//     Querying {
//         /// The prompt displayed to the user.
//         prompt: &'static str,

//         /// The maximum length of the queried string.
//         length: Option<usize>,

//         /// The buffer of the query.
//         buffer: Buffer,

//         /// Cursor position in the buffer.
//         cursor: Cursor,

//         /// Operation to be executed.
//         name: &'static str,
//     },

//     /// Applies an operation over a queried text.
//     Query {
//         /// The buffer of the query.
//         content: Buffer,

//         /// Operation to be executed.
//         name: &'static str,
//     },
// }
