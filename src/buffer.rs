use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::slice;

use crate::alloc;

// ===== Buffer =====

pub struct Buffer {
    ptr: NonNull<u8>,
    off: u32,
    len: u32,
    cap: u32,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        alloc::deallocate_offset(self.ptr, self.cap, self.off);
    }
}

impl Buffer {
    pub fn with_capacity(cap: u32) -> Self {
        Self {
            ptr: alloc::allocate(cap),
            off: 0,
            len: 0,
            cap,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len as usize) }
    }

    pub fn advance(&mut self, cnt: u32) {
        assert!(cnt <= self.len);
        self.ptr = unsafe { self.ptr.add(cnt as usize) };
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

    pub fn try_split_to(&mut self, cnt: u32) -> Option<&[u8]> {
        if cnt > self.len {
            return None;
        }
        self.advance(cnt);
        Some(unsafe { slice::from_raw_parts(self.ptr.sub(cnt as usize).as_ptr(), cnt as usize) })
    }

    /// Returns `true` if remaining capacity is sufficient and the data is copied.
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.reserve(slice.len() as u32);
        unsafe {
            self.spare_capacity_mut()
                .as_mut_ptr()
                .copy_from_nonoverlapping(slice.as_ptr().cast(), slice.len());
            self.advance_mut(slice.len() as u32);
        }
    }

    pub fn reserve(&mut self, len: u32) {
        if self.cap - self.len < len {
            self.cap = alloc::grow(&mut self.ptr, self.cap, len);
        }
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            slice::from_raw_parts_mut(
                self.ptr.add(self.len as usize).cast().as_ptr(),
                (self.cap - self.len) as usize,
            )
        }
    }

    pub fn clear(&mut self) {
        self.ptr = unsafe { self.ptr.sub(self.off as usize) };
        self.cap += self.off;
        self.len = 0;
        self.off = 0;
    }
}

impl std::ops::Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
