use todex::collections::slab::Slab;
use todex::wayland::object::Object;
use todex::wayland::interface::wl_shm::FormatEnum;
use todex::wayland::interface::WlBuffer;

use crate::handle::Handle;
use crate::shm::ShmPool;

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

    pub fn insert(&mut self, buffer: Buffer) -> Handle<Buffer> {
        Handle::from_idx(self.buf.insert(buffer).0)
    }

    pub fn remove(&mut self, handle: Handle<Buffer>) -> Buffer {
        self.buf
            .remove(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}

impl std::ops::Index<Handle<Buffer>> for Buffers {
    type Output = Buffer;

    fn index(&self, handle: Handle<Buffer>) -> &Self::Output {
        self.buf
            .get(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}

impl std::ops::IndexMut<Handle<Buffer>> for Buffers {
    fn index_mut(&mut self, handle: Handle<Buffer>) -> &mut Self::Output {
        self.buf
            .get_mut(handle.to_idx())
            .unwrap_or_else(|| handle.dangling())
    }
}

// ===== Buffer =====

#[derive(Debug, Clone, Copy)]
pub enum BufferFactory {
    ShmPool(Handle<ShmPool>),
}

#[expect(dead_code)]
pub struct Buffer {
    pub offset: u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: FormatEnum,
    pub factory: BufferFactory,
    pub wl_buffer: Object<WlBuffer>,
}
