use crate::wayland::primitives::{Fixed, ObjectId, Version, WlEnum};
use crate::wayland::object::{NewId, Object};
use crate::wayland::interface::InterfaceMarker;
use crate::wayland::wire::DecodeError;

use DecodeError as E;

// ===== Reader =====

/// Primitive type reader.
pub struct Reader<'a> {
    read_buf: &'a [u8],
}

impl<'a> Reader<'a> {
    pub(super) fn new(read_buf: &'a [u8]) -> Self {
        Self { read_buf }
    }

    /// Read one primitive value.
    pub fn read<T: Read<'a>>(&mut self) -> Result<T, DecodeError> {
        T::read(self)
    }

    fn read_ne_bytes(&mut self) -> Result<[u8; 4], DecodeError> {
        let Some((chunk, rest)) = self.read_buf.split_first_chunk() else {
            return Err(E::InsufficientSize);
        };
        self.read_buf = rest;
        Ok(*chunk)
    }
}

// ===== Readable =====

pub trait Read<'a>: Sized {
    fn read(reader: &mut Reader<'a>) -> Result<Self, DecodeError>;
}

// blanket impl

impl<E: WlEnum> Read<'_> for E {
    #[inline]
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        reader
            .read()
            .and_then(|e| E::from_u32(e).ok_or(DecodeError::UnknownEnumEntry))
    }
}

// primitives

impl Read<'_> for u32 {
    #[inline]
    fn read(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_ne_bytes().map(u32::from_ne_bytes)
    }
}

impl Read<'_> for i32 {
    #[inline]
    fn read(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read_ne_bytes().map(i32::from_ne_bytes)
    }
}

impl Read<'_> for Version {
    #[inline]
    fn read(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader
            .read_ne_bytes()
            .and_then(|id| Version::new(u32::from_ne_bytes(id)).ok_or(E::InvalidVersion))
    }
}

impl Read<'_> for Fixed {
    #[inline]
    fn read(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader.read().map(Fixed::from_i32)
    }
}

// generic

impl<T: InterfaceMarker> Read<'_> for Object<T> {
    #[inline]
    fn read(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader
            .read_ne_bytes()
            .and_then(|id| ObjectId::new(u32::from_ne_bytes(id)).ok_or(E::ZeroId))
            .map(|id| Object::new_typed(id, T::MARKER))
    }
}

impl<T: InterfaceMarker> Read<'_> for Option<Object<T>> {
    #[inline]
    fn read(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader
            .read_ne_bytes()
            .map(|id| ObjectId::new(u32::from_ne_bytes(id)))
            .map(|id| id.map(|id| Object::new_typed(id, T::MARKER)))
    }
}

impl<T: InterfaceMarker> Read<'_> for NewId<T> {
    #[inline]
    fn read(reader: &mut Reader) -> Result<Self, DecodeError> {
        reader
            .read_ne_bytes()
            .and_then(|id| ObjectId::new(u32::from_ne_bytes(id)).ok_or(E::ZeroId))
            .map(|id| NewId::new(id, T::MARKER))
    }
}

// arbitrary length

impl<'a> Read<'a> for &'a [u8] {
    #[inline]
    fn read(reader: &mut Reader<'a>) -> Result<Self, DecodeError> {
        let len = u32::read(reader)?;
        let round_len = roundup4!(len as u16) as usize;
        let (bytes, rest) = reader
            .read_buf
            .split_at_checked(round_len)
            .ok_or(E::InsufficientSize)?;
        reader.read_buf = rest;
        // SAFETY: `len <= round_len`
        Ok(unsafe { bytes.get_unchecked(..len as usize) })
    }
}

impl<'a> Read<'a> for &'a str {
    #[inline]
    fn read(reader: &mut Reader<'a>) -> Result<Self, DecodeError> {
        match u32::read(reader)? {
            0 => Err(E::Null),
            len => decode_str(len, reader),
        }
    }
}

impl<'a> Read<'a> for Option<&'a str> {
    #[inline]
    fn read(reader: &mut Reader<'a>) -> Result<Self, DecodeError> {
        match u32::read(reader)? {
            0 => Ok(None),
            len => decode_str(len, reader).map(Some),
        }
    }
}

fn decode_str<'a>(len: u32, reader: &mut Reader<'a>) -> Result<&'a str, DecodeError> {
    let round_len = roundup4!(len as u16) as usize;
    let Some((bytes, rest)) = reader.read_buf.split_at_checked(round_len) else {
        return Err(E::InsufficientSize);
    };
    reader.read_buf = rest;
    // SAFETY: `len <= round_len` and `len` is non-zero
    let unrounded = unsafe { bytes.get_unchecked(..(len - 1) as usize) };
    str::from_utf8(unrounded).map_err(|_| E::NonUtf8)
}
