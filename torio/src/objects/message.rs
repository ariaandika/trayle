use tcio::bytes::{Buf, BytesMut};

// ===== Generic Message =====

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ObjectKind {
    /// `wl_display` (core)
    Display,
    /// `wl_registry` (core)
    Registry,
    /// `wl_shm` (core)
    Shm,
}

pub struct Header {
    bytes: [u8; 8],
}

pub struct Message {
    bytes: BytesMut,
}

#[derive(Clone, Copy, Debug)]
pub struct Fixed(i32);

impl Fixed {
    pub fn from_f32(f: f32) -> Self {
        Self((f * 256.0).round() as i32)
    }

    pub fn from_int(i: i32) -> Self {
        Self(i << 8)
    }

    pub fn to_float(self) -> f32 {
        self.0 as f32 / 256.0
    }

    pub fn to_raw(self) -> i32 {
        self.0
    }
}

impl Header {
    pub fn new(bytes: [u8; 8]) -> Self {
        Self { bytes }
    }

    pub fn len_of(bytes: &[u8; 8]) -> usize {
        unsafe { *bytes.as_ptr().add(6).cast::<u16>() as usize }
    }

    pub fn object_id(&self) -> u32 {
        unsafe { *self.bytes.as_ptr().cast::<u32>() }
    }

    pub fn opcode(&self) -> u16 {
        unsafe { *self.bytes.as_ptr().add(2).cast::<u16>() }
    }

    #[allow(clippy::len_without_is_empty, reason = "len >= 8")]
    pub fn len(&self) -> usize {
        unsafe { *self.bytes.as_ptr().add(6).cast::<u16>() as usize }
    }
}

impl Message {
    pub fn new(bytes: BytesMut) -> Self {
        assert!(bytes.len() >= 8, "len: {}", bytes.len());
        Self { bytes }
    }

    pub fn object_id(&self) -> u32 {
        unsafe { *self.bytes.as_ptr().cast::<u32>() }
    }

    pub fn opcode(&self) -> u16 {
        unsafe { *self.bytes.as_ptr().add(2).cast::<u16>() }
    }

    #[allow(clippy::len_without_is_empty, reason = "len >= 8")]
    pub fn len(&self) -> usize {
        unsafe { *self.bytes.as_ptr().add(6).cast::<u16>() as usize }
    }

    pub fn body(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.bytes.as_ptr().add(8), self.len() - 8) }
    }

    pub fn into_body(mut self) -> BytesMut {
        self.bytes.advance(8);
        self.bytes
    }
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header")
            .field("object_id", &self.object_id())
            .field("opcode", &self.opcode())
            .field("len", &self.len())
            .finish()
    }
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("object_id", &self.object_id())
            .field("opcode", &self.opcode())
            .field("len", &self.len())
            .field("body_len", &self.len().wrapping_sub(8))
            .finish()
    }
}

