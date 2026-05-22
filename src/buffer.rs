use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::slice;

use crate::alloc;
// use crate::ptr::Ptr;

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

// ===== SmallBuf =====

const INT: usize = size_of::<u32>();
const HEADER: usize = const { INT * 3 };

pub struct SmallBuf {
    ptr: *mut u8,
}

impl Drop for SmallBuf {
    fn drop(&mut self) {
        if let Some(ptr) = NonNull::new(self.ptr) {
            let cap = unsafe { ptr.cast().read_unaligned() };
            alloc::deallocate(ptr, cap);
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
        let cap = HEADER as u32 + read_len + write_len;
        let ptr = alloc::allocate::<u8>(cap).as_ptr();

        unsafe {
            let hptr = ptr.cast::<u32>();
            hptr.write_unaligned(cap);
            hptr.add(1).write_unaligned(read_len);
            hptr.add(2).write_unaligned(write_len);

            ptr.add(HEADER)
                .copy_from_nonoverlapping(read_buf.as_ptr(), read_len as usize);
            ptr.add(HEADER + read_len as usize)
                .copy_from_nonoverlapping(write_buf.as_ptr(), write_len as usize);
        }

        read_buf.clear();
        write_buf.clear();
        self.ptr = ptr;
    }

    pub fn copy_to(&mut self, read_buf: &mut Buffer, write_buf: &mut Buffer) {
        let Some(ptr) = NonNull::new(self.ptr) else {
            return;
        };

        unsafe {
            let read_len = ptr.cast::<u32>().add(1).read_unaligned() as usize;
            let read_rem = slice::from_raw_parts(self.ptr.add(HEADER), read_len);
            read_buf.extend_from_slice(read_rem);

            let write_len = ptr.cast::<u32>().add(2).read_unaligned() as usize;
            let write_rem = slice::from_raw_parts(self.ptr.add(HEADER + read_len), write_len);
            write_buf.extend_from_slice(write_rem);
        }

        drop(std::mem::take(self));
    }
}
