use tcio::bytes::BytesMut;

mod id;
pub mod wl_display;
pub mod wl_registry;

pub use id::GlobalId;

pub trait Request {
    const OP_CODE: u16;

    fn object_id(&self) -> u32;

    /// Note that implementor should not remove bytes in buffer.
    ///
    /// In future the buffer type should be changed.
    fn write_body(&self, buffer: &mut BytesMut);
}

// ===== Generic Message =====

pub struct Header {
    bytes: [u8; 8],
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

pub struct Message {
    bytes: BytesMut,
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

// ===== Utils =====

#[macro_export]
macro_rules! roundup_4 {
    ($n:expr) => {
        ((($n) + 3usize) & (usize::MAX << 2))
    };
}

pub use {roundup_4};
