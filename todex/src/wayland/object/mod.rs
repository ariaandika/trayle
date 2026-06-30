pub use handle::{AsHandle, Handle};
pub use error::ObjectError;
pub use new_id::{AsNewId, NewId};
pub use object::Object;
pub use global::{Global, WlGlobal, global_of};

mod handle;
mod error;
mod new_id;
mod object;
mod global;
