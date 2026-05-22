use std::mem::MaybeUninit;

use crate::ptr::Ptr;

// ===== Buffer =====

pub struct Buffer {
    ptr: Ptr<u8>,
    off: u32,
    len: u32,
    cap: u32,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.ptr.sub_mut(self.off);
        self.ptr.deallocate(self.cap + self.off);
    }
}

impl Buffer {
    pub fn with_capacity(cap: u32) -> Self {
        Self {
            ptr: Ptr::with_capacity(cap),
            off: 0,
            len: 0,
            cap,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.ptr.as_slice(self.len)
    }

    pub fn advance(&mut self, cnt: u32) {
        debug_assert!(cnt <= self.len);
        self.ptr = self.ptr.add(cnt);
        self.off += cnt;
        self.len -= cnt;
        self.cap -= cnt;
    }

    /// # Safety
    ///
    /// `cnt` element after the last element must be initialized.
    pub unsafe fn advance_mut(&mut self, cnt: u32) {
        debug_assert!(self.cap - self.len >= cnt);
        self.len += cnt;
    }

    /// Returns `true` if remaining capacity is sufficient and the data is copied.
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        if self.cap - self.len < slice.len() as u32 {
            self.ptr.grow(self.cap, self.cap + slice.len() as u32);
        }
        unsafe {
            self.spare_capacity_mut()
                .as_mut_ptr()
                .copy_from_nonoverlapping(slice.as_ptr().cast(), slice.len());
            self.advance_mut(slice.len() as u32);
        }
    }

    pub fn reserve(&mut self, len: u32) {
        if self.cap - self.len < len {
            self.cap = self.ptr.grow(self.cap, self.cap + len);
        }
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.ptr
            .cast()
            .add(self.len)
            .as_mut_slice(self.cap - self.len)
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.cap += self.off;
        self.ptr.sub_mut(self.off);
        self.off = 0;
    }
}

impl std::ops::Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

// ===== SmallBuf =====

const INT: u32 = 4;
const HEADER: u32 = const { INT * 3 };

pub struct SmallBuf {
    ptr: *mut u8,
}

impl Drop for SmallBuf {
    fn drop(&mut self) {
        if let Some(ptr) = Ptr::new(self.ptr) {
            let len = u32::from_ne_bytes(ptr.cast().read());
            ptr.deallocate(len);
        }
    }
}

impl Default for SmallBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl SmallBuf {
    pub const fn new() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
        }
    }

    pub fn copy_from(&mut self, read_buf: &mut Buffer, write_buf: &mut Buffer) {
        debug_assert!(self.ptr.is_null());
        debug_assert!(!(read_buf.is_empty() & write_buf.is_empty()));

        let read_len = read_buf.len() as u32;
        let write_len = write_buf.len() as u32;
        let cap = HEADER + read_len + write_len;
        let ptr = Ptr::allocate(cap);

        ptr.cast().write(cap.to_ne_bytes());
        ptr.add(INT).cast().write(read_len.to_ne_bytes());
        ptr.add(INT + INT).cast().write(write_len.to_ne_bytes());

        ptr.add(HEADER).copy_from_nonoverlapping(read_buf.as_ptr(), read_len);
        ptr.add(HEADER + read_len).copy_from_nonoverlapping(write_buf.as_ptr(), write_len);

        read_buf.clear();
        write_buf.clear();
        self.ptr = ptr.as_ptr();
    }

    pub fn copy_to(&mut self, read_buf: &mut Buffer, write_buf: &mut Buffer) {
        let Some(ptr) = Ptr::new(self.ptr) else {
            return;
        };

        let read_len = u32::from_ne_bytes(ptr.cast().add(INT).read());
        let read_rem = unsafe {
            std::slice::from_raw_parts(self.ptr.add(HEADER as usize), read_len as usize)
        };
        read_buf.extend_from_slice(read_rem);

        let write_len = u32::from_ne_bytes(ptr.cast().add(INT + INT).read());
        let write_rem = unsafe {
            std::slice::from_raw_parts(self.ptr.add((HEADER + read_len) as usize), write_len as usize)
        };
        write_buf.extend_from_slice(write_rem);

        drop(std::mem::take(self));
    }
}
