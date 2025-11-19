use tcio::bytes::BytesMut;

mod id;
pub mod wl_display;

pub use id::GlobalId;

pub trait Request {
    const OP_CODE: u16;

    fn object_id(&self) -> u32;

    /// Note that implementor should not remove bytes in buffer.
    ///
    /// In future the buffer type should be changed.
    fn write_body(&self, buffer: &mut BytesMut);
}

