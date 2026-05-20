use crate::wayland::WlError;

/// Represent type that can be decoded from bytes.
pub trait Decode: Sized {
    /// Decode wayland message payload.
    ///
    /// `body` is message payload without the header.
    fn decode(body: &[u8]) -> Result<Self, WlError>;
}

pub struct Decoder<D> {
    _p: std::marker::PhantomData<D>,
}

impl<D: Decode> Decoder<D> {
    pub fn new() -> Decoder<D> {
        Self { _p: std::marker::PhantomData }
    }

    pub fn decode(self, body: &[u8]) -> Result<D, WlError> {
        D::decode(body)
    }
}
