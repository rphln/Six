//! Six - A Vi-like toy text editor.

#![deny(clippy::all, clippy::pedantic)]

#[macro_use]
extern crate derivative;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate derive_new;

pub mod cursor;
pub mod event;
pub mod mode;
pub mod state;

pub use cursor::Cursor;
pub use event::{Event, Key};
pub use mode::Mode;
pub use state::{Buffer, Editor};

pub use rlua::Lua;
