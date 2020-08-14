//! Six - A Vi-like toy text editor.

#![deny(clippy::all, clippy::pedantic)]

#[macro_use]
extern crate derivative;

pub mod buffer;
pub mod cursor;
pub mod state;

pub use buffer::Buffer;
pub use cursor::Cursor;
pub use state::{Editor, Event, Mode, State};

pub use rlua::Lua;
