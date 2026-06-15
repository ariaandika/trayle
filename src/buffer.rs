use std::mem;
use todex::sys::buffer::Buffer;
use todex::sys::cmsg::Cmsg;

struct Entry {
    id: usize,
    read: Buffer,
    write: Buffer,
}

pub struct BufferPool {
    pub read_buf: Buffer,
    pub read_fd: Cmsg,
    pub write_buf: Buffer,
    pub write_fd: Cmsg,
    pendings: Vec<Entry>,
}

impl BufferPool {
    pub fn new() -> Self {
        Self {
            read_buf: Buffer::new(),
            read_fd: Cmsg::new(),
            write_buf: Buffer::new(),
            write_fd: Cmsg::new(),
            pendings: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.read_buf.is_empty() && self.write_buf.is_empty()
    }

    /// Store pending buffer with given id if any.
    ///
    /// Returns `Some((read_len, write_len))` if there is in fact pending buffer.
    pub fn store_pending(&mut self, id: usize) -> Option<(usize, usize)> {
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
    pub fn restore_pending(&mut self, id: usize) {
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

fn take_if_not_empty(buffer: &mut Buffer) -> Buffer {
    if buffer.is_empty() {
        Buffer::new()
    } else {
        mem::replace(buffer, Buffer::new())
    }
}

fn swap_if_not_empty(buffer: &mut Buffer, dst: &mut Buffer) {
    if !buffer.is_empty() {
        mem::swap(buffer, dst);
    }
}
