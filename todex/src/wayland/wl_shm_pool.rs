use crate::wayland::prelude::*;
use crate::wayland::wl_buffer::WlBuffer;
use crate::wayland::wl_shm::PixelFormat;

#[derive(Interface, Debug)]
pub struct WlShmPool {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    CreateBuffer,
    Destroy,
    Resize,
}

#[derive(Message, Debug)]
#[message(request = WlShmPool)]
pub struct CreateBuffer {
    pub wl_buffer: NewId<WlBuffer>,
    pub offset: u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
}

#[derive(Message, Debug)]
#[message(request = WlShmPool, destructor)]
pub struct Destroy;

#[derive(Message, Debug)]
#[message(request = WlShmPool)]
pub struct Resize {
    pub size: u32,
}
