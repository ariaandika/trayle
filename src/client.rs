use std::io;
use std::os::fd::{AsRawFd, RawFd};
use std::task::Poll;

use crate::buffer::Buffer;
use crate::conn::Connection;

pub struct Client {
    id: u64,
    conn: Connection,
}

impl Client {
    pub const fn new(id: u64, conn: Connection) -> Self {
        Self { id, conn }
    }

    pub const fn id(&self) -> u64 {
        self.id
    }
}

impl AsRawFd for Client {
    fn as_raw_fd(&self) -> RawFd {
        self.conn.as_raw_fd()
    }
}

// ===== delegate Connection =====

impl Client {
    pub fn poll_read(&self, buffer: &mut Buffer, fds: &mut Buffer) -> Poll<io::Result<()>> {
        self.conn.poll_read(buffer, fds)
    }
}

