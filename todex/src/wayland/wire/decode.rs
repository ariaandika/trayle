use crate::wayland::wire::{DecodeError, Frame, Read, Reader};

// ===== trait =====

/// Decode wayland message.
pub trait Decode {
    type Output<'a>;

    /// Decode wayland message.
    ///
    /// Note that this is for implementor, application should use [`Decode::decode_frame`] instead.
    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, DecodeError>;

    /// Decode wayland message from a [`Frame`].
    #[inline]
    fn decode_frame<'a>(message: Frame<'a>) -> Result<Self::Output<'a>, DecodeError> {
        Self::decode(Decoder::new(message))
    }
}

// ===== Decoder =====

/// Message decoder helper.
///
/// This is for internal usage only, application should use [`Decode`] instead.
pub struct Decoder<'a> {
    message: Frame<'a>,
}

impl<'a> Decoder<'a> {
    fn new(message: Frame<'a>) -> Self {
        Self { message }
    }

    /// Removes the first `fd` and returns it.
    #[inline]
    pub fn pop_fd(&mut self) -> Result<i32, DecodeError> {
        match self.message.pop_fd() {
            Some(ok) => Ok(ok),
            None => Err(DecodeError::MissingFd),
        }
    }

    /// Read single primitive value.
    pub fn read<T: Read<'a>>(self) -> Result<T, DecodeError> {
        T::read(&mut self.reader())
    }

    /// Consume decoder and returns [`Reader`].
    #[inline]
    pub fn reader(self) -> Reader<'a> {
        Reader::new(self.message.body())
    }
}
