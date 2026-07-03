use std::debug_assert_matches;

use todex::collections::slab::Slab;
use todex::wayland::interface::wl_shm::Error;
use todex::wayland::interface::wl_shm_pool::CreateBuffer;
use todex::wayland::object::{Handle, ObjectError};
use todex::wayland::error::WlError;

use crate::wayland::buffer::{Buffer, BufferFactory};

// ===== ShmPools =====

const INITIAL_CAP: usize = 8;

pub struct ShmPools {
    buf: Slab<ShmPool>,
}

impl ShmPools {
    pub fn new() -> Self {
        Self {
            buf: Slab::with_capacity(INITIAL_CAP),
        }
    }

    pub fn insert(&mut self, fd: i32, size: i32) -> Handle {
        Handle::from_idx(self.buf.insert(ShmPool::new(fd, size)).0)
    }

    pub fn get_mut(&mut self, handle: Handle) -> Result<&mut ShmPool, ObjectError> {
        self.buf
            .get_mut(handle.to_idx())
            .ok_or(ObjectError::UnknownId)
    }

    pub fn create_buffer(&mut self, handle: Handle, msg: &CreateBuffer) -> Result<Buffer, WlError> {
        self.get_mut(handle)?.create_buffer(handle, msg).map_err(<_>::into)
    }

    pub fn destroy(&mut self, handle: Handle) -> Result<(), ObjectError> {
        if self.get_mut(handle)?.destroy() {
            self.buf.remove(handle.to_idx());
        }
        Ok(())
    }

    pub fn destroy_buffer(&mut self, buffer: Buffer) -> Result<(), ObjectError> {
        let handle = buffer.factory_handle;
        if self.get_mut(handle)?.destroy_buffer(buffer) {
            self.buf.remove(handle.to_idx());
        }
        Ok(())
    }
}

// ===== ShmPool =====

pub struct ShmPool {
    #[expect(dead_code)]
    fd: i32,
    size: i32,
    ref_count: u32,
}

impl ShmPool {
    pub fn new(fd: i32, size: i32) -> Self {
        // TODO: mmap the shm
        Self { fd, size, ref_count: 1 }
    }

    pub fn size(&self) -> i32 {
        self.size
    }

    pub fn resize(&mut self, size: i32) {
        debug_assert!(size > self.size);
        self.size = size;
    }

    fn create_buffer(&mut self, handle: Handle, msg: &CreateBuffer) -> Result<Buffer, Error> {
        // let end = offset + stride * (height - 1) + width * bytes_per_pixel;
        fn map_e<T>(_: T) -> Error {
            Error::InvalidFormat
        }
        Ok(Buffer {
            offset: msg.offset.try_into().map_err(map_e)?,
            width: msg.width.try_into().map_err(map_e)?,
            height: msg.height.try_into().map_err(map_e)?,
            stride: msg.stride.try_into().map_err(map_e)?,
            format: msg.format,
            factory: BufferFactory::ShmPool,
            factory_handle: {
                self.ref_count += 1;
                handle
            },
        })
    }

    fn destroy(&mut self) -> bool {
        self.ref_count -= 1;
        self.ref_count == 0
    }

    fn destroy_buffer(&mut self, buffer: Buffer) -> bool {
        debug_assert_matches!(buffer.factory, BufferFactory::ShmPool);
        self.ref_count -= 1;
        self.ref_count == 0
    }
}
