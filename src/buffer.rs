use std::{mem::MaybeUninit, ptr::NonNull, slice};

pub struct Buffer {
    ptr: NonNull<u8>,
    off: u32,
    len: u32,
    cap: u32,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        alloc::deallocate(self.ptr, self.cap, self.off);
    }
}

impl Buffer {
    pub fn with_capacity(capacity: u32) -> Self {
        debug_assert_ne!(capacity, 0);
        let ptr = unsafe { alloc::allocate(capacity) };
        Self {
            ptr,
            off: 0,
            len: 0,
            cap: capacity,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len as usize) }
    }

    /// # Safety
    ///
    /// `cnt` element after the last element must be initialized.
    pub unsafe fn advance_mut(&mut self, cnt: u32) {
        debug_assert!(self.cap - self.len >= cnt);
        self.len += cnt;
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.cap += self.off;
        self.ptr = unsafe { self.ptr.sub(self.off as usize) };
        self.off = 0;
    }

    /// Returns `true` if remaining capacity is sufficient and the data is copied.
    pub fn try_extend_from_slice(&mut self, slice: &[u8]) -> bool {
        let spare = self.spare_capacity_mut();
        if spare.len() >= slice.len() {
            unsafe {
                spare
                    .as_mut_ptr()
                    .copy_from_nonoverlapping(slice.as_ptr().cast(), slice.len());
                self.advance_mut(slice.len() as u32);
            }
            true
        } else {
            false
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
}

impl std::ops::Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

mod alloc {
    use std::alloc::{self, Layout};
    use std::ptr::NonNull;

    pub unsafe fn allocate(capacity: u32) -> NonNull<u8> {
        unsafe {
            let layout = Layout::from_size_align_unchecked(capacity as usize, 1);
            match NonNull::new(alloc::alloc(layout)) {
                Some(ok) => ok,
                None => alloc::handle_alloc_error(layout),
            }
        }
    }

    // pub unsafe fn grow(ptr: NonNull<u8>, old_cap: u32, new_cap: u32) -> NonNull<u8> {
    //     unsafe {
    //         let layout = Layout::from_size_align_unchecked(old_cap as usize, 1);
    //         match NonNull::new(alloc::realloc(ptr.as_ptr(), layout, new_cap as usize)) {
    //             Some(ok) => ok,
    //             None => alloc::handle_alloc_error(layout),
    //         }
    //     }
    // }

    pub fn deallocate(ptr: NonNull<u8>, cap: u32, offset: u32) {
        unsafe {
            let layout = Layout::from_size_align_unchecked((cap + offset) as usize, 1);
            alloc::dealloc(ptr.as_ptr().sub(offset as usize), layout);
        }
    }
}

