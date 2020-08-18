//! Six - A Vi-like toy text editor.

#![deny(clippy::all, clippy::pedantic)]

#[macro_use]
extern crate derivative;

#[macro_use]
extern crate bitflags;

pub mod buffer;
pub mod cursor;
pub mod event;
pub mod mode;
pub mod state;

pub use buffer::Buffer;
pub use cursor::Cursor;
pub use event::{Event, Key, Modifiers};
pub use mode::Mode;
pub use state::Editor;
