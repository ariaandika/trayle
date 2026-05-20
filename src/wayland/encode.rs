use crate::buffer::Buffer;
use crate::wayland::{Id, roundup4};

pub trait Encoder {
    fn message(&mut self, object_id: Id, op: u16, len: u16) -> *mut u8;
}

impl Encoder for Buffer {
    fn message(&mut self, object_id: Id, op: u16, len: u16) -> *mut u8 {
        self.reserve(len as u32);
        let ptr = self.spare_capacity_mut().as_mut_ptr().cast::<u8>();
        unsafe {
            self.advance_mut(len as u32);
            ptr.put(object_id).put(op).put(len)
        }
    }
}

// ===== Traits =====

pub trait PtrWrite {
    /// # Safety
    ///
    /// Pointer must be valid for write until required length.
    unsafe fn put<P: PrimitiveEncode>(self, value: P) -> *mut u8;
}

impl PtrWrite for *mut u8 {
    unsafe fn put<P: PrimitiveEncode>(self, value: P) -> *mut u8 {
        unsafe { value.encode(self) }
    }
}

pub trait PrimitiveEncode {
    /// # Safety
    ///
    /// Pointer must be valid for write until required length.
    unsafe fn encode(self, ptr: *mut u8) -> *mut u8;
}

impl PrimitiveEncode for u32 {
    unsafe fn encode(self, ptr: *mut u8) -> *mut u8 {
        unsafe {
            ptr.copy_from_nonoverlapping(self.to_ne_bytes().as_ptr(), 4);
            ptr.add(4)
        }
    }
}

impl PrimitiveEncode for Id {
    unsafe fn encode(self, ptr: *mut u8) -> *mut u8 {
        unsafe {
            ptr.copy_from_nonoverlapping(self.to_ne_bytes().as_ptr(), 4);
            ptr.add(4)
        }
    }
}

impl PrimitiveEncode for u16 {
    unsafe fn encode(self, ptr: *mut u8) -> *mut u8 {
        unsafe {
            ptr.copy_from_nonoverlapping(self.to_ne_bytes().as_ptr(), 2);
            ptr.add(2)
        }
    }
}

impl PrimitiveEncode for &str {
    unsafe fn encode(self, ptr: *mut u8) -> *mut u8 {
        unsafe {
            let len = self.len() as u16;
            ptr.copy_from_nonoverlapping(len.to_ne_bytes().as_ptr(), 4);
            ptr.add(4).copy_from_nonoverlapping(self.as_ptr(), self.len());
            ptr.add((4 + len) as usize).write(0);
            ptr.add((4 + roundup4!(len + 1)) as usize)
        }
    }
}
