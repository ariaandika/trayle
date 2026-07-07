use todex::wayland::primitives::Version;
use todex::wayland::object::{Handle, Object};
use todex::wayland::message::{Message, WlMessage};
use todex::wayland::error::WlError;

use crate::client::ClientMut;

// ===== Handler =====

/// Request object.
pub type Obj<I> = Object<I>;

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

pub mod v2 {
    use super::*;

    /// Handle incoming message.
    pub trait MessageHandler<M>: Sized
    where
        M: WlMessage,
    {
        fn handle(
            &mut self,
            obj: Obj<M::WlInterface>,
            msg: Msg<M>,
            client: &mut ClientMut,
        ) -> Result<(), WlError>;
    }

    impl<C, M> MessageHandler<M> for C
    where
        M: WlMessage,
        C: super::MessageHandler<M>,
    {
        fn handle(
            &mut self,
            _: Obj<M::WlInterface>,
            msg: Msg<M>,
            client: &mut ClientMut,
        ) -> Result<(), WlError> {
            super::MessageHandler::handle(self, msg, client)
        }
    }
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
