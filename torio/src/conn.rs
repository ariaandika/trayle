use std::io::Read;
use std::os::fd::{AsFd, AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::task::Poll;
use std::{env, io, ptr};
use tcio::bytes::BytesMut;

use crate::objects::{Buffer, Fixed, Header, Message, Request};
use crate::roundup_4;

#[derive(Debug)]
pub struct WaylandSocket {
    io: UnixStream,
    read_buffer: BytesMut,
    write_buffer: Vec<u8>,
    sendmsg_buffer: Vec<u8>,
    fds: Vec<RawFd>,
}

impl Buffer for (&mut Vec<u8>, &mut Vec<RawFd>) {
    fn put_int(&mut self, int: i32) {
        self.0.extend_from_slice(&int.to_ne_bytes());
    }

    fn put_uint(&mut self, uint: u32) {
        self.0.extend_from_slice(&uint.to_ne_bytes());
    }

    fn put_fixed(&mut self, fixed: Fixed) {
        self.put_int(fixed.to_raw());
    }

    fn put_string(&mut self, string: &str) {
        self.0.extend_from_slice(&roundup_4!(string.len() + 1).to_ne_bytes());
        self.0.extend_from_slice(string.as_bytes());
        self.0.extend_from_slice(b"\0");
    }

    fn put_new_id(&mut self, interface: &str, version: u32, new_id: u32) {
        self.put_string(interface);
        self.put_uint(version);
        self.put_uint(new_id);
    }

    fn put_array<T>(&mut self, _array: &[T]) {
        todo!()
    }

    fn put_fd<Fd: AsFd>(&mut self, fd: Fd) {
        self.1.push(fd.as_fd().as_raw_fd());
    }
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
            write_buffer: Vec::with_capacity(1024),
            sendmsg_buffer: Vec::with_capacity(512),
            fds: vec![],
        })
    }

    pub fn send_request<R: Request>(&mut self, request: R) -> io::Result<()> {
        let offset = u16::try_from(self.write_buffer.len()).unwrap();

        self.write_buffer.extend_from_slice(&request.object_id().to_ne_bytes());
        self.write_buffer.extend_from_slice(&R::OP_CODE.to_ne_bytes());

        // spare 2 bytes for later write
        self.write_buffer.extend_from_slice(&0u16.to_ne_bytes());

        request.write_body(&mut (&mut self.write_buffer, &mut self.fds));
        let len = u16::try_from(self.write_buffer.len()).unwrap().strict_sub(offset);
        self.write_buffer[6..8].copy_from_slice(&len.to_ne_bytes());

        Ok(())
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        // self.io.write_all(&self.write_buffer)?;
        sendmsg(
            self.io.as_raw_fd(),
            &mut self.sendmsg_buffer,
            &self.write_buffer,
            &self.fds,
        )?;

        self.sendmsg_buffer.clear();
        self.write_buffer.clear();
        self.fds.clear();
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
