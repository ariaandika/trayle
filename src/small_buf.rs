use std::ptr::NonNull;
use std::slice;

use crate::alloc;
use crate::buffer::Buffer;

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
