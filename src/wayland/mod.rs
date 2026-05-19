pub use id::Id;
pub use error::WlError;

use crate::buffer::Buffer;

mod id;
mod error;

pub mod wl_display;
pub mod wl_registry;
pub mod wl_callback;

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

use roundup4;

/// `(id, op, len)`
pub fn header(bytes: &[u8]) -> Option<(u32, u16, u16)> {
    let header = bytes.first_chunk::<8>()?;
    let ptr = header.as_ptr();
    unsafe {
        let id = u32::from_ne_bytes(*ptr.cast::<[u8; _]>());
        let op = u16::from_ne_bytes(*ptr.add(4).cast::<[u8; _]>());
        let len = u16::from_ne_bytes(*ptr.add(6).cast::<[u8; _]>());
        Some((id, op, len))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum Interface {
    WlDisplay,
    WlRegistry,
}

// ===== Encode =====

/// Message writer.
///
/// # Safety
///
/// Implementor must ensure that the pointer returned from [`message`] is valid for write until given
/// length.
///
/// [`message`]: Self::message
pub unsafe trait Write: Sized {
    /// Returns writable memory.
    ///
    /// # Safety
    ///
    /// Caller must initialize the returned pointer until `len` bytes.
    unsafe fn spare(&mut self, len: u32) -> *mut u8;

    fn encode<E: Encode>(&mut self, message: &E) {
        message.encode(self);
    }
}

pub trait Encode {
    fn encode<W: Write>(&self, writer: W);
}

unsafe impl Write for Buffer {
    unsafe fn spare(&mut self, len: u32) -> *mut u8 {
        self.reserve(len);
        // SAFETY: caller ensure the returned pointer will be initialized
        let ptr = self.spare_capacity_mut().as_mut_ptr().cast();
        unsafe { self.advance_mut(len) };
        ptr
    }
}

// ===== Decode =====

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

    pub fn decode(&self, body: &[u8]) -> Result<D, WlError> {
        D::decode(body)
    }
}

// ===== PrimitiveEncode =====

trait PrimitiveEncode {
    /// # Safety
    ///
    /// Pointer must be valid for write until required length.
    unsafe fn encode(self, ptr: *mut u8);
}

trait PtrWrite {
    /// # Safety
    ///
    /// Pointer must be valid for write until required length.
    unsafe fn put<P: PrimitiveEncode>(self, value: P);
}

impl PrimitiveEncode for u32 {
    unsafe fn encode(self, ptr: *mut u8) {
        unsafe { ptr.copy_from_nonoverlapping(self.to_ne_bytes().as_ptr(), 4) };
    }
}

impl PrimitiveEncode for u16 {
    unsafe fn encode(self, ptr: *mut u8) {
        unsafe { ptr.copy_from_nonoverlapping(self.to_ne_bytes().as_ptr(), 2) };
    }
}

impl PtrWrite for *mut u8 {
    unsafe fn put<P: PrimitiveEncode>(self, value: P) {
        unsafe { value.encode(self) };
    }
}

// ===== blanket implementation =====

unsafe impl<W: Write> Write for &mut W {
    unsafe fn spare(&mut self, len: u32) -> *mut u8 {
        unsafe { W::spare(self, len) }
    }
}
