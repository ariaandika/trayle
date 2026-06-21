use crate::wayland::prelude::*;
use crate::wayland::wl_shm_pool::WlShmPool;

#[derive(Interface, Debug)]
#[interface(global = 2)]
pub struct WlShm {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    CreatePool,
    Release,
}

#[derive(Message, Debug)]
#[message(request = WlShm)]
pub struct CreatePool {
    pub id: NewId<WlShmPool>,
    #[fd]
    pub fd: i32,
    pub size: i32,
}

#[derive(Message, Debug)]
#[message(request = WlShm, destructor)]
pub struct Release;

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Format,
}

#[derive(Message, Debug)]
#[message(event = WlShm)]
pub struct Format {
    pub format: PixelFormat,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    /// buffer format is not known
    InvalidFormat,
    /// invalid size or stride during pool or buffer creation
    InvalidStride,
    /// mmapping the file descriptor failed
    InvalidFd,
}

#[derive(WlEnum, Debug, Clone, Copy)]
#[repr(u32)]
pub enum PixelFormat {
    Argb8888 = 0,
    Xrgb8888 = 1,
}
