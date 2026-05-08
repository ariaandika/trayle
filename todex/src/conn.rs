use std::os::unix::net::UnixStream;
use std::{env, io};

use tcio::bytes::Buf;

use crate::Id;
use crate::error::BoxError;
use crate::message::{EncodePayload, Message};
use crate::net::Socket;

#[derive(Debug)]
pub struct WaylandSocket {
    socket: Socket,
}

impl WaylandSocket {
    pub fn connect_default() -> Result<Self, BoxError> {
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "wayland-0".into());
        let wayland_display = env::var("WAYLAND_DISPLAY")?;

        let path = format!("{xdg_runtime_dir}/{wayland_display}");
        let io = UnixStream::connect(path)?;

        Ok(Self {
            socket: Socket::new(io),
        })
    }
}

impl WaylandSocket {
    /// Request are buffered.
    pub fn send<P: EncodePayload>(&mut self, object_id: Id, message: P) {
        let payload_size = message.encoded_size();
        let msg_size = 8 + payload_size;
        unsafe {
            // SAFETY: ptr are initialized after this call
            let ptr = self.socket.spare(msg_size as usize);
            ptr.cast::<u32>().write(object_id.as_u32());
            ptr.add(4).cast::<u16>().write(P::OPCODE);
            ptr.add(6).cast::<u16>().write(msg_size);
            message.encode_raw(ptr.add(8));
        }
    }

    pub fn poll_message(&mut self) -> Result<Message, BoxError> {
        loop {
            let read_buffer = self.socket.read_buffer();
            let Some(header) = read_buffer.first_chunk::<8>() else {
                self.socket.read()?;
                continue;
            };

            let header = Message::new(header.as_ptr());

            if read_buffer.len() < header.len() as usize {
                self.socket.read()?;
                continue;
            }
            self.socket.read_buffer_mut().advance(header.len() as usize);
            break Ok(header);
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.socket.flush()
    }
}
