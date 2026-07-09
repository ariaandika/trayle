use todex::wayland::primitives::Version;
use todex::wayland::object::Object;
use todex::wayland::message::{Message, WlMessage};

use crate::handle::{Handle, WithHandle};
use crate::compositor::error::HandleResult;
use crate::client::ClientMut;

pub use crate::compositor::error::CommitError;

// ===== Handler =====

/// Message payload, version, and resource handle.
///
/// Version is used for creating object with new id.
///
/// Resource [`Handle`] are safely typed with [`WithHandle`].
pub type Msg<M> =
    Message<M, Version, Handle<<<M as WlMessage>::WlInterface as WithHandle>::Handle>>;

/// Handle incoming message.
pub trait MessageHandler<M>: Sized
where
    M: WlMessage,
    M::WlInterface: WithHandle,
{
    // note on the `use<Self, M>`:
    // in 2024 edition, by default rust capture all lifetime and generic in `impl TraitMe`, but in
    // this case, `HandleResult` does not capture anything, therefore one must explicitly opt out of
    // lifetime capturing
    fn handle(&mut self, msg: Msg<M>, client: &mut ClientMut) -> impl HandleResult + use<Self, M>;
}

macro_rules! todo_handler {
    ($ty:ty) => {
        impl MessageHandler<$ty> for Compositor {
            fn handle(&mut self, _: Msg<$ty>, _: &mut ClientMut) -> Todo<$ty> {
                Todo::new()
            }
        }
    };
}

pub(crate) use todo_handler;

// ===== Effects =====

/// Side effect for `wl_registry::bind` request.
pub trait BindEffect<Interface> {
    fn bind(&mut self, obj: Object<Interface>, client: &mut ClientMut);
}

/// Side effect for `wl_surface::commit` request.
//
// This is assuming all surface role have single corresponding interface.
pub trait CommitEffect<Interface> {
    fn commit(&mut self, obj: Object<Interface>, client: &mut ClientMut) -> Result<(), CommitError>;
}
