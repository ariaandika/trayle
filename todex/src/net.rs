use std::collections::VecDeque;
use std::io::Read;
use std::io::Result;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::{io, ptr};
use tcio::bytes::BytesMut;

#[derive(Debug)]
pub struct Socket {
    io: UnixStream,
    read_buffer: BytesMut,
    write_buffer: Vec<u8>,
    sendmsg_buffer: Vec<u8>,
    send_fds: Vec<RawFd>,
    recv_fds: VecDeque<RawFd>,
}

impl Socket {
    pub fn new(io: UnixStream) -> Self {
        Self {
            io,
            read_buffer: BytesMut::with_capacity(1024),
            write_buffer: Vec::with_capacity(1024),
            sendmsg_buffer: Vec::with_capacity(512),
            send_fds: vec![],
            recv_fds: VecDeque::new(),
        }
    }

    pub fn read_buffer(&self) -> &BytesMut {
        &self.read_buffer
    }

    pub fn read_buffer_mut(&mut self) -> &mut BytesMut {
        &mut self.read_buffer
    }
}

impl Socket {
    /// Returns writable memory until `len`.
    ///
    /// # Safety
    ///
    /// Caller must ensure `len` data from the returned pointer are initialized.
    pub unsafe fn spare(&mut self, len: usize) -> *mut u8 {
        self.write_buffer.reserve(len);
        let ptr = self.write_buffer.spare_capacity_mut().as_mut_ptr().cast();
        // SAFETY: caller ensure that `len` additional data is initialized
        unsafe { self.write_buffer.set_len(self.write_buffer.len() + len) };
        ptr
    }

    /// Read data to the read buffer.
    pub fn read(&mut self) -> Result<()> {
        if self.read_buffer.capacity() == self.read_buffer.len() {
            self.read_buffer.reserve(64);
        }
        let spare = self.read_buffer.spare_capacity_mut();
        let spare =
            unsafe { std::slice::from_raw_parts_mut(spare.as_mut_ptr().cast::<u8>(), spare.len()) };
        let read = self.io.read(spare)?;
        if read == 0 {
            return Err(io::ErrorKind::ConnectionAborted.into());
        }
        unsafe { self.read_buffer.set_len(self.read_buffer.len() + read) };
        Ok(())
    }

    /// Flush write buffer.
    pub fn flush(&mut self) -> Result<()> {
        let write = sendmsg(
            self.io.as_raw_fd(),
            &mut self.sendmsg_buffer,
            &self.write_buffer,
            &self.send_fds,
        )?;
        if write != self.write_buffer.len() {
            todo!("partial write")
        }
        self.sendmsg_buffer.clear();
        self.write_buffer.clear();
        self.send_fds.clear();
        Ok(())
    }
}

fn sendmsg(socket_fd: RawFd, buffer: &mut Vec<u8>, msg: &[u8], fds: &[RawFd]) -> Result<usize> {
    use libc::{CMSG_DATA, CMSG_FIRSTHDR, CMSG_LEN, CMSG_SPACE};
    use libc::{SCM_RIGHTS, SOL_SOCKET, c_void, iovec, msghdr};

    let (buf_ptr, buf_len) = if fds.is_empty() {
        (std::ptr::null_mut(), 0)
    } else {
        let cmsg_size = unsafe { CMSG_SPACE(size_of_val(fds) as u32) };
        buffer.reserve(cmsg_size as usize);
        (
            buffer.spare_capacity_mut().as_mut_ptr().cast(),
            cmsg_size as usize,
        )
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

        let result = libc::sendmsg(socket_fd, &msghdr, 0);
        match usize::try_from(result) {
            Ok(write) => Ok(write),
            Err(_) => Err(std::io::Error::last_os_error()),
        }
    }
}
