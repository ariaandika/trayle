use std::fs::File;
use std::ptr::NonNull;
use std::rc::Rc;
use std::{io, slice};

pub struct FileBuffer {
    shared: Rc<Shared>,
    file: File,
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
}

#[derive(Clone)]
pub struct Str {
    shared: Rc<Shared>,
    // is utf-8
    ptr: NonNull<u8>,
    len: usize,
}

impl Str {
    pub fn first(&self) -> Option<&u8> {
        self.as_bytes().first()
    }

    pub fn slice<R: slice::SliceIndex<str, Output = str>>(&self, range: R) -> Self {
        let slice = &self[range];
        Self {
            shared: self.shared.clone(),
            ptr: NonNull::new(slice.as_ptr().cast_mut()).unwrap(),
            len: slice.len(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }

    pub fn advance(&mut self, cnt: usize) {
        let _ = &self[cnt..]; // checks for utf-8 char boundary
        self.len = self.len.strict_sub(cnt);
        self.ptr = unsafe { self.ptr.add(cnt) };
    }

    pub fn split_to(&mut self, len: usize) -> Str {
        let ptr = self.ptr;
        self.advance(len);
        let shared = self.shared.clone();
        Str { shared, ptr, len }
    }

    pub fn trim_ascii_mut(&mut self) {
        let string = self.as_str().trim_ascii();
        let ptr = NonNull::new(string.as_ptr().cast_mut()).unwrap();
        let len = string.len();
        self.ptr = ptr;
        self.len = len;
    }

    pub fn trim_ascii_start_mut(&mut self) {
        let string = self.as_str().trim_ascii_start();
        let ptr = NonNull::new(string.as_ptr().cast_mut()).unwrap();
        let len = string.len();
        self.ptr = ptr;
        self.len = len;
    }
}

impl FileBuffer {
    pub fn new(file: File) -> Self {
        let shared = Self::new_shared();
        Self {
            ptr: shared.ptr,
            len: 0,
            cap: shared.cap,
            file,
            shared,
        }
    }

    // utf-8 cannot be enforced because reading file can split utf-8 code point

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn first(&self) -> Option<u8> {
        self.as_bytes().first().copied()
    }

    pub fn advance(&mut self, cnt: usize) {
        self.len = self.len.strict_sub(cnt);
        self.ptr = unsafe { self.ptr.add(cnt) };
        self.cap -= cnt;
    }

    pub fn split_to(&mut self, len: usize) -> Str {
        let ptr = self.ptr;
        self.advance(len);
        let shared = self.shared.clone();
        Str { shared, ptr, len }
    }

    pub fn trim_ascii_start_mut(&mut self) {
        unsafe {
            self.advance(
                self.as_bytes()
                    .trim_ascii_start()
                    .as_ptr()
                    .offset_from_unsigned(self.ptr.as_ptr()),
            );
        }
    }
}

impl std::ops::Deref for FileBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl std::ops::Deref for Str {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl std::fmt::Debug for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::fmt::Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

// ===== Allocation =====

const MIN_CAP: usize = 1024 * 4;
const MIN_READ: usize = 1024;

struct Shared {
    ptr: NonNull<u8>,
    cap: usize,
}

impl Drop for Shared {
    fn drop(&mut self) {
        unsafe { Vec::from_raw_parts(self.ptr.as_ptr(), 0, self.cap) };
    }
}

impl FileBuffer {
    pub fn read(&mut self) {
        let spare = self.cap - self.len;
        if spare < MIN_READ {
            self.reserve();
        }

        unsafe {
            let spare = self.cap - self.len;
            let spare = slice::from_raw_parts_mut(self.ptr.add(self.len).as_ptr(), spare);
            let read = io::Read::read(&mut self.file, spare).expect("cannot read file");
            if read == 0 {
                panic!("unexpected EOF, len: {}, cap: {}, offset", self.len, self.cap);
            }
            // ensure invariant is still valid
            assert!(str::from_utf8(&spare[..read]).is_ok(), "non utf-8 file");
            self.len += read;
        }
    }

    fn new_shared() -> Rc<Shared> {
        let (ptr, _, cap) = Vec::with_capacity(MIN_CAP).into_raw_parts();
        let ptr = NonNull::new(ptr).unwrap();
        Rc::new(Shared { ptr, cap })
    }

    fn reserve(&mut self) {
        self.shared = Self::new_shared();
        unsafe { self.ptr.copy_to_nonoverlapping(self.shared.ptr, self.len) };
        self.ptr = self.shared.ptr;
        self.cap = self.shared.cap;
    }
}
