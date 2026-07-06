use todex::wayland::primitives::Version;
use todex::wayland::object::{Handle, Object};
use todex::wayland::message::Message;
use todex::wayland::error::WlError;

use crate::client::ClientMut;

// ===== Handler =====

/// Request message.
///
/// Contains payload, version, and object id.
///
/// Version is used for creating object with new id.
pub type Msg<M> = Message<M, Version, Handle>;

/// Handle incoming message.
pub trait MessageHandler<M>: Sized {
    fn handle(&mut self, msg: Msg<M>, client: &mut ClientMut) -> Result<(), WlError>;
}

macro_rules! todo_handler {
    ($ty:ident) => {
        impl MessageHandler<$ty> for Compositor {
            fn handle(&mut self, _: Msg<$ty>, _: &mut ClientMut) -> Result<(), WlError> {
                Err(WlError::NotYetImplemented)
            }
        }
    };
}

pub(crate) use todo_handler;

// ===== BindEffect =====

/// Side effect for `wl_registry::bind` request.
pub trait BindEffect<Interface> {
    fn bind(&mut self, obj: Object<Interface>, client: &mut ClientMut) -> Result<(), WlError>;
}

/// Side effect for `wl_surface::commit` request.
//
// This is assuming all surface role have single corresponding interface.
pub trait CommitEffect<Interface> {
    fn commit(&mut self, obj: Object<Interface>, client: &mut ClientMut) -> Result<(), WlError>;
}
