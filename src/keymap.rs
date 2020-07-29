
// #[derive(Hash, PartialEq, Eq)]
// enum Keymap {
//     /// The default editor mode.
//     Normal,

//     /// Text input mode.
//     Edit,

//     /// Queries the user for a text range.
//     Select,

//     /// Queries the user for a text object and applies an operation.
//     Operator,

//     /// Queries the user for a text input and applies an operation.
//     Query,
// }

// impl<Buf: Buffer> From<Mode<Buf>> for Keymap {
//     fn from(mode: Mode<Buf>) -> Self {
//         match mode {
//             Mode::Edit => Keymap::Edit,
//             Mode::Normal { .. } => Keymap::Normal,
//             Mode::Select { .. } => Keymap::Select,
//             Mode::Operator { .. } => Keymap::Operator,
//             Mode::Query { .. } => Keymap::Query,
//         }
//     }
// }
