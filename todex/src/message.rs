//! Wayland message.
//!
//! Contains:
//!
//! - [`Message`]
//! - [`EncodedMessage`]
//! - [`Writer`]
//! - [`Payload`]
#![allow(clippy::len_without_is_empty)]
use crate::Id;
use crate::error::DecodeError;

/// Message writer.
///
/// # Safety
///
/// Implementor must ensure that the pointer returned from [`spare`] is valid for write until given
/// length.
///
/// [`spare`]: Writer::spare
pub unsafe trait Writer {
    fn spare(&mut self, size: u16) -> *mut u8;
}

pub trait EncodePayload {
    const OPCODE: u16;

    fn encoded_size(&self) -> u16;

    /// Encode message payload to given pointer.
    ///
    /// For safe alternative, see [`encode()`].
    ///
    /// [`encode()`]: Encode::encode
    ///
    /// # Safety
    ///
    /// `ptr` must be valid for write until [`size()`] length.
    ///
    /// [`size()`]: Encode::size
    unsafe fn encode_raw(&self, ptr: *mut u8);
}

pub trait DecodePayload<'a>: Sized {
    /// Decode message payload from given pointer.
    ///
    /// # Safety
    ///
    /// `msg` must contains valid message header and its payload length.
    unsafe fn decode_raw(msg: *const u8) -> Result<Self, DecodeError>;
}

pub struct Message<P> {
    pub object_id: Id,
    pub opcode: u16,
    pub payload: P,
}

pub fn encode_message<P: EncodePayload, W: Writer>(object_id: u32, payload: &P, writer: &mut W) {
    let size = payload.encoded_size();
    let ptr = writer.spare(8 + size);
    // SAFETY: `Writer` implementation guarantee the pointer is valid for write until `size`
    unsafe {
        ptr.cast::<u32>().write(object_id);
        ptr.add(4).cast::<u16>().write(P::OPCODE);
        ptr.add(6).cast::<u16>().write(size);
        payload.encode_raw(ptr.add(8));
    }
}

pub fn decode_message<'a, P, W>(bytes: &[u8]) -> Result<Option<P>, DecodeError>
where
    P: DecodePayload<'a>,
    W: Writer,
{
    let Some(header) = bytes.first_chunk::<8>() else {
        return Ok(None);
    };
    let len = u16::from_ne_bytes(*header.last_chunk().unwrap());
    if len < 8 {
        return Err(DecodeError::Insufficient);
    }
    if bytes.len() < len as usize {
        return Ok(None);
    }
    // SAFETY: `bytes` is checked that it contains valid message
    unsafe { P::decode_raw(bytes.as_ptr()).map(Some) }
}
