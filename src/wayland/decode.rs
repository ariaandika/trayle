use crate::wayland::{WlError, Id, roundup4};

// ===== trait =====

/// Represent type that can be decoded from bytes.
pub trait Decode: Sized {
    type Output<'a>;

    /// Decode wayland message payload.
    ///
    /// `body` is message payload without the header.
    fn decode<'a>(reader: &mut Reader<'a>) -> Result<Self::Output<'a>, WlError>;
}

// ===== Decoder =====

pub struct Decoder<D> {
    _p: std::marker::PhantomData<D>,
}

impl<D> Decoder<D> {
    pub fn new() -> Decoder<D> {
        Self { _p: std::marker::PhantomData }
    }
}

impl<D: Decode> Decoder<D> {
    pub fn decode(self, bytes: &[u8]) -> Result<D::Output<'_>, WlError> {
        let mut reader = Reader::new(bytes);
        let ok = D::decode(&mut reader)?;
        if reader.bytes.is_empty() {
            Ok(ok)
        } else {
            Err(WlError::ExcessiveSize)
        }
    }
}

// ===== Reader =====

pub struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn as_array<const N: usize>(&mut self) -> Result<[u8; N], WlError> {
        let Some((chunk, rest)) = self.bytes.split_first_chunk() else {
            return Err(WlError::InvalidSize);
        };
        self.bytes = rest;
        Ok(*chunk)
    }

    pub fn read<T: PrimitiveDecode<'a>>(&mut self) -> Result<T, WlError> {
        T::decode(self)
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], WlError> {
        let Some((bytes, rest)) = self.bytes.split_at_checked(len) else {
            return Err(WlError::InvalidSize);
        };
        self.bytes = rest;
        Ok(bytes)
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
        let round_len = roundup4!(len as u16) as u32;
        let bytes = reader.read_bytes(round_len as usize)?;
        Ok(str::from_utf8(bytes).unwrap())
    }
}
