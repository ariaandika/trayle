use crate::wayland::AsInterface;
use crate::wayland::primitives::{AsObjectId, FromObjectId};

pub use error::ObjectError;
pub use object::Object;
pub use global::{Global, WlGlobal, global_of};
pub use handle::{AsHandle, Handle};

mod error;
mod object;
mod global;
mod handle;

// ===== trait =====

/// Type that represent a wayland object.
pub trait WlObject: FromObjectId + AsObjectId + AsInterface {}

impl<O: FromObjectId + AsObjectId + AsInterface> WlObject for O {}
