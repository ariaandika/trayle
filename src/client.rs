use std::os::unix::net::UnixStream;

use crate::net::Socket;
use crate::wayland::{Id, WlDisplay, Write};

#[derive(Debug)]
pub struct Client {
    socket: Socket,
    wl_display: WlDisplay,
}

impl Client {
    pub fn new(stream: UnixStream) -> Self {
        Self {
            socket: Socket::new(stream),
            wl_display: WlDisplay::new(),
        }
    }

    /// Returns current message object id and opcode.
    ///
    /// This does not consume the message.
    pub fn peek_message(&mut self) -> Option<(Id, u16)> {
        if self.socket.read_buffer().len() < 8 {
            return None;
        }
        let ptr = self.socket.read_buffer().as_ptr();
        unsafe { Some((*ptr.cast::<Id>(), *ptr.add(4).cast())) }
    }

    pub fn error(&mut self, object_id: Id, code: u32, message: &str) {
        struct Wrapper<'a>(&'a mut Socket);
        unsafe impl Write for Wrapper<'_> {
            unsafe fn spare(&mut self, len: usize) -> *mut u8 {
                unsafe { self.0.spare(len) }
            }
        }
        self.wl_display
            .error(object_id, code, message, Wrapper(&mut self.socket));
    }

    pub fn read(&mut self) -> std::io::Result<()> {
        self.socket.read()
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.socket.flush()
    }
}
