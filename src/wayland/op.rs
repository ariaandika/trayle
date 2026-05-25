use crate::wayland::WlError;

// ===== trait =====

pub trait FromOpCode {
    type RequestOp;

    fn from_request_op(op: u16) -> Result<Self::RequestOp, WlError>;
}

// ===== Op =====

pub struct Op<O>(std::marker::PhantomData<O>);

impl<O> Op<O> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<O: FromOpCode> Op<O> {
    pub fn request(self, op: u16) -> Result<O::RequestOp, WlError> {
        O::from_request_op(op)
    }
}

// ===== std traits =====

// manual implementation to remove the `O: Trait` from derive macro

impl<O> Copy for Op<O> {}

impl<O> Clone for Op<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<O> std::fmt::Debug for Op<O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Op").finish()
    }
}
