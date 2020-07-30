//! Six - A Vi-like toy text editor.

pub mod buffer;
pub mod cursor;
pub mod event;
pub mod state;

pub use buffer::Buffer;
pub use cursor::Cursor;

#[macro_use]
extern crate derivative;
