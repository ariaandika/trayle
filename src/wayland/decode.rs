use crate::wayland::{Frame, NewId, ObjectId, WlError, roundup4};

/// Decodable wayland message.
pub trait Decode {
    type Output<'a>;

    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError>;

    fn decode_with<'a>(message: Frame<'a>) -> Result<Self::Output<'a>, WlError> {
        Self::decode(Decoder::new(message))
    }
}

// ===== Decoder =====

pub struct Decoder<'a> {
    message: Frame<'a>,
}

impl<'a> Decoder<'a> {
    fn new(message: Frame<'a>) -> Self {
        Self { message }
    }

    pub fn pop_fd(&mut self) -> Result<i32, WlError> {
        match self.message.pop_fd() {
            Some(ok) => Ok(ok),
            None => Err(WlError::MissingFd),
        }
    }

    pub fn read<T: Read<'a>>(self) -> Result<T, WlError> {
        T::decode(&mut self.body())
    }

    pub fn body(self) -> Reader<'a> {
        Reader { read_buf: self.message.body() }
    }
}

// ===== Reader =====

pub struct Reader<'a> {
    read_buf: &'a [u8],
}

impl<'a> Reader<'a> {
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

impl Read<'_> for u32 {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        reader.read_ne_bytes().map(u32::from_ne_bytes)
    }
}

impl Read<'_> for ObjectId {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        ObjectId::from_ne_bytes(reader.read_ne_bytes()?).ok_or(WlError::ZeroId)
    }
}

impl<T> Read<'_> for NewId<T> {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        NewId::from_ne_bytes(reader.read_ne_bytes()?).ok_or(WlError::ZeroId)
    }
}

impl Read<'_> for i32 {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        reader.read_ne_bytes().map(i32::from_ne_bytes)
    }
}

impl<'a> Read<'a> for &'a str {
    fn decode(reader: &mut Reader<'a>) -> Result<Self, WlError> {
        let len = u32::decode(reader)?;
        if len == 0 {
            return Err(WlError::Null);
        }
        let round_len = roundup4!(len as u16) as usize;
        let Some((bytes, rest)) = reader.read_buf.split_at_checked(round_len) else {
            return Err(WlError::InvalidSize);
        };
        reader.read_buf = rest;
        // SAFETY: `len <= round_len` and `len` is non-zero
        let unrounded = unsafe { bytes.get_unchecked(..(len - 1) as usize) };
        match str::from_utf8(unrounded) {
            Ok(ok) => Ok(ok),
            Err(_) => Err(WlError::NonUtf8),
        }
    }
}

