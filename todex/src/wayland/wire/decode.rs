use std::task::Poll::{*, self};

use crate::sys::bytes::Bytes;
use crate::sys::cmsg::Cmsg;
use crate::wayland::primitives::{AsObjectId, ObjectId, Version};
use crate::wayland::message::Message;
use crate::wayland::wire::{DecodeError, Frame, OpCode, Read, Reader};

use DecodeError as E;

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
        self.message.pop_fd().ok_or(E::MissingFd)
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

// ===== DecodePayload =====

/// Decode message payload.
pub trait DecodePayload {
    type Output<'a>;

    type Fd;

    /// Decode the payload.
    fn decode_payload<'a>(
        reader: Reader<'a>,
        fd: Self::Fd,
    ) -> Result<Self::Output<'a>, DecodeError>;
}

// ===== Payload =====

#[derive(Debug, Clone, Copy)]
pub struct Payload<'a>(&'a [u8]);

// ===== RawMessage =====

/// A decodable raw bytes message.
pub type RawMessage<'a> = Message<Payload<'a>, u16>;

impl<'a> RawMessage<'a> {
    #[inline]
    pub fn decode_with(bytes: &'a mut Bytes) -> Poll<Result<Self, DecodeError>> {
        let Some(header) = bytes.first_chunk::<8>() else {
            return Pending;
        };
        let Some(id) = ObjectId::new(u32::from_ne_bytes(*header.first_chunk().unwrap())) else {
            return Ready(Err(E::ZeroId));
        };
        let hdr2 = u32::from_ne_bytes(*header.last_chunk().unwrap());
        let len = hdr2 >> u16::BITS;
        let Some(body_len) = len.checked_sub(8) else {
            return Ready(Err(E::InsufficientSize));
        };
        // bytes.split
        let Some(payload) = bytes.get(..body_len as usize) else {
            return Pending;
        };
        Ready(Ok(Message::from_parts(id, Payload(payload), hdr2 as u16)))
    }

    #[inline]
    pub fn opcode<Op: OpCode>(&self) -> Result<Op, DecodeError> {
        Op::try_from_op(self.marker())
    }

    #[inline]
    pub fn decode_payload<const N: usize, P>(
        self,
        cmsg: &mut Cmsg,
        version: Version,
    ) -> Result<Message<P::Output<'a>, Version>, DecodeError>
    where
        P: DecodePayload<Fd = [i32; N]>,
    {
        Ok(Message::from_parts(
            self.object_id(),
            P::decode_payload(
                Reader::new(self.into_payload().0),
                cmsg.read_chunk().ok_or(DecodeError::MissingFd)?,
            )?,
            version,
        ))
    }
}
