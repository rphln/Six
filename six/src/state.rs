use std::collections::HashMap;

use rlua::Lua;

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::event::Key;
use crate::mode::{Advance, Context, Mode, Operation};

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum Keymap {
    Normal,
    Query,
    Insert,
    Select,
    Operator,
}

/// An modal editor.
#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct Editor {
    /// The active mode.
    mode: Mode,

    /// The text buffer.
    buffer: Buffer,

    /// The editor message log.
    messages: Vec<String>,

    /// The editor key map.
    #[derivative(Default(value = "default_keymap()"))]
    #[derivative(Debug = "ignore")]
    keymap: HashMap<(Keymap, Key), Vec<Operation>>,

    /// The scripting engine.
    #[derivative(Debug = "ignore")]
    interpreter: Lua,
}

impl Editor {
    /// Returns a reference to the buffer mode.
    #[must_use]
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Returns the cursor position.
    #[inline]
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.buffer.cursor()
    }

    /// Returns a reference to the content contents.
    #[inline]
    #[must_use]
    pub fn content(&self) -> &str {
        self.buffer.as_str()
    }

    /// Returns the active key map.
    #[must_use]
    pub fn keymap(&self) -> Keymap {
        match self.mode() {
            Mode::Normal { .. } => Keymap::Normal,
            Mode::Insert { .. } => Keymap::Insert,
            Mode::Select { .. } => Keymap::Select,
            Mode::Operator { .. } => Keymap::Operator,
            Mode::Query { .. } => Keymap::Query,
        }
    }

    // TODO: Replace `Key` with `Operation`.
    pub fn handle_key(&mut self, input: Key) {
        let operations = if let Some(operations) = self.keymap.get(&(self.keymap(), input)) {
            operations.clone()
        } else {
            match (self.keymap(), input) {
                (Keymap::Insert, Key::Char(ch)) => vec![Operation::Input(ch)],
                (Keymap::Query, Key::Char(ch)) => vec![Operation::Input(ch)],

                (..) => vec![],
            }
        };

        self.advance(operations.as_slice())
    }

    /// Advances the buffer buffer by handling events.
    pub fn advance(&mut self, operations: &[Operation]) {
        let mut mode = std::mem::take(&mut self.mode);

        for &operation in operations {
            let context = Context::new(&mut self.buffer, &mut self.messages, &self.interpreter);

            match mode.advance(context, operation) {
                Advance::Continue(next) => mode = next,
                Advance::Break(next) => {
                    mode = next;
                    break;
                },
            }
        }

        self.mode = mode;
    }
}

