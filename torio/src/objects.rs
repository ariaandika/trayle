use tcio::bytes::BytesMut;

mod message;

mod id;
pub mod wl_display;
pub mod wl_registry;

pub use id::{GlobalId, ObjectManager};
pub use message::{ObjectKind, Header, Message};

pub trait Request {
    const OP_CODE: u16;

    fn object_id(&self) -> u32;

    /// Note that implementor should not remove bytes in buffer.
    ///
    /// In future the buffer type should be changed.
    fn write_body(&self, buffer: &mut BytesMut);
}

/// Wayland object.
pub trait Object {
    const KIND: ObjectKind;
}

// ===== Utils =====

#[macro_export]
macro_rules! roundup_4 {
    ($n:expr) => {
        ((($n) + 3usize) & (usize::MAX << 2))
    };
}

pub use {roundup_4};
