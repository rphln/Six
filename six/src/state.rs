
use std::fmt::Debug;
use std::ops::RangeBounds;

use crate::buffer::Buffer;
use crate::cursor::{Bounded, Codepoint, Cursor, Head, Line, Tail};

bitflags! {
    pub struct Modifiers: u8 {
        const NONE  = 0;

        const CTRL  = 1 << 1;
        const SHFT = 1 << 2;
        const META  = 1 << 3;
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Key {
    Backspace,
    Delete,
    Esc,

    Left,
    Right,
    Up,
    Down,

    Char(Modifiers, char),
}

impl From<char> for Key {
    fn from(ch: char) -> Key {
        Key::Char(Modifiers::NONE, ch)
    }
}

impl ToString for Key {
    fn to_string(&self) -> String {
        match self {
            Self::Backspace => "<Back>".to_owned(),
            Self::Delete => "<Del>".to_owned(),
            Self::Esc => "<Esc>".to_owned(),

            Self::Left => "<Left>".to_owned(),
            Self::Right => "<Right>".to_owned(),
            Self::Up => "<Up>".to_owned(),
            Self::Down => "<Down>".to_owned(),

            Self::Char(Modifiers::CTRL, ch) => format!("<C-{}>", ch),
            Self::Char(Modifiers::META, ch) => format!("<M-{}>", ch),
            Self::Char(Modifiers::SHFT, ch) => format!("<S-{}>", ch),

            Self::Char(Modifiers::NONE, ch) => format!("{}", ch),

            _ => "\u{fffd}".to_owned(),
        }
    }
}

/// An event.
#[derive(Copy, Clone, Debug)]
pub enum Event {
    Escape,
    Insert,

    Left,
    Right,
    Up,
    Down,

    Backward,
    Forward,

    Bol,
    Eol,

    Head { reverse: bool },
    Tail { reverse: bool },

    Delete,
    Surround,

    Input(char),
}

// TODO: Replace with a trait alias once it stabilizes.
pub trait Callback<Arg>:
    FnOnce(&mut State, Arg) -> Result<Mode, Mode> + Send + Sync + 'static
{
}

impl<Arg, F> Callback<Arg> for F where
    F: FnOnce(&mut State, Arg) -> Result<Mode, Mode> + Send + Sync + 'static
{
}

/// An modal editor.
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct Editor {
    /// The state shared between modes.
    state: State,

    /// The current mode.
    mode: Mode,

    /// Input queue.
    queue: Vec<Key>,
}

/// The state state.
#[derive(Debug, Default)]
pub struct State {
    /// The state buffer.
    buffer: Buffer,

    /// The cursor position.
    cursor: Cursor,
}

bitflags! {
    pub struct Keymap: u8 {
        const NORMAL   = 1 << 0;
        const QUERY    = 1 << 1;

        const INSERT   = 1 << 2;
        const SELECT   = 1 << 3;
        const OPERATOR = 1 << 4;

        const INPUT = Self::INSERT.bits | Self::QUERY.bits;
    }
}

/// An state mode.
#[derive(Derivative)]
#[derivative(Default, Debug)]
pub enum Mode {
    /// The default editor mode.
    #[derivative(Default)]
    Normal,

    /// The text insertion mode.
    Insert,

    /// Queries the user for a text range.
    Select {
        /// The fixed point of the selection.
        anchor: Cursor,
    },

    /// Queries the user for a text object and applies an operation.
    Operator {
        /// The operator name.
        name: &'static str,

        /// Operator to be executed.
        #[derivative(Debug = "ignore")]
        and_then: Box<dyn Callback<(Cursor, Cursor)>>,
    },

    /// Queries the user for a text input and applies an operation.
    Query {
        /// The operation name.
        name: &'static str,

        /// The buffer of the query.
        buffer: Buffer,

        /// Cursor position in the buffer.
        cursor: Cursor,

        /// Whether to finish the query.
        #[derivative(Debug = "ignore")]
        until: fn(&str, Event) -> bool,

        /// Function to be called after the input is submitted.
        #[derivative(Debug = "ignore")]
        and_then: Box<dyn for<'a> Callback<&'a str>>,
    },
}

impl Editor {
    /// Returns a reference to the state inner state.
    #[must_use]
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Returns a reference to the state mode.
    #[must_use]
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Returns a slice containing the key queue.
    #[must_use]
    pub fn queue(&self) -> &[Key] {
        self.queue.as_slice()
    }

    /// Returns a flag containing the active keymap.
    #[must_use]
    pub fn keymap(&self) -> Keymap {
        match self.mode() {
            Mode::Normal { .. } => Keymap::NORMAL,
            Mode::Insert { .. } => Keymap::INSERT,
            Mode::Select { .. } => Keymap::SELECT,
            Mode::Operator { .. } => Keymap::OPERATOR,
            Mode::Query { .. } => Keymap::QUERY,
        }
    }

    pub fn handle_key(&mut self, key: Option<Key>) {
        if let Some(key) = key {
            self.queue.push(key);
        }

        macro_rules! map { { $keymap:expr, $($key:expr),+ } => { ($keymap, vec![$($key.into()),+]) } }

        macro_rules! insmap { { $($key:expr),+ } => { map!(Keymap::INSERT,  $($key),+) } }
        macro_rules! allmap { { $($key:expr),+ } => { map!(Keymap::all(),   $($key),+) } }
        macro_rules! ninmap { { $($key:expr),+ } => { map!(!Keymap::INSERT, $($key),+) } }
        macro_rules! niqmap { { $($key:expr),+ } => { map!(!Keymap::INPUT,  $($key),+) } }

        let keymap = hashmap! {
            allmap! { Key::Char(Modifiers::CTRL, 'a') } => vec![Event::Bol],
            allmap! { Key::Char(Modifiers::CTRL, 'e') } => vec![Event::Eol],

            allmap! { Key::Delete }                     => vec![Event::Delete],

            allmap! { Key::Down }                       => vec![Event::Down],
            allmap! { Key::Left }                       => vec![Event::Left],
            allmap! { Key::Right }                      => vec![Event::Right],
            allmap! { Key::Up }                         => vec![Event::Up],

            insmap! { Key::Backspace }                  => vec![Event::Delete, Event::Left],
            insmap! { Key::Delete }                     => vec![Event::Delete],

            insmap! { Key::Esc }                        => vec![Event::Escape, Event::Left],
            ninmap! { Key::Esc }                        => vec![Event::Escape],

            niqmap! { 'a' }                             => vec![Event::Insert, Event::Right],
            niqmap! { 'i' }                             => vec![Event::Insert],

            niqmap! { 'h' }                             => vec![Event::Left],
            niqmap! { 'j' }                             => vec![Event::Down],
            niqmap! { 'k' }                             => vec![Event::Up],
            niqmap! { 'l' }                             => vec![Event::Right],

            niqmap! { 'w' }                             => vec![Event::Head { reverse: false }],
            niqmap! { 'b' }                             => vec![Event::Head { reverse: true }],
            niqmap! { 'e' }                             => vec![Event::Tail { reverse: false }],
            niqmap! { 'g', 'e' }                        => vec![Event::Tail { reverse: true }],
        };

        let matches: Vec<_> = keymap
            .keys()
            .filter(|(map, sequence)| {
                map.contains(self.keymap())
                    && if key.is_none() {
                        sequence == &self.queue
                    } else {
                        sequence.starts_with(self.queue.as_slice())
                    }
            })
            .collect();

        if matches.len() > 1 {
            return;
        }

        if let [matched] = matches.as_slice() {
            let (_, sequence) = matched;

            if sequence == &self.queue {
                self.queue.clear();
                self.advance(keymap[matched].as_slice());
            }

            return;
        }

        match (self.mode(), self.queue.as_slice()) {
            (Mode::Insert { .. }, &[Key::Char(Modifiers::NONE, ch)]) => {
                self.advance(&[Event::Input(ch), Event::Forward]);
            },

            (Mode::Query { .. }, &[Key::Char(Modifiers::NONE, ch)]) => {
                self.advance(&[Event::Input(ch), Event::Forward]);
            },

            _ => {},
        };

        self.queue.clear();
    }

    /// Advances the state state by handling events.
    pub fn advance(&mut self, events: &[Event]) {
        self.mode = events
            .iter()
            .try_fold(std::mem::take(&mut self.mode), |mode, &event| {
                mode.advance(&mut self.state, event)
            })
            .unwrap_or_else(|mode| mode);
    }
}

impl State {
    /// Returns the cursor position.
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Returns a reference to the buffer.
    #[must_use]
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }

    pub fn insert(&mut self, at: Cursor, ch: char) {
        self.buffer.insert(at, ch)
    }

    pub fn replace_range(&mut self, range: impl RangeBounds<usize>, text: &str) {
        self.buffer.replace_range(range, text)
    }
}

impl Mode {
    #[must_use]
    pub fn escape() -> Mode {
        Mode::Normal
    }

    #[must_use]
    pub fn insert() -> Mode {
        Mode::Insert
    }

    pub fn operator(name: &'static str, and_then: impl Callback<(Cursor, Cursor)>) -> Mode {
        Mode::Operator { name, and_then: Box::new(and_then) }
    }

    #[must_use]
    pub fn query(
        name: &'static str,
        until: fn(&str, Event) -> bool,
        and_then: impl for<'r> Callback<&'r str>,
    ) -> Mode {
        Mode::Query {
            name,
            until,

            cursor: Cursor::default(),
            buffer: Buffer::default(),

            and_then: Box::new(and_then),
        }
    }

    /// Returns an user-friendly name for the mode.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Mode::Normal { .. } => "Normal",
            Mode::Insert { .. } => "Insert",
            Mode::Select { .. } => "Select",

            Mode::Query { name, .. } => name,
            Mode::Operator { name, .. } => name,
        }
    }

    /// Advances the state state by handling an event.
    pub fn advance(self, state: &mut State, event: Event) -> Result<Mode, Mode> {
        use Mode::{Normal, Operator, Query};

        match (self, event) {
            (_, Event::Escape) => Ok(Mode::escape()),
            (_, Event::Insert) => Ok(Mode::insert()),

            (Query { mut buffer, until, name, mut cursor, and_then }, event) => {
                match event {
                    Event::Input(ch) => {
                        buffer.insert(cursor, ch);
                        cursor = cursor
                            .iter::<Codepoint>(&buffer)
                            .next()
                            .unwrap_or_else(|| Cursor::eof(&buffer));
                    },

                    _ => (),
                };

                if until(buffer.as_str(), event) {
                    and_then(state, buffer.as_str())
                } else {
                    Ok(Query { buffer, until, name, cursor, and_then })
                }
            },

            (mode, Event::Forward) => {
                if let Some(cursor) = state.cursor.iter::<Codepoint>(state.buffer()).next() {
                    state.set_cursor(cursor);

                    Ok(mode)
                } else {
                    Err(mode)
                }
            },

            (mode, Event::Backward) => {
                state.set_cursor(
                    state
                        .cursor()
                        .iter::<Codepoint>(state.buffer())
                        .next_back()
                        .unwrap_or(state.cursor()),
                );

                Ok(mode)
            },

            (mode, Event::Left) => {
                state.set_cursor(
                    state
                        .cursor()
                        .iter::<Bounded>(state.buffer())
                        .next_back()
                        .unwrap_or(state.cursor()),
                );

                Ok(mode)
            },

            (mode, Event::Right) => {
                state.set_cursor(
                    state.cursor().iter::<Bounded>(state.buffer()).next().unwrap_or(state.cursor()),
                );
                Ok(mode)
            },

            (mode, Event::Down) => {
                state.set_cursor(
                    state.cursor().iter::<Line>(state.buffer()).next().unwrap_or(state.cursor()),
                );
                Ok(mode)
            },

            (mode, Event::Up) => {
                state.set_cursor(
                    state
                        .cursor()
                        .iter::<Line>(state.buffer())
                        .next_back()
                        .unwrap_or(state.cursor()),
                );
                Ok(mode)
            },

            (Normal, Event::Delete) => {
                Ok(Mode::operator("Delete", |state: &mut State, range: (Cursor, Cursor)| {
                    state.replace_range(range.0.offset()..range.1.offset(), "");
                    Ok(Mode::escape())
                }))
            },

            (mode, Event::Bol) => {
                state.set_cursor(
                    state.cursor().iter::<Bounded>(state.buffer()).rev().last().expect("first"),
                );
                Ok(mode)
            },

            (mode, Event::Eol) => {
                state.set_cursor(
                    state.cursor().iter::<Bounded>(state.buffer()).last().expect("last"),
                );
                Ok(mode)
            },

            (mode, Event::Input(ch)) => {
                state.insert(state.cursor(), ch);
                Ok(mode)
            },

            (Normal, Event::Surround) => {
                let surround = |_: &mut State, (start, end): (Cursor, Cursor)| {
                    let surround = move |state: &mut State, sandwich: &str| {
                        let mut chars = sandwich.chars();

                        let prefix = chars.next().expect("prefix");
                        let suffix = chars.next().expect("suffix");

                        state.insert(end, suffix);
                        state.insert(start, prefix);

                        Ok(Mode::escape())
                    };

                    Ok(Mode::query("Surround", |buf: &str, _: Event| buf.len() == 2, surround))
                };

                Ok(Mode::operator("Surround", surround))
            },

            (mode, Event::Head { reverse: true }) => {
                let end = state.cursor();
                state.set_cursor(
                    state
                        .cursor()
                        .iter::<Head>(state.buffer())
                        .next_back()
                        .unwrap_or_else(|| state.cursor()),
                );

                if let Operator { and_then, .. } = mode {
                    and_then(state, (state.cursor(), end))
                } else {
                    Ok(mode)
                }
            },

            (mode, Event::Head { reverse: false }) => {
                let start = state.cursor();
                state.set_cursor(
                    state
                        .cursor()
                        .iter::<Head>(state.buffer())
                        .next()
                        .unwrap_or_else(|| Cursor::eof(state.buffer())),
                );

                if let Operator { and_then, .. } = mode {
                    and_then(state, (start, state.cursor()))
                } else {
                    Ok(mode)
                }
            },

            (mode, Event::Tail { reverse: false }) => {
                if let Some(end) = state.cursor().iter::<Tail>(state.buffer()).next() {
                    let start = state.cursor();
                    state.set_cursor(end);

                    if let Operator { and_then, .. } = mode {
                        and_then(state, (start, end))
                    } else {
                        Ok(mode)
                    }
                } else {
                    Err(mode)
                }
            },

            (mode, Event::Tail { reverse: true }) => {
                if let Some(start) = state.cursor().iter::<Tail>(state.buffer()).next_back() {
                    let end = state.cursor();
                    state.set_cursor(start);

                    if let Operator { and_then, .. } = mode {
                        and_then(state, (start, end))
                    } else {
                        Ok(mode)
                    }
                } else {
                    Err(mode)
                }
            },

            (mode, _) => Err(mode),
        }
    }
}
