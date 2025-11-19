use std::io::Write;
use std::os::unix::net::UnixStream;
use std::{env, io};
use tcio::bytes::BytesMut;

use super::objects::Request;

#[derive(Debug)]
pub struct WaylandSocket {
    io: UnixStream,
    write_buffer: BytesMut,
}

impl WaylandSocket {
    pub fn connect_default() -> anyhow::Result<Self> {
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "wayland-0".into());
        let wayland_display = env::var("WAYLAND_DISPLAY")?;

        let path = format!("{xdg_runtime_dir}/{wayland_display}");
        let io = UnixStream::connect(path)?;

        let write_buffer = BytesMut::with_capacity(1024);

        Ok(Self { io, write_buffer })
    }

    pub fn send_request<R: Request>(&mut self, request: R) -> io::Result<()> {
        let offset = u16::try_from(self.write_buffer.len()).unwrap();

        self.write_buffer.extend_from_slice(&request.object_id().to_ne_bytes());
        self.write_buffer.extend_from_slice(&R::OP_CODE.to_ne_bytes());

        // spare 2 bytes for later write
        self.write_buffer.extend_from_slice(&0u16.to_ne_bytes());

        request.write_body(&mut self.write_buffer);
        let len = u16::try_from(self.write_buffer.len()).unwrap().strict_sub(offset);
        self.write_buffer[6..8].copy_from_slice(&len.to_ne_bytes());

        Ok(())
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        self.io.write_all(&self.write_buffer)?;
        self.write_buffer.clear();
        Ok(())
    }
}


