use std::debug_assert_matches;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

use todex::sys::memmap::Memmap;
use todex::collections::slab::Slab;
use todex::wayland::interface::wl_shm::Error;
use todex::wayland::interface::wl_shm_pool::CreateBuffer;
use todex::wayland::object::{Object, ObjectError};
use todex::wayland::error::WlError;

use crate::handle::Handle;
use crate::shm::buffer::{Buffer, BufferFactory};

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

    /// Create [`ShmPool`].
    pub fn create_pool(&mut self, fd: i32, size: i32) -> Result<Handle<ShmPool>, Error> {
        Ok(Handle::from_idx(self.buf.insert(ShmPool::new(fd, size)?).0))
    }

    pub fn get_mut(&mut self, handle: Handle<ShmPool>) -> Result<&mut ShmPool, ObjectError> {
        self.buf
            .get_mut(handle.to_idx())
            .ok_or(ObjectError::UnknownId)
    }

    pub fn create_buffer(&mut self, handle: Handle<ShmPool>, msg: &CreateBuffer) -> Result<Buffer, WlError> {
        self.get_mut(handle)?.create_buffer(handle, msg).map_err(<_>::into)
    }

    pub fn destroy(&mut self, handle: Handle<ShmPool>) -> Result<(), ObjectError> {
        if self.get_mut(handle)?.destroy() {
            self.buf.remove(handle.to_idx());
        }
        Ok(())
    }

    pub fn destroy_buffer(&mut self, buffer: Buffer) -> Result<(), ObjectError> {
        let BufferFactory::ShmPool(handle) = buffer.factory;
        if self.get_mut(handle)?.destroy_buffer(buffer) {
            self.buf.remove(handle.to_idx());
        }
        Ok(())
    }
}

// ===== ShmPool =====

pub struct ShmPool {
    mem: Memmap,
    fd: OwnedFd,
    size: i32,
    ref_count: u32,
}

impl ShmPool {
    pub fn new(fd: i32, size: i32) -> Result<Self, Error> {
        let mem = Memmap::new(fd, size as usize).map_err(|e| {
            println!("ERROR: {fd}, {e}");
            Error::InvalidFd
        })?;
        // SAFETY: mmap call success indicate valid fd
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };
        Ok(Self {
            fd,
            mem,
            size,
            ref_count: 1,
        })
    }

    pub fn resize(&mut self, size: i32) -> Result<(), Error> {
        if size <= self.size {
            return Err(Error::InvalidStride);
        }
        self.mem = Memmap::new(self.fd.as_raw_fd(), size as usize).map_err(|_| Error::InvalidFd)?;
        self.size = size;
        Ok(())
    }

    fn create_buffer(
        &mut self,
        handle: Handle<ShmPool>,
        msg: &CreateBuffer,
    ) -> Result<Buffer, Error> {
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
            factory: {
                self.ref_count += 1;
                BufferFactory::ShmPool(handle)
            },
            wl_buffer: Object::from_new_id(msg.id),
        })
    }

    fn destroy(&mut self) -> bool {
        self.ref_count -= 1;
        self.ref_count == 0
    }

    fn destroy_buffer(&mut self, buffer: Buffer) -> bool {
        debug_assert_matches!(buffer.factory, BufferFactory::ShmPool(_));
        self.ref_count -= 1;
        self.ref_count == 0
    }
}
