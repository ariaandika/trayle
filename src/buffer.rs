use std::mem;
use todex::bytes::{Bytes, Cmsg};

struct Entry {
    id: u64,
    read: Bytes,
    write: Bytes,
}

const INIT_BUF_CAP: usize = 512;

pub struct BufferPool {
    pub read_buf: Bytes,
    pub read_fd: Cmsg,
    pub write_buf: Bytes,
    pub write_fd: Cmsg,
    pendings: Vec<Entry>,
}

impl BufferPool {
    pub fn new() -> Self {
        Self {
            read_buf: Bytes::with_capacity(INIT_BUF_CAP),
            read_fd: Cmsg::new(),
            write_buf: Bytes::with_capacity(INIT_BUF_CAP),
            write_fd: Cmsg::new(),
            pendings: Vec::new(),
        }
    }

    /// Store pending buffer with given id if any.
    ///
    /// Returns `Some((read_len, write_len))` if there is in fact pending buffer.
    pub fn store_pending(&mut self, id: u64) -> Option<(usize, usize)> {
        if self.read_buf.is_empty() && self.write_buf.is_empty() {
            return None;
        }
        let rl = self.read_buf.len();
        let wl = self.read_buf.len();
        self.pendings.push(Entry {
            id,
            read: take_if_not_empty(&mut self.read_buf),
            write: take_if_not_empty(&mut self.write_buf),
        });
        Some((rl, wl))

    }

    /// Restore pending buffer for given id if any.
    ///
    /// Note that this is expected to be rare case. It must be flagged externally that given id does
    /// infact have a pending buffer.
    pub fn restore_pending(&mut self, id: u64) {
        let Some(idx) = self.pendings.iter().position(|e| e.id == id) else {
            return;
        };
        let mut pending = self.pendings.swap_remove(idx);
        swap_if_not_empty(&mut pending.read, &mut self.read_buf);
        swap_if_not_empty(&mut pending.write, &mut self.write_buf);
        // should one store this swapped buffer ?
    }

    pub fn clear(&mut self) {
        self.read_buf.clear();
        self.read_fd.clear();
        self.write_buf.clear();
        self.write_fd.clear();
    }
}

fn take_if_not_empty(buffer: &mut Bytes) -> Bytes {
    if buffer.is_empty() {
        Bytes::with_capacity(INIT_BUF_CAP)
    } else {
        mem::replace(buffer, Bytes::with_capacity(INIT_BUF_CAP))
    }
}

fn swap_if_not_empty(buffer: &mut Bytes, dst: &mut Bytes) {
    if !buffer.is_empty() {
        mem::swap(buffer, dst);
    }
}
