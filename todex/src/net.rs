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
    /// Returns writable `len` sized memory.
    ///
    /// # Safety
    ///
    /// Caller must initialize `len` data from the returned pointer.
    pub unsafe fn spare(&mut self, len: usize) -> *mut u8 {
        self.write_buffer.reserve(len);
        let ptr = self.write_buffer.spare_capacity_mut().as_mut_ptr().cast();
        // SAFETY: caller ensure that `len` additional data will be initialized
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
        sendmsg(
            &self.write_buffer,
            &self.send_fds,
            &mut self.sendmsg_buffer,
            self.io.as_raw_fd(),
        )?;
        self.sendmsg_buffer.clear();
        self.write_buffer.clear();
        self.send_fds.clear();
        Ok(())
    }
}

// ===== syscall =====

fn sendmsg(msg: &[u8], fds: &[RawFd], cmsg_buffer: &mut Vec<u8>, socket: RawFd,) -> Result<()> {
    use libc::{CMSG_DATA, CMSG_FIRSTHDR, CMSG_LEN, CMSG_SPACE};
    use libc::{SCM_RIGHTS, SOL_SOCKET, c_void, iovec, msghdr};

    let (cmsg_ptr, cmsg_len) = if fds.is_empty() {
        (ptr::null_mut(), 0)
    } else {
        let fd_size = size_of_val(fds) as u32;

        // CMSG_SPACE used when calculating required allocation of ancillary data
        let cmsg_space = unsafe { CMSG_SPACE(fd_size) };
        cmsg_buffer.reserve(cmsg_space as usize);

        // CMSG_LEN used when calculating exact length of ancillary data
        let cmsg_len = unsafe { CMSG_LEN(fd_size) };
        let cmsg_ptr = cmsg_buffer.spare_capacity_mut().as_mut_ptr().cast();
        (cmsg_ptr, cmsg_len)
    };

    // https://linux.die.net/man/3/cmsg

    let msghdr = msghdr {
        msg_name: std::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &mut iovec {
            iov_base: msg.as_ptr() as *mut c_void,
            iov_len: msg.len(),
        },
        msg_iovlen: 1,
        msg_control: cmsg_ptr,
        msg_controllen: cmsg_len as usize,
        msg_flags: 0,
    };

    if !fds.is_empty() {
        unsafe {
            let cmsg = &mut *CMSG_FIRSTHDR(&msghdr);
            cmsg.cmsg_len = cmsg_len as usize;
            cmsg.cmsg_level = SOL_SOCKET;
            cmsg.cmsg_type = SCM_RIGHTS;

            // initialize the payload
            let fdptr = CMSG_DATA(cmsg).cast::<RawFd>();
            fdptr.copy_from_nonoverlapping(fds.as_ptr(), fds.len());
        }
    }

    let mut rem = msg.len();
    let mut msghdr = msghdr;
    loop {
        let result = unsafe { libc::sendmsg(socket, &msghdr, 0) };
        let Ok(write) = usize::try_from(result) else {
            return Err(std::io::Error::last_os_error());
        };
        if rem == write {
            break;
        }
        if write == 0 {
            return Err(std::io::ErrorKind::WriteZero.into());
        }
        rem -= write;

        unsafe {
            // `advance` the message buffer
            let iov_mut = &mut *msghdr.msg_iov;
            iov_mut.iov_base = iov_mut.iov_base.add(write);
            iov_mut.iov_len = rem;

            // Ancillary data is received as if it were queued along with the first normal data octet in
            // the segment (if any).
            //
            // - https://unix.stackexchange.com/questions/185011/what-happens-with-unix-stream-ancillary-data-on-partial-reads
            //
            // unset the ancillary data
            msghdr.msg_control = ptr::null_mut();
            msghdr.msg_controllen = 0;
        }
    }
    Ok(())
}
