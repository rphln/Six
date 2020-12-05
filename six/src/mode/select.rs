use crate::cursor::Cursor;

/// Selects a text range.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Select {
    anchor: Cursor,
}
