use std::os::fd::{AsRawFd, RawFd};

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

    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }
}

impl AsRawFd for Client {
    fn as_raw_fd(&self) -> RawFd {
        self.conn.as_raw_fd()
    }
}
