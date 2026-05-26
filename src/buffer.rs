use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::slice;

use crate::alloc;

// ===== Buffer =====

pub struct Buffer {
    ptr: NonNull<u8>,
    off: usize,
    len: usize,
    cap: usize,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let base_ptr = unsafe { self.ptr.sub(self.off) };
        alloc::deallocate(base_ptr);
    }
}

impl Buffer {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            ptr: alloc::allocate(cap),
            off: 0,
            len: 0,
            cap,
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self, additional: usize) {
        let new_cap = alloc::calc_grow(self.cap, additional);
        unsafe {
            let base_ptr = self.ptr.sub(self.off);
            self.ptr = alloc::reallocate(base_ptr, new_cap).add(self.off);
            self.cap = new_cap;
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn advance(&mut self, cnt: usize) {
        assert!(cnt <= self.len);
        self.ptr = unsafe { self.ptr.add(cnt) };
        self.off += cnt;
        self.len -= cnt;
        self.cap -= cnt;
    }

    /// # Safety
    ///
    /// `cnt` element after the last element must be initialized.
    pub unsafe fn advance_mut(&mut self, cnt: usize) {
        debug_assert!(self.cap - self.len >= cnt);
        self.len += cnt;
    }

    pub fn try_split_to(&mut self, cnt: usize) -> Option<&[u8]> {
        if cnt > self.len {
            return None;
        }
        self.advance(cnt);
        Some(unsafe { slice::from_raw_parts(self.ptr.sub(cnt).as_ptr(), cnt) })
    }

    /// Returns `true` if remaining capacity is sufficient and the data is copied.
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.reserve(slice.len());
        unsafe {
            self.spare_capacity_mut()
                .as_mut_ptr()
                .copy_from_nonoverlapping(slice.as_ptr().cast(), slice.len());
            self.advance_mut(slice.len());
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        if self.cap - self.len < additional {
            self.grow(additional);
        }
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            slice::from_raw_parts_mut(
                self.ptr.add(self.len).cast().as_ptr(),
                self.cap - self.len
            )
        }
    }

    pub fn clear(&mut self) {
        self.ptr = unsafe { self.ptr.sub(self.off) };
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