/// Returns the default keymap.
#[must_use]
pub fn default_keymap() -> HashMap<(Keymap, Key), Vec<Operation>> {
    // TODO: Allow matching on multiple keys. Select the first longest match. Clean up this.

    let mut keymap = HashMap::new();

    keymap.insert((Keymap::Insert, Key::Backspace), vec![Operation::Delete, Operation::Backward]);
    keymap.insert((Keymap::Query, Key::Backspace), vec![Operation::Left, Operation::Delete]);
    keymap.insert((Keymap::Normal, Key::Char('$')), vec![Operation::Eol]);
    keymap.insert((Keymap::Operator, Key::Char('$')), vec![Operation::Eol]);
    keymap.insert((Keymap::Select, Key::Char('$')), vec![Operation::Eol]);
    keymap.insert((Keymap::Insert, Key::Char('(')), vec![
        Operation::Input('('),
        Operation::Input(')'),
        Operation::Left,
    ]);
    keymap.insert((Keymap::Normal, Key::Char(';')), vec![Operation::Eval]);
    keymap.insert((Keymap::Operator, Key::Char(';')), vec![Operation::Eval]);
    keymap.insert((Keymap::Select, Key::Char(';')), vec![Operation::Eval]);
    keymap.insert((Keymap::Normal, Key::Char('E')), vec![Operation::Tail { reverse: true }]);
    keymap.insert((Keymap::Operator, Key::Char('E')), vec![Operation::Tail { reverse: true }]);
    keymap.insert((Keymap::Select, Key::Char('E')), vec![Operation::Tail { reverse: true }]);
    keymap.insert((Keymap::Normal, Key::Char('W')), vec![Operation::Head { reverse: true }]);
    keymap.insert((Keymap::Operator, Key::Char('W')), vec![Operation::Head { reverse: true }]);
    keymap.insert((Keymap::Select, Key::Char('W')), vec![Operation::Head { reverse: true }]);
    keymap.insert((Keymap::Normal, Key::Char('^')), vec![Operation::Bol]);
    keymap.insert((Keymap::Operator, Key::Char('^')), vec![Operation::Bol]);
    keymap.insert((Keymap::Select, Key::Char('^')), vec![Operation::Bol]);
    keymap.insert((Keymap::Normal, Key::Char('a')), vec![Operation::Insert, Operation::Right]);
    keymap.insert((Keymap::Operator, Key::Char('a')), vec![Operation::Insert, Operation::Right]);
    keymap.insert((Keymap::Select, Key::Char('a')), vec![Operation::Insert, Operation::Right]);
    keymap.insert((Keymap::Normal, Key::Char('d')), vec![Operation::Delete]);
    keymap.insert((Keymap::Operator, Key::Char('d')), vec![Operation::Delete]);
    keymap.insert((Keymap::Select, Key::Char('d')), vec![Operation::Delete]);
    keymap.insert((Keymap::Normal, Key::Char('e')), vec![Operation::Tail { reverse: false }]);
    keymap.insert((Keymap::Operator, Key::Char('e')), vec![Operation::Tail { reverse: false }]);
    keymap.insert((Keymap::Select, Key::Char('e')), vec![Operation::Tail { reverse: false }]);
    keymap.insert((Keymap::Normal, Key::Char('h')), vec![Operation::Left]);
    keymap.insert((Keymap::Operator, Key::Char('h')), vec![Operation::Left]);
    keymap.insert((Keymap::Select, Key::Char('h')), vec![Operation::Left]);
    keymap.insert((Keymap::Normal, Key::Char('i')), vec![Operation::Insert]);
    keymap.insert((Keymap::Operator, Key::Char('i')), vec![Operation::Insert]);
    keymap.insert((Keymap::Select, Key::Char('i')), vec![Operation::Insert]);
    keymap.insert((Keymap::Normal, Key::Char('j')), vec![Operation::Down]);
    keymap.insert((Keymap::Operator, Key::Char('j')), vec![Operation::Down]);
    keymap.insert((Keymap::Select, Key::Char('j')), vec![Operation::Down]);
    keymap.insert((Keymap::Normal, Key::Char('k')), vec![Operation::Up]);
    keymap.insert((Keymap::Operator, Key::Char('k')), vec![Operation::Up]);
    keymap.insert((Keymap::Select, Key::Char('k')), vec![Operation::Up]);
    keymap.insert((Keymap::Normal, Key::Char('l')), vec![Operation::Right]);
    keymap.insert((Keymap::Operator, Key::Char('l')), vec![Operation::Right]);
    keymap.insert((Keymap::Select, Key::Char('l')), vec![Operation::Right]);
    keymap.insert((Keymap::Normal, Key::Char('s')), vec![Operation::Surround]);
    keymap.insert((Keymap::Operator, Key::Char('s')), vec![Operation::Surround]);
    keymap.insert((Keymap::Select, Key::Char('s')), vec![Operation::Surround]);
    keymap.insert((Keymap::Normal, Key::Char('w')), vec![Operation::Head { reverse: false }]);
    keymap.insert((Keymap::Operator, Key::Char('w')), vec![Operation::Head { reverse: false }]);
    keymap.insert((Keymap::Select, Key::Char('w')), vec![Operation::Head { reverse: false }]);
    keymap.insert((Keymap::Insert, Key::Delete), vec![Operation::Delete, Operation::Forward]);
    keymap.insert((Keymap::Query, Key::Delete), vec![Operation::Delete]);
    keymap.insert((Keymap::Insert, Key::Down), vec![Operation::Down]);
    keymap.insert((Keymap::Normal, Key::Down), vec![Operation::Down]);
    keymap.insert((Keymap::Operator, Key::Down), vec![Operation::Down]);
    keymap.insert((Keymap::Query, Key::Down), vec![Operation::Down]);
    keymap.insert((Keymap::Select, Key::Down), vec![Operation::Down]);
    keymap.insert((Keymap::Insert, Key::Esc), vec![Operation::Escape, Operation::Left]);
    keymap.insert((Keymap::Normal, Key::Esc), vec![Operation::Escape]);
    keymap.insert((Keymap::Operator, Key::Esc), vec![Operation::Escape]);
    keymap.insert((Keymap::Query, Key::Esc), vec![Operation::Escape]);
    keymap.insert((Keymap::Select, Key::Esc), vec![Operation::Escape]);
    keymap.insert((Keymap::Insert, Key::Left), vec![Operation::Left]);
    keymap.insert((Keymap::Normal, Key::Left), vec![Operation::Left]);
    keymap.insert((Keymap::Operator, Key::Left), vec![Operation::Left]);
    keymap.insert((Keymap::Query, Key::Left), vec![Operation::Left]);
    keymap.insert((Keymap::Select, Key::Left), vec![Operation::Left]);
    keymap.insert((Keymap::Insert, Key::Right), vec![Operation::Right]);
    keymap.insert((Keymap::Normal, Key::Right), vec![Operation::Right]);
    keymap.insert((Keymap::Operator, Key::Right), vec![Operation::Right]);
    keymap.insert((Keymap::Query, Key::Right), vec![Operation::Right]);
    keymap.insert((Keymap::Select, Key::Right), vec![Operation::Right]);
    keymap.insert((Keymap::Insert, Key::Up), vec![Operation::Up]);
    keymap.insert((Keymap::Normal, Key::Up), vec![Operation::Up]);
    keymap.insert((Keymap::Operator, Key::Up), vec![Operation::Up]);
    keymap.insert((Keymap::Query, Key::Up), vec![Operation::Up]);
    keymap.insert((Keymap::Select, Key::Up), vec![Operation::Up]);

    keymap
}
