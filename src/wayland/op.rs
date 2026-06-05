use crate::wayland::WlError;

pub trait OpCode: Sized {
    fn from_op(op: u16) -> Result<Self, WlError>;

    fn to_op(self) -> u16;
}
