use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::task::Poll;
use std::{env, io};
use tcio::bytes::BytesMut;

use super::objects::{Request, Header, Message};

#[derive(Debug)]
pub struct WaylandSocket {
    io: UnixStream,
    read_buffer: BytesMut,
    write_buffer: BytesMut,
}

impl WaylandSocket {
    pub fn connect_default() -> anyhow::Result<Self> {
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "wayland-0".into());
        let wayland_display = env::var("WAYLAND_DISPLAY")?;

        let path = format!("{xdg_runtime_dir}/{wayland_display}");
        let io = UnixStream::connect(path)?;

        Ok(Self {
            io,
            read_buffer: BytesMut::with_capacity(1024),
            write_buffer: BytesMut::with_capacity(1024),
        })
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

    pub fn poll_message(&mut self) -> anyhow::Result<Option<Message>> {
        if self.read_buffer.is_empty() {
            let read = self.read_io()?;
            if read == 0 {
                return Ok(None);
            }
        }

        match self.poll_message_inner() {
            Poll::Ready(message) => Ok(Some(message)),
            Poll::Pending => {
                let read = self.read_io()?;
                if read == 0 {
                    return Ok(None);
                }
                self.poll_message()
            }
        }
    }

    fn poll_message_inner(&mut self) -> Poll<Message> {
        let Some(header) = self.read_buffer.first_chunk::<8>() else {
            return Poll::Pending;
        };

        let len = Header::len_of(header);

        if self.read_buffer.len() < len {
            return Poll::Pending;
        }

        Poll::Ready(Message::new(self.read_buffer.split_to(len)))
    }

    fn read_io(&mut self) -> anyhow::Result<usize> {
        if self.read_buffer.capacity() == self.read_buffer.len() {
            self.read_buffer.reserve(64);
        }
        let spare = self.read_buffer.spare_capacity_mut();
        let spare = unsafe {
            std::slice::from_raw_parts_mut(spare.as_mut_ptr().cast::<u8>(), spare.len())
        };
        let read = self.io.read(spare)?;
        unsafe {
            self.read_buffer.set_len(self.read_buffer.len() + read);
        }
        Ok(read)
    }
}

