use crate::buffer::Buffer;
use crate::wayland::{Id, WlError, roundup4};

/// Decodable wayland message.
pub trait Decode {
    type Output<'a>;

    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError>;

    fn decode_with<'a>(read_buf: &'a mut Buffer) -> Result<Self::Output<'a>, WlError> {
        Decoder::new(read_buf).and_then(Self::decode)
    }
}

// ===== Decoder =====

pub struct Decoder<'a> {
    len: u16,
    read_buf: &'a mut Buffer,
}

impl<'a> Decoder<'a> {
    fn new(read_buf: &'a mut Buffer) -> Result<Self, WlError> {
        let Some(header) = read_buf.try_split_first_chunk::<8>() else {
            return Err(WlError::InvalidSize);
        };
        let Some(len) = u16::from_ne_bytes(*header.last_chunk().unwrap()).checked_sub(8) else {
            return Err(WlError::InvalidSize);
        };
        Ok(Self { len, read_buf })
    }

    pub fn pop_fd(&mut self) -> Result<i32, WlError> {
        match self.read_buf.pop_front_fd() {
            Some(ok) => Ok(ok),
            None => Err(WlError::MissingFd),
        }
    }

    pub fn read<T: PrimitiveDecode<'a>>(self) -> Result<T, WlError> {
        T::decode(&mut self.body()?)
    }

    pub fn body(self) -> Result<Reader<'a>, WlError> {
        match self.read_buf.try_split_to(self.len as usize) {
            Some(read_buf) => Ok(Reader { read_buf }),
            None => Err(WlError::InvalidSize),
        }
    }
}

// ===== Reader =====

pub struct Reader<'a> {
    read_buf: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn read<T: PrimitiveDecode<'a>>(&mut self) -> Result<T, WlError> {
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

// ===== primitive =====

pub trait PrimitiveDecode<'a>: Sized {
    fn decode(reader: &mut Reader<'a>) -> Result<Self, WlError>;
}

impl PrimitiveDecode<'_> for u32 {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        reader.read_ne_bytes().map(u32::from_ne_bytes)
    }
}

impl PrimitiveDecode<'_> for Id {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        Id::from_ne_bytes(reader.read_ne_bytes()?).map_err(<_>::into)
    }
}

impl PrimitiveDecode<'_> for i32 {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        reader.read_ne_bytes().map(i32::from_ne_bytes)
    }
}

impl<'a> PrimitiveDecode<'a> for &'a str {
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

