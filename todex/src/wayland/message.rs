use crate::wayland::{AsInterface, AsObjectId, AsOpCode, Interface, ObjectId};

// ===== trait =====

/// Type that represent wayland operation.
///
/// Wayland operation is either a request or event.
pub trait WlMessage: AsInterface + AsOpCode {
    const IS_REQUEST: bool;

    const IS_EVENT: bool = !Self::IS_REQUEST;

    const IS_DESTRUCTOR: bool = false;
}

// ===== message =====

/// Associate object id with a message.
#[derive(Debug)]
pub struct Message<T> {
    pub object_id: ObjectId,
    pub payload: T,
}

impl<T> Message<T> {
    /// Create new [`Message`].
    pub fn new<O: AsObjectId>(object: &O, payload: T) -> Self {
        Self {
            object_id: object.object_id(),
            payload,
        }
    }
}

impl<T> AsObjectId for Message<T> {
    #[inline]
    fn object_id(&self) -> ObjectId {
        self.object_id
    }
}

impl<T: AsInterface> AsInterface for Message<T> {
    #[inline]
    fn interface(&self) -> Interface {
        self.payload.interface()
    }
}

impl<T: AsOpCode> AsOpCode for Message<T> {
    type OpCode = T::OpCode;

    const OPCODE: Self::OpCode = T::OPCODE;

    const OPNAME: &str = T::OPNAME;
}

