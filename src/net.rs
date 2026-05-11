use std::io::Result;
use std::os::fd::{AsRawFd, RawFd};
use std::task::{Poll, ready};
use std::{io, ptr};

use crate::macros::syscall;

#[derive(Debug)]
pub struct Connection {
    fd: i32,
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>,
    send_fds: Vec<RawFd>,
    recv_fds: Vec<RawFd>,
    cmsg_buffer: Vec<u8>,
}

impl Drop for Connection {
    fn drop(&mut self) {
        if let Err(err) = syscall!(close, self.fd) {
            eprintln!("cannot close socket: {err}");
        }
    }
}

impl Connection {
    pub fn from_fd(fd: i32) -> Self {
        Self {
            fd,
            read_buffer: Vec::with_capacity(1024),
            write_buffer: Vec::with_capacity(1024),
            send_fds: Vec::with_capacity(8),
            recv_fds: Vec::with_capacity(8),
            cmsg_buffer: Vec::with_capacity(512),
        }
    }

    pub fn read_buffer(&self) -> &[u8] {
        &self.read_buffer
    }
}

impl AsRawFd for Connection {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Connection {
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
    pub fn poll_read(&mut self) -> Poll<Result<()>> {
        if self.read_buffer.capacity() == self.read_buffer.len() {
            self.read_buffer.reserve(64);
        }
        recvmsg(&mut self.read_buffer, &mut self.recv_fds, &mut self.cmsg_buffer, self.fd)
    }

    /// Flush write buffer.
    pub fn poll_flush(&mut self) -> Poll<Result<()>> {
        ready!(sendmsg(
            &self.write_buffer,
            &self.send_fds,
            &mut self.cmsg_buffer,
            self.fd,
        ))?;
        self.cmsg_buffer.clear();
        self.write_buffer.clear();
        self.send_fds.clear();
        Poll::Ready(Ok(()))
    }
}

// ===== syscall =====

fn sendmsg(buf: &[u8], fds: &[RawFd], cmsg_buffer: &mut Vec<u8>, socket: RawFd) -> Poll<Result<()>> {
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
            iov_base: buf.as_ptr() as *mut c_void,
            iov_len: buf.len(),
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

    let mut rem = buf.len();
    let mut msghdr = msghdr;
    loop {
        let write = match syscall!(sendmsg(socket, &msghdr, 0)) {
            Ok(ok) => ok,
            Err(err) => return match err.kind() {
                io::ErrorKind::WouldBlock => Poll::Pending,
                _ => Poll::Ready(Err(err)),
            },
        };
        if rem == write {
            break;
        }
        if write == 0 {
            return Poll::Ready(Err(std::io::ErrorKind::WriteZero.into()));
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
    Poll::Ready(Ok(()))
}

fn recvmsg(
    buffer: &mut Vec<u8>,
    fds_buffer: &mut Vec<RawFd>,
    cmsg_buffer: &mut Vec<u8>,
    socket: RawFd,
) -> Poll<Result<()>> {
    use libc::{CMSG_DATA, CMSG_FIRSTHDR, CMSG_LEN, CMSG_NXTHDR};
    use libc::{SCM_RIGHTS, SOL_SOCKET, iovec, msghdr};

    // FEAT: better fd buffer management

    let buffer_spare = buffer.spare_capacity_mut();
    let cmsg_spare = {
        const CMSG_SPACE: u32 = unsafe { libc::CMSG_SPACE(size_of::<RawFd>() as u32 * 8) };
        cmsg_buffer.reserve((CMSG_SPACE / 8) as usize);
        let cmsg_spare = cmsg_buffer.spare_capacity_mut();
        unsafe {
            std::slice::from_raw_parts_mut(
                cmsg_spare.as_mut_ptr().cast::<u8>(),
                CMSG_SPACE as usize
            )
        }
    };

    let mut iov = iovec {
        iov_base: buffer_spare.as_mut_ptr().cast(),
        iov_len: buffer_spare.len(),
    };
    let mut msghdr = msghdr {
        msg_name: std::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &mut iov,
        msg_iovlen: 1,
        msg_control: cmsg_spare.as_mut_ptr().cast(),
        msg_controllen: cmsg_spare.len(),
        msg_flags: 0,
    };

    let read = match syscall!(recvmsg(socket, &mut msghdr, 0)) {
        Ok(ok) => ok,
        Err(err) => return match err.kind() {
            io::ErrorKind::WouldBlock => Poll::Pending,
            _ => Poll::Ready(Err(err)),
        },
    };
    if read == 0 {
        return Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into()));
    }

    unsafe {
        let mut cmsg_ptr = CMSG_FIRSTHDR(&msghdr);
        while let Some(cmsg) = cmsg_ptr.as_ref() {
            let (SOL_SOCKET, SCM_RIGHTS) = (cmsg.cmsg_level, cmsg.cmsg_type) else {
                break;
            };

            let nfds = (cmsg.cmsg_len - CMSG_LEN(0) as usize) / size_of::<RawFd>();

            let fds = CMSG_DATA(cmsg).cast::<RawFd>();
            let dst = fds_buffer.spare_capacity_mut().as_mut_ptr().cast();
            fds.copy_to_nonoverlapping(dst, nfds);

            fds_buffer.set_len(fds_buffer.len() + nfds);

            cmsg_ptr = CMSG_NXTHDR(&msghdr, cmsg_ptr);
        }
    }

    unsafe { buffer.set_len(buffer.len() + read) };
    Poll::Ready(Ok(()))
}
