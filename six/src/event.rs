use std::time::Duration;

bitflags! {
    pub struct Modifiers: u8 {
        const NONE  = 0;

        const CTRL  = 1 << 1;
        const SHFT = 1 << 2;
        const META  = 1 << 3;
    }
}

/// A key.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Key {
    /// Backspace.
    Backspace,

    /// Delete.
    Delete,

    /// Escape.
    Esc,

    /// Left arrow.
    Left,

    /// Right arrow.
    Right,

    /// Up arrow.
    Up,

    /// Down arrow.
    Down,

    /// Home key.
    Home,

    /// A character key.
    Char(char),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Event {
    /// An event periodically sent after no other event has been emitted.
    Idle(Duration),

    /// A key press.
    Key(Key, Modifiers),
}
