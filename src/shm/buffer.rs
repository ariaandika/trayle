use todex::collections::slab::Slab;
use todex::wayland::object::{Handle, Object, ObjectError};
use todex::wayland::interface::wl_shm::FormatEnum;
use todex::wayland::interface::WlBuffer;

// ===== Buffers =====

const INITIAL_CAP: usize = 8;

pub struct Buffers {
    buf: Slab<Buffer>,
}

impl Buffers {
    pub fn new() -> Self {
        Self {
            buf: Slab::with_capacity(INITIAL_CAP),
        }
    }

    pub fn insert(&mut self, buffer: Buffer) -> Handle {
        Handle::from_idx(self.buf.insert(buffer).0)
    }

    pub fn get_mut(&mut self, handle: Handle) -> Result<&mut Buffer, ObjectError> {
        self.buf
            .get_mut(handle.to_idx())
            .ok_or(ObjectError::UnknownId)
    }

    pub fn remove(&mut self, handle: Handle) -> Result<Buffer, ObjectError> {
        self.buf
            .remove(handle.to_idx())
            .ok_or(ObjectError::UnknownId)
    }
}

// ===== Buffer =====

#[derive(Debug, Clone, Copy)]
pub enum BufferFactory {
    ShmPool,
}

#[expect(dead_code)]
pub struct Buffer {
    pub offset: u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: FormatEnum,
    pub factory: BufferFactory,
    pub factory_handle: Handle,
    pub wl_buffer: Object<WlBuffer>,
}
