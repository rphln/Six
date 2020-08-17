use rlua::{Lua, UserData};

use crate::cursor::{Bounded, Codepoint, Cursor, Head, Line, Tail};
use crate::state::Buffer;

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
        and_then: Box<dyn for<'r> Callback<'r, (Cursor, Cursor)>>,
    },

    /// Queries the user for a text input and applies an operation.
    Query {
        /// The operation name.
        name: &'static str,

        /// The buffer of the query.
        buffer: Buffer,

        /// The maximum length of the input.
        length: Option<usize>,

        /// Function to be called after the input is submitted.
        #[derivative(Debug = "ignore")]
        and_then: Box<dyn for<'a> Callback<'a, &'a str>>,
    },
}

/// Specifies whether to continue or halt the current macro operation.
pub enum Advance {
    Continue(Mode),
    Halt(Mode),
}

#[derive(new)]
pub struct Context<'a> {
    buffer: &'a mut Buffer,
    lua: &'a Lua,
    messages: &'a mut Vec<String>,
}

// TODO: Replace with a trait alias once it stabilizes.
pub trait Callback<'a, Arg: 'a>:
    FnOnce(Context<'a>, Arg) -> Advance + Send + Sync + 'static
{
}

impl<'a, Arg: 'a, F> Callback<'a, Arg> for F where
    F: FnOnce(Context<'a>, Arg) -> Advance + Send + Sync + 'static
{
}

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

    /// Deletes a region.
    Delete,

    /// Surrounds a region.
    Surround,

    /// Queries the user for an Lua expression and evaluates it.
    Eval,

    /// Evaluates a Lua expression.
    Command(&'static str),

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
        Advance::Halt(Mode::Normal)
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
    pub fn to_operator(
        name: &'static str,
        and_then: impl for<'r> Callback<'r, (Cursor, Cursor)>,
    ) -> Advance {
        Advance::Continue(Mode::Operator { name, and_then: Box::new(and_then) })
    }

    /// Enters the `Query` mode.
    #[must_use]
    pub fn to_query(
        name: &'static str,
        length: Option<usize>,
        and_then: impl for<'r> Callback<'r, &'r str>,
    ) -> Advance {
        Advance::Continue(Mode::Query {
            name,
            length,
            buffer: Buffer::default(),
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
    pub fn advance(self, context: Context, event: Operation) -> Advance {
        use Advance::{Continue, Halt};
        use Mode::{Normal, Operator, Query};

        match (self, event) {
            (_, Operation::Escape) => Mode::escape(),
            (_, Operation::Insert) => Mode::to_insert(),

            (Query { mut buffer, name, length, and_then }, Operation::Input(ch)) => {
                buffer.append(ch);

                if ch == '\n' || length.map_or(false, |len| buffer.as_str().len() == len) {
                    and_then(context, buffer.as_str())
                } else {
                    Continue(Query { buffer, name, length, and_then })
                }
            }

            (Query { mut buffer, name, length, and_then }, Operation::Delete) => {
                let end = buffer
                    .cursor()
                    .forward::<Codepoint>(buffer.as_str())
                    .unwrap_or_else(|| Cursor::eof(buffer.as_str()));
                buffer.edit("", buffer.cursor()..end);

                Continue(Query { name, buffer, length, and_then })
            }

            (Query { mut buffer, name, length, and_then }, Operation::Left) => {
                if buffer.backward::<Codepoint>().is_some() {
                    Continue(Query { buffer, name, length, and_then })
                } else {
                    Halt(Query { buffer, name, length, and_then })
                }
            }

            (mode, Operation::Input(ch)) => {
                context.buffer.append(ch);
                Continue(mode)
            }

            (mode, Operation::Backward) => {
                if let Some(end) = context.buffer.backward::<Codepoint>() {
                    if let Operator { and_then, .. } = mode {
                        let start = context.buffer.cursor();
                        and_then(context, (start, end))
                    } else {
                        Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            }

            (mode, Operation::Left) => {
                if let Some(end) = context.buffer.backward::<Bounded>() {
                    if let Operator { and_then, .. } = mode {
                        let start = context.buffer.cursor();
                        and_then(context, (start, end))
                    } else {
                        Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            }

            (mode, Operation::Up) => {
                if let Some(end) = context.buffer.backward::<Line>() {
                    if let Operator { and_then, .. } = mode {
                        let start = context.buffer.cursor();
                        and_then(context, (start, end))
                    } else {
                        Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            }

            (mode, Operation::Head { reverse: true }) => {
                if let Some(end) = context.buffer.backward::<Head>() {
                    if let Operator { and_then, .. } = mode {
                        let start = context.buffer.cursor();
                        and_then(context, (start, end))
                    } else {
                        Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            }

            (mode, Operation::Tail { reverse: true }) => {
                if let Some(end) = context.buffer.backward::<Tail>() {
                    if let Operator { and_then, .. } = mode {
                        let start = context.buffer.cursor();
                        and_then(context, (start, end))
                    } else {
                        Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            }

            (mode, Operation::Forward) => {
                if let Some(start) = context.buffer.forward::<Codepoint>() {
                    if let Operator { and_then, .. } = mode {
                        let end = context.buffer.cursor();
                        and_then(context, (start, end))
                    } else {
                        Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            }

            (mode, Operation::Right) => {
                if let Some(start) = context.buffer.forward::<Bounded>() {
                    if let Operator { and_then, .. } = mode {
                        let end = context.buffer.cursor();
                        and_then(context, (start, end))
                    } else {
                        Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            }

            (mode, Operation::Down) => {
                if let Some(start) = context.buffer.forward::<Line>() {
                    if let Operator { and_then, .. } = mode {
                        let end = context.buffer.cursor();
                        and_then(context, (start, end))
                    } else {
                        Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            }

            (mode, Operation::Head { reverse: false }) => {
                if let Some(start) = context.buffer.forward::<Head>() {
                    if let Operator { and_then, .. } = mode {
                        let end = context.buffer.cursor();
                        and_then(context, (start, end))
                    } else {
                        Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            }

            (mode, Operation::Tail { reverse: false }) => {
                if let Some(start) = context.buffer.forward::<Tail>() {
                    if let Operator { and_then, .. } = mode {
                        let end = context.buffer.cursor();
                        and_then(context, (start, end))
                    } else {
                        Continue(mode)
                    }
                } else {
                    Mode::abort()
                }
            }

            (mode, Operation::Delete) => {
                let delete = move |context: Context, (start, end): (Cursor, Cursor)| {
                    context.buffer.edit("", start..end);
                    context.buffer.set_cursor(start);

                    Continue(mode)
                };

                Mode::to_operator("Delete", delete)
            }

            (mode, Operation::Eval) => {
                let eval = |context: Context, program: &str| {
                    let buffer = context.buffer;
                    let messages = context.messages;

                    context
                        .lua
                        .context(|ctx| {
                            ctx.scope(|scope| {
                                let globals = ctx.globals();

                                let class = ctx.create_table()?;

                                globals.set("buffer", scope.create_nonstatic_userdata(buffer)?)?;
                                messages.push(ctx.load(program).eval()?);

                                Ok(Continue(mode))
                            })
                        })
                        .unwrap_or_else(|_: Box<dyn std::error::Error>| Mode::abort())
                };

                Mode::to_query("Eval", None, eval)
            }

            (Normal, Operation::Surround) => {
                let surround = |_: Context, (start, end): (Cursor, Cursor)| {
                    let surround = move |context: Context, sandwich: &str| {
                        let mut chars = sandwich.chars();

                        let prefix = chars.next().expect("prefix");
                        let suffix = chars.next().expect("suffix");

                        context.buffer.insert(suffix, end);
                        context.buffer.insert(prefix, start);

                        Mode::escape()
                    };

                    Mode::to_query("Surround", Some(2), surround)
                };

                Mode::to_operator("Surround", surround)
            }

            (mode, ..) => Halt(mode),
        }
    }
}
