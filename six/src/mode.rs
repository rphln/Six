use crate::cursor::{Bounded, Codepoint, Cursor, Head, Line, Tail};
use crate::state::State;

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
        buffer: State,

        /// The maximum length of the input.
        length: Option<usize>,

        /// Function to be called after the input is submitted.
        #[derivative(Debug = "ignore")]
        and_then: Box<dyn for<'a> Callback<&'a str>>,
    },
}

pub enum Advance {
    Continue(Mode),
    Stop(Mode),
}

// TODO: Replace with a trait alias once it stabilizes.
pub trait Callback<Arg>: FnOnce(&mut State, Arg) -> Advance + Send + Sync + 'static {}

impl<Arg, F> Callback<Arg> for F where F: FnOnce(&mut State, Arg) -> Advance + Send + Sync + 'static {}

/// An operation.
#[derive(Clone, Copy, Debug)]
pub enum Operation {
    /// Return to the `Normal` mode.
    Escape,

    /// Enters the `Insert` mode.
    Insert,

    /// Moves the cursor to the left within a line.
    Left,

    /// Moves the cursor to the right within a line.
    Right,

    /// Moves the cursor upwards.
    Up,

    /// Moves the cursor downwards.
    Down,

    /// Moves the cursor backwards.
    Backward,

    /// Moves the cursor forwards.
    Forward,

    /// Moves the cursor to the beginning of the line.
    Bol,

    /// Moves the cursor to the end of the line.
    Eol,

    /// Moves the cursor to the next word head.
    ///
    /// If `reverse` is set, moves to the previous instead.
    Head { reverse: bool },

    /// Moves the cursor to the next word tail.
    ///
    /// If `reverse` is set, moves to the previous instead.
    Tail { reverse: bool },

    /// Deletes the character at the cursor.
    Delete,

    /// Surrounds a region.
    Surround,

    /// Executes a Lua expression.
    Eval,

    /// Inserts a character at the cursor position and advances it.
    Input(char),
}

bitflags! {
    pub struct Keymap: u8 {
        const NORMAL   = 1 << 0;
        const QUERY    = 1 << 1;
        const INSERT   = 1 << 2;
        const SELECT   = 1 << 3;
        const OPERATOR = 1 << 4;
    }
}
impl Mode {
    /// Halts the current macro-operation and returns to the `Normal` mode.
    #[must_use]
    pub fn abort() -> Advance {
        Advance::Stop(Mode::Normal)
    }

    /// Enters the `Normal` mode.
    #[must_use]
    pub fn escape() -> Advance {
        Advance::Continue(Mode::Normal)
    }

    /// Enters the `Insert` mode.
    #[must_use]
    pub fn to_insert() -> Advance {
        Advance::Continue(Mode::Insert)
    }

    /// Enters the `Operator` mode.
    pub fn to_operator(name: &'static str, and_then: impl Callback<(Cursor, Cursor)>) -> Advance {
        Advance::Continue(Mode::Operator { name, and_then: Box::new(and_then) })
    }

