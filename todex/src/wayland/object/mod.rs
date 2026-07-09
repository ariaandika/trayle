pub use error::{UnknownId, OccupiedNewId};
pub use new_id::{AsNewId, NewId};
pub use object::Object;
pub use global::{Global, WlGlobal, global_of};

mod error;
mod new_id;
mod object;
mod global;
