use std::fmt::Debug;

use crate::event::Event;
use crate::state::Context;

mod insert;
mod normal;
mod operator;
mod query;
mod select;

pub use insert::Insert;
pub use normal::Normal;
pub use operator::Operator;
pub use query::Query;
pub use select::Select;

pub trait Mode: Debug + Send + Sync {
    /// Returns an user-friendly name for the mode.
    fn name(&self) -> &str;

    /// Advances the state state by handling an event.
    #[must_use]
    fn advance(self: Box<Self>, context: &mut Context, event: Event) -> Box<dyn Mode>;
}
