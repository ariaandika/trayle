use todex::wayland::message::{Message, WlMessage};
use todex::wayland::primitives::Version;

use crate::handle::{Handle, WithHandle};

pub use crate::compositor::error::CommitError;

// ===== Handler =====

/// Message payload, version, and resource handle.
///
/// Version is used for creating object with new id.
///
/// Resource [`Handle`] are safely typed with [`WithHandle`].
pub type Msg<M> =
    Message<M, Version, Handle<<<M as WlMessage>::WlInterface as WithHandle>::Handle>>;
