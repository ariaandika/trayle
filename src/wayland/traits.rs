use crate::wayland::{AsObjectId, FromObjectId, Interface, NewId, WlError};

/// Request/event opcode.
pub trait OpCode: Sized {
    fn from_op(op: u16) -> Option<Self>;

    fn to_op(self) -> u16;

    fn try_from_op(op: u16) -> Result<Self, WlError> {
        match Self::from_op(op) {
            Some(ok) => Ok(ok),
            None => Err(WlError::UnknownOp),
        }
    }
}

/// Object that is a wayland interface.
pub trait AsInterface {
    /// The interface of this object is associated with.
    const INTERFACE: Interface;
}

impl<T: AsInterface> AsInterface for NewId<T> {
    const INTERFACE: Interface = T::INTERFACE;
}

/// Wayland object.
pub trait WlObject: FromObjectId + AsObjectId + AsInterface {}

impl<O: FromObjectId + AsObjectId + AsInterface> WlObject for O {}
