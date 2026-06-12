use crate::wayland::{Fixed, Frame, FromObjectId, NewId, Object, ObjectId, WlEnum, WlError};

/// Decodable wayland message.
pub trait Decode {
    type Output<'a>;

    /// Decode wayland message.
    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError>;

    /// Decode wayland message from a [`Frame`].
    fn decode_with<'a>(message: Frame<'a>) -> Result<Self::Output<'a>, WlError> {
        Self::decode(Decoder::new(message))
    }
}

// ===== Decoder =====

/// Message decoder API.
///
/// This is for internal usage only. To decode a message, use [`Decode::decode_with`].
pub struct Decoder<'a> {
    message: Frame<'a>,
}

impl<'a> Decoder<'a> {
    fn new(message: Frame<'a>) -> Self {
        Self { message }
    }

    /// Remove and returns fd from buffer.
    pub fn pop_fd(&mut self) -> Result<i32, WlError> {
        match self.message.pop_fd() {
            Some(ok) => Ok(ok),
            None => Err(WlError::MissingFd),
        }
    }

    /// Read one primitive value.
    pub fn read<T: Read<'a>>(self) -> Result<T, WlError> {
        T::decode(&mut self.reader())
    }

    /// Consume decoder and returns [`Reader`].
    pub fn reader(self) -> Reader<'a> {
        Reader { read_buf: self.message.body() }
    }
}

// ===== Reader =====

/// Primitive type reader.
pub struct Reader<'a> {
    read_buf: &'a [u8],
}

impl<'a> Reader<'a> {
    /// Read one primitive value.
    pub fn read<T: Read<'a>>(&mut self) -> Result<T, WlError> {
        T::decode(self)
    }

    fn read_ne_bytes(&mut self) -> Result<[u8; 4], WlError> {
        let Some((chunk, rest)) = self.read_buf.split_first_chunk() else {
            return Err(WlError::InvalidSize);
        };
        self.read_buf = rest;
        Ok(*chunk)
    }
}

// ===== Readable =====

pub trait Read<'a>: Sized {
    fn decode(reader: &mut Reader<'a>) -> Result<Self, WlError>;
}

impl<E: WlEnum> Read<'_> for E {
    fn decode(reader: &mut Reader<'_>) -> Result<Self, WlError> {
        reader.read().and_then(E::try_from_u32)
    }
}

impl Read<'_> for u32 {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        reader.read_ne_bytes().map(u32::from_ne_bytes)
    }
}

impl Read<'_> for ObjectId {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        reader
            .read_ne_bytes()
            .and_then(|id| ObjectId::new(u32::from_ne_bytes(id)).ok_or(WlError::ZeroId))
    }
}

impl Read<'_> for Option<ObjectId> {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        reader
            .read_ne_bytes()
            .map(|id| ObjectId::new(u32::from_ne_bytes(id)))
    }
}

impl<T: FromObjectId> Read<'_> for Object<T> {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        ObjectId::decode(reader).map(|id| Object::new(T::from_object_id(id)))
    }
}

impl<T: FromObjectId> Read<'_> for Option<Object<T>> {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        <Option<_>>::decode(reader).map(|e| e.map(|id| Object::new(T::from_object_id(id))))
    }
}

impl Read<'_> for Fixed {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        reader.read().map(Fixed::from_i32)
    }
}

impl<T> Read<'_> for NewId<T> {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        reader.read().map(NewId::new)
    }
}

impl Read<'_> for i32 {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        reader.read_ne_bytes().map(i32::from_ne_bytes)
    }
}

impl<'a> Read<'a> for &'a [u8] {
    fn decode(reader: &mut Reader<'a>) -> Result<Self, WlError> {
        let len = u32::decode(reader)?;
        let round_len = roundup4!(len as u16) as usize;
        let (bytes, rest) = reader
            .read_buf
            .split_at_checked(round_len)
            .ok_or(WlError::InvalidSize)?;
        reader.read_buf = rest;
        // SAFETY: `len <= round_len`
        Ok(unsafe { bytes.get_unchecked(..len as usize) })
    }
}

impl<'a> Read<'a> for &'a str {
    fn decode(reader: &mut Reader<'a>) -> Result<Self, WlError> {
        let len = u32::decode(reader)?;
        if len == 0 {
            return Err(WlError::Null);
        }
        decode_str(len, reader)
    }
}

impl<'a> Read<'a> for Option<&'a str> {
    fn decode(reader: &mut Reader<'a>) -> Result<Self, WlError> {
        let len = u32::decode(reader)?;
        if len == 0 {
            return Ok(None);
        }
        decode_str(len, reader).map(Some)
    }
}

fn decode_str<'a>(len: u32, reader: &mut Reader<'a>) -> Result<&'a str, WlError> {
    let round_len = roundup4!(len as u16) as usize;
    let Some((bytes, rest)) = reader.read_buf.split_at_checked(round_len) else {
        return Err(WlError::InvalidSize);
    };
    reader.read_buf = rest;
    // SAFETY: `len <= round_len` and `len` is non-zero
    let unrounded = unsafe { bytes.get_unchecked(..(len - 1) as usize) };
    str::from_utf8(unrounded).map_err(|_| WlError::NonUtf8)
}
