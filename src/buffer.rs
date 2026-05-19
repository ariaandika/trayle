use std::mem::MaybeUninit;

use crate::ptr::Ptr;

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
