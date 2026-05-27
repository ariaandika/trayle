use crate::buffer::Buffer;
use crate::wayland::{Id, WlError, roundup4};

// ===== Decode =====

/// Represent type that can be decoded from bytes.
pub trait Decode: Sized {
    type Output<'a>;

    /// Decode wayland message payload.
    ///
    /// `body` is message payload without the header.
    fn decode<'a>(reader: &mut Reader<'a>) -> Result<Self::Output<'a>, WlError>;

    fn decode_with<'a>(
        read_buf: &'a mut Buffer,
    ) -> Result<Self::Output<'a>, WlError> {
        let mut reader = Reader::new(read_buf);
        Self::decode(&mut reader)
    }
}

// ===== Reader =====

pub struct Reader<'a> {
    read_buf: &'a mut Buffer,
}

impl<'a> Reader<'a> {
    pub fn new(read_buf: &'a mut Buffer) -> Self {
        Self { read_buf }
    }

    fn as_array<const N: usize>(&mut self) -> Result<[u8; N], WlError> {
        let Some(chunk) = self.read_buf.try_split_first_chunk() else {
            return Err(WlError::InvalidSize);
        };
        Ok(*chunk)
    }

    pub fn read<T: PrimitiveDecode<'a>>(&mut self) -> Result<T, WlError> {
        T::decode(self)
    }

    pub fn read_fd(&mut self) -> Result<i32, WlError> {
        match self.read_buf.pop_front_fd() {
            Some(ok) => Ok(ok),
            None => Err(WlError::MissingFd),
        }
    }

    /// Not as generic `read` to separate with `i32` fd.
    pub fn read_int(&mut self) -> Result<i32, WlError> {
        Ok(i32::from_ne_bytes(self.as_array()?))
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], WlError> {
        let Some(bytes) = self.read_buf.try_split_to(len) else {
            return Err(WlError::InvalidSize);
        };
        // SAFETY: SWIGGITY SWOOTY
        // LIKE WHAT THE FUCK DO YOU MEAN, `read_buf` HAS LIFETIME OF `'a`, BUT THE RETURNED SLICE
        // IS LIFETIME OF `self` ????
        Ok(unsafe { std::mem::transmute::<&[u8], &[u8]>(bytes) })
        // Ok(bytes)
    }
}

// ===== primitive =====

pub trait PrimitiveDecode<'a>: Sized {
    fn decode(reader: &mut Reader<'a>) -> Result<Self, WlError>;
}

impl PrimitiveDecode<'_> for u32 {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        Ok(u32::from_ne_bytes(reader.as_array()?))
    }
}

impl PrimitiveDecode<'_> for Id {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        Ok(Id::from_ne_bytes(reader.as_array()?)?)
    }
}

impl PrimitiveDecode<'_> for u16 {
    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        Ok(u16::from_ne_bytes(reader.as_array()?))
    }
}

impl<'a> PrimitiveDecode<'a> for &'a str {
    fn decode(reader: &mut Reader<'a>) -> Result<Self, WlError> {
        let len = u32::decode(reader)?;
        if len == 0 {
            return Err(WlError::Null);
        }
        let round_len = roundup4!(len as u16) as u32;
        let bytes = reader.read_bytes(round_len as usize)?;
        // SAFETY: `len <= round_len` and `len` is non-zero
        let unrounded = unsafe { bytes.get_unchecked(..(len - 1) as usize) };
        Ok(str::from_utf8(unrounded).unwrap())
    }
}
