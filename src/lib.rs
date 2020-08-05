//! Six - A Vi-like toy text editor.

pub mod buffer;
pub mod cursor;
pub mod state;
pub mod ui;

pub use buffer::Buf;
pub use cursor::Cursor;

#[macro_use]
extern crate derivative;
