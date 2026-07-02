use std::task::Poll::{*, self};

use crate::sys::bytes::Bytes;
use crate::sys::cmsg::Cmsg;
use crate::wayland::primitives::{AsObjectId, ObjectId, Version};
use crate::wayland::message::Message;
use crate::wayland::wire::{DecodeError, OpCode, Reader};

use DecodeError as E;

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

// ===== RawMessage =====

#[derive(Debug, Clone, Copy)]
pub struct Payload<'a>(&'a [u8]);

/// A decodable raw bytes message.
pub type RawMessage<'a, Op = u16> = Message<Payload<'a>, Op>;

impl<'a> RawMessage<'a, u16> {
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
        let Some(msg) = bytes.split_to(len as usize) else {
            return Pending;
        };
        let Some(payload) = msg.get(8..) else {
            return Ready(Err(E::InsufficientSize));
        };
        Ready(Ok(Message::from_parts(id, Payload(payload), hdr2 as u16)))
    }

    #[inline]
    pub fn opcode(&self) -> u16 {
        self.meta()
    }

    #[inline]
    pub fn with_op<Op: OpCode>(self) -> Result<Message<Payload<'a>, Op>, DecodeError> {
        let op = Op::try_from_op(self.meta())?;
        Ok(Message::from_parts(
            self.object_id(),
            self.into_payload(),
            op,
        ))
    }
}

impl<'a, Op> RawMessage<'a, Op> {
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