    /// Enters the `Query` mode.
    #[must_use]
    pub fn to_query(
        name: &'static str,
        length: Option<usize>,
        and_then: impl for<'r> Callback<&'r str>,
    ) -> Advance {
        Advance::Continue(Mode::Query {
            name,
            length,
            buffer: State::default(),
            and_then: Box::new(and_then),
        })
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

    /// Returns the active key map.
    #[must_use]
    pub fn keymap(&self) -> Keymap {
        match self {
            Mode::Normal { .. } => Keymap::NORMAL,
            Mode::Insert { .. } => Keymap::INSERT,
            Mode::Select { .. } => Keymap::SELECT,
            Mode::Operator { .. } => Keymap::OPERATOR,
            Mode::Query { .. } => Keymap::QUERY,
        }
    }

    /// Advances the state state by handling an event.
    pub fn advance(self, state: &mut State, event: Operation) -> Advance {
        use Mode::{Normal, Operator, Query};

        match (self, event) {
            (_, Operation::Escape) => Mode::escape(),
            (_, Operation::Insert) => Mode::to_insert(),

            (Query { mut buffer, name, length, and_then }, Operation::Input(ch)) => {
                buffer.append(ch);

                if ch == '\n' || length.map_or(false, |len| buffer.buffer().len() == len) {
                    and_then(state, buffer.buffer())
                } else {
                    Advance::Continue(Query { buffer, name, length, and_then })
                }
            },

            (mode, Operation::Input(ch)) => {
                state.append(ch);
                Advance::Continue(mode)
            },

            (mode, Operation::Backward) => {
                if let Some(end) = state.backward::<Codepoint>() {
                    if let Operator { and_then, .. } = mode {
                        and_then(state, (state.cursor(), end))
                    } else {
                        Advance::Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            },

            (mode, Operation::Left) => {
                if let Some(end) = state.backward::<Bounded>() {
                    if let Operator { and_then, .. } = mode {
                        and_then(state, (state.cursor(), end))
                    } else {
                        Advance::Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            },

            (mode, Operation::Up) => {
                if let Some(end) = state.backward::<Line>() {
                    if let Operator { and_then, .. } = mode {
                        and_then(state, (state.cursor(), end))
                    } else {
                        Advance::Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            },

            (mode, Operation::Head { reverse: true }) => {
                if let Some(end) = state.backward::<Head>() {
                    if let Operator { and_then, .. } = mode {
                        and_then(state, (state.cursor(), end))
                    } else {
                        Advance::Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            },

            (mode, Operation::Tail { reverse: true }) => {
                if let Some(end) = state.backward::<Tail>() {
                    if let Operator { and_then, .. } = mode {
                        and_then(state, (state.cursor(), end))
                    } else {
                        Advance::Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            },

            (mode, Operation::Forward) => {
                if let Some(start) = state.forward::<Codepoint>() {
                    if let Operator { and_then, .. } = mode {
                        and_then(state, (start, state.cursor()))
                    } else {
                        Advance::Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            },

            (mode, Operation::Right) => {
                if let Some(start) = state.forward::<Bounded>() {
                    if let Operator { and_then, .. } = mode {
                        and_then(state, (start, state.cursor()))
                    } else {
                        Advance::Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            },

            (mode, Operation::Down) => {
                if let Some(start) = state.forward::<Line>() {
                    if let Operator { and_then, .. } = mode {
                        and_then(state, (start, state.cursor()))
                    } else {
                        Advance::Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            },

            (mode, Operation::Head { reverse: false }) => {
                if let Some(start) = state.forward::<Head>() {
                    if let Operator { and_then, .. } = mode {
                        and_then(state, (start, state.cursor()))
                    } else {
                        Advance::Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            },

            (mode, Operation::Tail { reverse: false }) => {
                if let Some(start) = state.forward::<Tail>() {
                    if let Operator { and_then, .. } = mode {
                        and_then(state, (start, state.cursor()))
                    } else {
                        Advance::Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            },

            (Normal, Operation::Delete) => {
                let delete = |state: &mut State, (start, end): (Cursor, Cursor)| {
                    state.edit("", start..=end);
                    state.set_cursor(start);

                    Mode::escape()
                };

                Mode::to_operator("Delete", delete)
            },

            (Normal, Operation::Eval) => {
                let eval = |state: &mut State, program: &str| {
                    eprintln!("Eval: {}", program);
                    // context .lua .context(|ctx| {
                    //         ctx.scope(|scope| {
                    //             ctx.globals()
                    //                 .set("state", scope.create_nonstatic_userdata(state)?)?;
                    //             ctx.load(program).eval()
                    //         })
                    //     })

                    Mode::escape()
                };

                Mode::to_query("Eval", None, eval)
            },

            (Normal, Operation::Surround) => {
                let surround = |_: &mut State, (start, end): (Cursor, Cursor)| {
                    let surround = move |state: &mut State, sandwich: &str| {
                        let mut chars = sandwich.chars();

                        let prefix = chars.next().expect("prefix");
                        let suffix = chars.next().expect("suffix");

                        state.insert(suffix, end);
                        state.insert(prefix, start);

                        Mode::escape()
                    };

                    Mode::to_query("Surround", Some(2), surround)
                };

                Mode::to_operator("Surround", surround)
            },

            (..) => Mode::abort(),
        }
    }
}
