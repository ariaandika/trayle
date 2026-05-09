//! Wayland message.
//!
//! Contains:
//!
//! - [`Message`]
//! - [`EncodedMessage`]
//! - [`Writer`]
//! - [`Payload`]
#![allow(clippy::len_without_is_empty)]
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

pub struct Message(*const u8);

impl Message {
    pub(crate) fn new(ptr: *const u8) -> Self {
        Self(ptr)
    }

    pub fn object_id(&self) -> u32 {
        unsafe { *self.0.cast::<u32>() }
    }

    pub fn opcode(&self) -> u16 {
        unsafe { *self.0.add(4).cast::<u16>() }
    }

    pub fn len(&self) -> u16 {
        unsafe { *self.0.add(6).cast::<u16>() }
    }

    pub fn payload(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.0.add(8), (self.len() - 8) as usize) }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.0
    }
}

pub struct MessageHeader([u8; 8]);

impl MessageHeader {
    // pub(crate) fn new(header: [u8; 8]) -> Self {
    //     Self(header)
    // }

    pub fn object_id(&self) -> u32 {
        u32::from_ne_bytes(*self.0.first_chunk().unwrap())
    }

    pub fn opcode(&self) -> u16 {
        u16::from_ne_bytes(*self.0[4..6].first_chunk().unwrap())
    }

    pub fn len(&self) -> u16 {
        u16::from_ne_bytes(*self.0.last_chunk().unwrap())
    }
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
