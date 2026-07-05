use crate::sys::bytes::Bytes;
use crate::sys::cmsg::Cmsg;
use crate::wayland::primitives::{ObjectId};
use crate::wayland::message::Message;
use crate::wayland::wire::{DecodeError, Reader};

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

// ===== Payload =====

#[derive(Debug, Clone, Copy)]
pub struct Payload<'a>(&'a [u8]);

impl<'a> Message<Payload<'a>, u16> {
    #[inline]
    pub fn get_message(bytes: &'a mut Bytes) -> Result<Option<Self>, DecodeError> {
        let Some(header) = bytes.first_chunk::<8>() else {
            return Ok(None);
        };
        let Some(id) = ObjectId::new(u32::from_ne_bytes(*header.first_chunk().unwrap())) else {
            return Err(E::ZeroId);
        };
        let hdr2 = u32::from_ne_bytes(*header.last_chunk().unwrap());
        let len = hdr2 >> u16::BITS;
        let Some(msg) = bytes.split_to(len as usize) else {
            return Ok(None);
        };
        let Some(payload) = msg.get(8..) else {
            return Err(E::InsufficientSize);
        };
        Ok(Some(Message::from_parts(id, Payload(payload), hdr2 as u16)))
    }

    #[inline]
    pub fn decode_payload<const N: usize, P>(
        self,
        cmsg: &mut Cmsg,
    ) -> Result<P::Output<'a>, DecodeError>
    where
        P: DecodePayload<Fd = [i32; N]>,
    {
        P::decode_payload(
            Reader::new(self.into_payload().0),
            cmsg.read_chunk().ok_or(DecodeError::MissingFd)?,
        )
    }
}

impl<T, D> Message<T, u16, D> {
    #[inline]
    pub fn opcode(&self) -> u16 {
        self.meta()
    }
}
