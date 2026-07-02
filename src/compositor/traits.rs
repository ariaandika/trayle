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
    ($($ty:ident),* $(,)?) => {$(
        impl MessageHandler<$ty> for Compositor {
            fn handle(&mut self, msg: Msg<$ty>, client: &mut ClientMut) -> Result<(), WlError> {
                self.todo(msg, client)
            }
        }
    )*};
}

pub(crate) use todo_handler;

// ===== BindEffect =====

/// After global `wl_registry::bind` effect.
pub trait BindEffect<Interface> {
    fn bind(&mut self, obj: Object<Interface>, client: &mut ClientMut) -> Result<(), WlError>;
}
