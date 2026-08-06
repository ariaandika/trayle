use core::fmt;
use core::mem;
use core::ops;
use core::ptr::{self, NonNull};
use core::slice;

use crate::collections::alloc;

/// A contiguous growable array type.
///
/// Additionally, this also have an offset field, which will mark the first n elements as removed.
/// This can simulate a queue that is more efficient than vector.
///
/// Note that this struct maintains contiguous memory. Any insertion to the back of the elements,
/// may grow allocation without reusing leftover capacity. Use [`Buffer::backshift`] to reset the
/// offset by copying data backwards. Calling [`Buffer::clear`] also reset the offset.
pub struct Buffer<T> {
    ptr: NonNull<T>,
    len: usize,
    off: usize,
    cap: usize,
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }
        unsafe {
            ptr::drop_in_place(self.as_mut_slice());
            alloc::deallocate(self.ptr.sub(self.off));
        }
    }
}

impl<T> Buffer<T> {
    /// Create new empty buffer.
    ///
    /// This does not allocate.
    #[inline]
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            off: 0,
            cap: 0,
        }
    }

    /// Create new empty buffer with given capacity.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            ptr: alloc::allocate(cap),
            len: 0,
            off: 0,
            cap,
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self, additional: usize) {
        let base_cap = self.cap + self.off;
        let new_cap = alloc::calc_grow(base_cap, additional);
        let base_ptr = unsafe { self.ptr.sub(self.off) };
        let new_ptr = alloc::reallocate(base_ptr, new_cap);
        self.ptr = unsafe { new_ptr.add(self.off) };
        self.cap = new_cap - self.off;
    }

    #[cold]
    #[inline(never)]
    fn grow_one(&mut self) {
        let new_cap = alloc::calc_exp(self.cap + self.off);
        let base_ptr = unsafe { self.ptr.sub(self.off) };
        let new_ptr = alloc::reallocate(base_ptr, new_cap);
        self.ptr = unsafe { new_ptr.add(self.off) };
        self.cap = new_cap - self.off;
    }

    /// Reserve capacity for additional more elements.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        if self.cap - self.len < additional {
            self.grow(additional);
        }
    }
}

impl<T> Buffer<T> {
    /// Returns elements as shared slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Returns elements as mutable slice.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    /// Returns buffer capacity.
    ///
    /// Note that this excluding the offset.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Appends an element to the back of the buffer.
    #[inline]
    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow_one();
        }
        unsafe {
            self.ptr.write(value);
            self.len += 1;
        }
    }

    /// Removes the first element and returns it, or `None` if the buffer is empty.
    #[inline]
    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.advance_offset(1);
        // SAFETY: the value ownership is returned
        unsafe { Some(self.ptr.sub(1).read()) }
    }

    /// Drop the first `cnt` elements.
    ///
    /// This operation is `O(1)`. See the struct [documentation][Buffer] for more details.
    #[inline]
    pub fn advance(&mut self, cnt: usize) {
        assert!(cnt <= self.len, "advance out of bounds");
        // SAFETY: asserted above
        unsafe { self.advance_unchecked(cnt) };
    }

    /// Returns slice of remaining capacity.
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [mem::MaybeUninit<T>] {
        unsafe {
            slice::from_raw_parts_mut(
                self.ptr.as_ptr().add(self.len).cast(),
                self.cap - self.len
            )
        }
    }

    /// Copy the data backwards, resetting the offset.
    ///
    /// This should be used as "maintenance" method to reuse leftover capacity. See the struct
    /// [documentation][Buffer] for more details.
    #[inline]
    pub fn backshift(&mut self) {
        unsafe {
            let dst = self.ptr.sub(self.off);
            let src = self.ptr;
            let count = self.len;
            if count <= self.off {
                dst.copy_from_nonoverlapping(src, count);
            } else {
                dst.copy_from(src, count);
            }
        }
        self.cap += self.off;
        self.off = 0;
    }

    /// Drop all elements, resetting the offset.
    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            ptr::drop_in_place(self.as_mut_slice());
            self.ptr = self.ptr.sub(self.off);
        }
        self.cap += self.off;
        self.len = 0;
        self.off = 0;
    }
}

impl<T> Buffer<T> {
    /// Drop the first `cnt` elements.
    ///
    /// See [`Buffer::advance`] for safe alternative.
    ///
    /// # Safety
    ///
    /// `cnt <= self.len()`
    #[inline]
    pub unsafe fn advance_unchecked(&mut self, cnt: usize) {
        debug_assert!(cnt <= self.len, "advance_unchecked out of bounds");
        unsafe { ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), cnt).drop_in_place() };
        self.advance_offset(cnt);
    }

    #[inline]
    fn advance_offset(&mut self, cnt: usize) {
        debug_assert!(cnt <= self.len, "internal: advance_offset out of bounds");
        self.ptr = unsafe { self.ptr.add(cnt) };
        self.off += cnt;
        self.len -= cnt;
        self.cap -= cnt;
    }

    /// Forces the length of the buffer to `new_len`.
    ///
    /// # Safety
    ///
    /// * `new_len` must be less than or equal to [`capacity()`].
    /// * The elements at `old_len..new_len` must be initialized.
    ///
    /// [`capacity()`]: Buffer::capacity
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.cap, "set_len out of bounds");
        self.len = new_len;
    }

}

impl<T> Default for Buffer<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ops::Deref for Buffer<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> ops::DerefMut for Buffer<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T: fmt::Debug> fmt::Debug for Buffer<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}
