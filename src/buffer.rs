use todex::sys::{buffer::Buffer, cmsg::Cmsg};

pub struct BufferPool {
    pub read_buf: Buffer,
    pub read_fd: Cmsg,
    pub write_buf: Buffer,
    pub write_fd: Cmsg,
}

impl BufferPool {
    pub fn new() -> Self {
        Self {
            read_buf: Buffer::new(),
            read_fd: Cmsg::new(),
            write_buf: Buffer::new(),
            write_fd: Cmsg::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.read_buf.is_empty() && self.write_buf.is_empty()
    }

    /// Store pending buffer if any for associated id.
    pub fn store_pending(&mut self, _id: usize) -> bool {
        if !self.read_buf.is_empty() || !self.write_buf.is_empty() {
            // TODO: associated array data structure
            true
        } else {
            false
        }
    }

    /// Restore pending buffer if any for associated id.
    pub fn restore_pending(&mut self, _id: usize) {
        // TODO: associated array data structure
    }

    pub fn clear(&mut self) {
        self.read_buf.clear();
        self.write_buf.clear();
    }
}
