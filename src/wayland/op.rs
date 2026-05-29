use crate::wayland::WlError;

// ===== trait =====

pub trait FromOp: Sized {
    fn from_op(op: u16) -> Result<Self, WlError>;
}

pub trait ToOp: Sized {
    fn to_op(&self) -> u16;
}

impl ToOp for u16 {
    #[inline]
    fn to_op(&self) -> u16 {
        *self
    }
}
