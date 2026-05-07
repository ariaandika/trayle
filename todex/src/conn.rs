use std::collections::VecDeque;
use std::io::Read;
use std::os::fd::{AsFd, AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::task::Poll;
use std::{env, io, ptr};
use tcio::bytes::{Buf, BytesMut};

use crate::Id;
use crate::error::BoxError;
use crate::message::{EncodePayload, Message, MessageHeader};
use crate::wayland::wl_display;

// use crate::objects::{Fixed, Header, Message, ReadBuffer, Request, WriteBuffer};
// use crate::roundup_4;

#[derive(Debug)]
pub struct WaylandSocket {
    io: UnixStream,
    read_buffer: BytesMut,
    write_buffer: Vec<u8>,
    sendmsg_buffer: Vec<u8>,
    send_fds: Vec<RawFd>,
    recv_fds: VecDeque<RawFd>,
}

impl WaylandSocket {
    pub fn connect_default() -> Result<Self, BoxError> {
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "wayland-0".into());
        let wayland_display = env::var("WAYLAND_DISPLAY")?;

        let path = format!("{xdg_runtime_dir}/{wayland_display}");
        let io = UnixStream::connect(path)?;

        Ok(Self {
            io,
            read_buffer: BytesMut::with_capacity(1024),
            write_buffer: Vec::with_capacity(1024),
            sendmsg_buffer: Vec::with_capacity(512),
            send_fds: vec![],
            recv_fds: VecDeque::new(),
        })
    }

    /// Request are buffered.
    pub fn send_request<P: EncodePayload>(&mut self, object_id: Id, request: P) {
        let payload_size = request.encoded_size();
        let msg_size = 8 + payload_size;
        self.write_buffer.reserve(msg_size as usize);
        let ptr = self.write_buffer.spare_capacity_mut().as_mut_ptr().cast::<u8>();
        unsafe {
            ptr.cast::<u32>().write(object_id.as_u32());
            ptr.add(4).cast::<u16>().write(P::OPCODE);
            ptr.add(6).cast::<u16>().write(msg_size);
            request.encode_raw(ptr.add(8));
            self.write_buffer.set_len(self.write_buffer.len() + msg_size as usize);
        }
    }

    pub fn flush(&mut self) -> Result<(), BoxError> {
        // self.io.write_all(&self.write_buffer)?;
        sendmsg(
            self.io.as_raw_fd(),
            &mut self.sendmsg_buffer,
            &self.write_buffer,
            &self.send_fds,
        )?;

        self.sendmsg_buffer.clear();
        self.write_buffer.clear();
        self.send_fds.clear();
        Ok(())
    }

    pub fn poll_message(&mut self) -> Result<Option<Message>, BoxError> {
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

        let header = Message::new(header.as_ptr());

        if self.read_buffer.len() < header.len() as usize {
            return Poll::Pending;
        }
        self.read_buffer.advance(header.len() as usize);
        Poll::Ready(header)
    }

    fn read_io(&mut self) -> Result<usize, BoxError> {
        if self.read_buffer.capacity() == self.read_buffer.len() {
            self.read_buffer.reserve(64);
        }
        let spare = self.read_buffer.spare_capacity_mut();
        let spare = unsafe {
            std::slice::from_raw_parts_mut(spare.as_mut_ptr().cast::<u8>(), spare.len())
        };
        let read = self.io.read(spare)?;
        unsafe { self.read_buffer.set_len(self.read_buffer.len() + read) };
        Ok(read)
    }
}

// ===== Helpers =====

/// Send message with file descriptor as ancillary data
fn sendmsg(socket_fd: RawFd, buffer: &mut Vec<u8>, msg: &[u8], fds: &[RawFd]) -> io::Result<()> {
    use libc::{CMSG_DATA, CMSG_FIRSTHDR, CMSG_LEN, CMSG_SPACE};
    use libc::{SCM_RIGHTS, SOL_SOCKET, c_void, iovec, msghdr};

    let (buf_ptr, buf_len) = if fds.is_empty() {
        (std::ptr::null_mut(), 0)
    } else {
        let cmsg_size = unsafe { CMSG_SPACE(size_of_val(fds) as u32) };
        buffer.reserve(cmsg_size as usize);
        (buffer.spare_capacity_mut().as_mut_ptr().cast(), cmsg_size as usize)
    };

    println!("{msg:?}");
    let mut iov = iovec {
        iov_base: msg.as_ptr() as *mut c_void,
        iov_len: msg.len(),
    };
    let msghdr = msghdr {
        msg_name: std::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &mut iov,
        msg_iovlen: 1,
        msg_control: buf_ptr,
        msg_controllen: buf_len,
        msg_flags: 0,
    };

    unsafe {
        if !fds.is_empty() {
            let cmsg_size = CMSG_SPACE(size_of_val(fds) as u32);
            let cmsg = CMSG_FIRSTHDR(&msghdr);
            let cmsg = cmsg.as_mut().expect("CMSG_FIRSTHDR returns null pointer");
            cmsg.cmsg_level = SOL_SOCKET;
            cmsg.cmsg_type = SCM_RIGHTS;
            cmsg.cmsg_len = CMSG_LEN(cmsg_size) as usize;

            let ptr = CMSG_DATA(cmsg).cast::<RawFd>();
            ptr::copy_nonoverlapping(fds.as_ptr(), ptr, fds.len());
        }

        let ok = libc::sendmsg(socket_fd, &msghdr, 0);
        if ok == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

