use crate::wayland::WlError;

// ===== trait =====

pub trait FromOp: Sized {
    fn from_op(op: u16) -> Result<Self, WlError>;
}
