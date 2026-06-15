use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::slice;

use crate::alloc;

const INITIAL_CAP: usize = 512;

pub struct Bytes {
    ptr: NonNull<u8>,
    off: usize,
    len: usize,
    cap: usize,
}

impl Drop for Bytes {
    fn drop(&mut self) {
        alloc::deallocate(self.ptr);
    }
}

impl Bytes {
    #[inline]
    pub fn new() -> Self {
        Self {
            ptr: alloc::allocate(INITIAL_CAP),
            off: 0,
            len: 0,
            cap: INITIAL_CAP,
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self, additional: usize) {
        let base_ptr = unsafe { self.ptr.sub(self.off) };
        let base_cap = self.cap + self.off;
        let new_cap = alloc::calc_grow(base_cap, additional);
        let new_ptr = alloc::reallocate(base_ptr, new_cap);
        self.ptr = unsafe { new_ptr.add(self.off) };
        self.cap = new_cap - self.off;
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    #[inline]
    pub fn advance(&mut self, cnt: usize) {
        assert!(cnt <= self.len);
        // SAFETY: asserted above
        unsafe { self.advance_unchecked(cnt) };
    }

    /// # Safety
    ///
    /// `cnt <= self.len()`
    #[inline]
    pub unsafe fn advance_unchecked(&mut self, cnt: usize) {
        debug_assert!(cnt <= self.len);
        self.ptr = unsafe { self.ptr.add(cnt) };
        self.off += cnt;
        self.len -= cnt;
        self.cap -= cnt;
    }

    /// # Safety
    ///
    /// `cnt` element after the last element must be initialized.
    #[inline]
    pub unsafe fn advance_mut(&mut self, cnt: usize) {
        debug_assert!(self.cap - self.len >= cnt);
        self.len += cnt;
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        if self.cap - self.len < additional {
            self.grow(additional);
        }
    }

    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            slice::from_raw_parts_mut(
                self.ptr.add(self.len).cast().as_ptr(),
                self.cap - self.len
            )
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.ptr = unsafe { self.ptr.sub(self.off) };
        self.cap += self.off;
        self.len = 0;
        self.off = 0;
    }
}

impl Bytes {
    /// Create [`iovec`] from this buffer.
    ///
    /// Note that the lifetime is not bound. This is intended to be used in a scope.
    ///
    /// [`iovec`]: libc::iovec
    pub(crate) fn iovec(&self) -> libc::iovec {
        libc::iovec {
            iov_base: self.ptr.as_ptr().cast(),
            iov_len: self.len,
        }
    }
}

impl std::ops::Deref for Bytes {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
