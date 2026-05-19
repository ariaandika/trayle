use std::os::fd::{AsRawFd, RawFd};

use crate::{conn::Connection, objects::Objects};

pub struct Client {
    id: u64,
    conn: Connection,
    objects: Objects,
}

impl Client {
    pub fn new(id: u64, conn: Connection) -> Self {
        Self {
            id,
            conn,
            objects: Objects::new(),
        }
    }

    pub const fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }

    pub(crate) fn objects_mut(&mut self) -> &mut Objects {
        &mut self.objects
    }
}

impl AsRawFd for Client {
    fn as_raw_fd(&self) -> RawFd {
        self.conn.as_raw_fd()
    }
}
