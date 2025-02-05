use crate::Trayle;
use smithay::{
    reexports::wayland_server::protocol::wl_buffer::WlBuffer, wayland::buffer::BufferHandler,
};

impl BufferHandler for Trayle {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) { }
}

