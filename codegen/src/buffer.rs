use std::fs::File;
use std::ptr::{self, NonNull};
use std::{io, slice, mem};

const MIN_READ: usize = 1024 * 2;

pub struct FileBuffer {
    file: File,
    base: NonNull<u8>,
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
}

impl Drop for FileBuffer {
    fn drop(&mut self) {
        unsafe { Vec::from_raw_parts(self.base.as_ptr(), 0, self.cap) };
    }
}

impl FileBuffer {
    pub fn new(file: File) -> Self {
        let (ptr, len, cap) = Vec::with_capacity(1024 * 8).into_raw_parts();
        let ptr = NonNull::new(ptr).expect("vec is non-null");
        Self {
            file,
            base: ptr,
            ptr,
            len,
            cap,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn advance(&mut self, cnt: usize) {
        self.len = self.len.strict_sub(cnt);
        self.ptr = unsafe { self.ptr.add(cnt) };
    }

    pub fn read(&mut self) {
        // try copy backward
        let mut head_spare = unsafe { self.ptr.offset_from_unsigned(self.base) };
        if self.len <= head_spare {
            unsafe { ptr::copy_nonoverlapping(self.ptr.as_ptr(), self.base.as_ptr(), self.len) };
            self.ptr = self.base;
            head_spare = 0;
        }

        let tail_spare = self.cap - (head_spare + self.len);
        if tail_spare < MIN_READ {
            self.reserve();
        }

        unsafe {
            let head_spare = self.ptr.offset_from_unsigned(self.base);
            let tail_spare = self.cap - (head_spare + self.len);

            let spare = slice::from_raw_parts_mut(self.ptr.add(self.len).as_ptr(), tail_spare);
            let read = io::Read::read(&mut self.file, spare).expect("cannot read file");
            if read == 0 {
                panic!("unexpected EOF, len: {}, cap: {}, offset: {}", self.len, self.cap, head_spare);
            }

            self.len += read;
        }
    }

    fn reserve(&mut self) {
        unsafe {
            let offset = self.ptr.offset_from_unsigned(self.base);
            let mut vec = mem::ManuallyDrop::new(Vec::from_raw_parts(
                self.base.as_ptr(),
                offset + self.len,
                self.cap,
            ));
            vec.reserve(MIN_READ);
            self.base = NonNull::new_unchecked(vec.as_mut_ptr());
            self.ptr = self.base;
            self.cap = vec.capacity();
        }
    }
}
