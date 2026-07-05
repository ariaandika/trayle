use core::fmt;
use core::mem::ManuallyDrop;
use core::ptr::{self, NonNull};
use core::slice;

use crate::alloc;

/// A growable array slots.
///
/// An array type where caller decide the index where the elements will take place. The index for
/// given element will never change, despite other mutable operations to the slots. This allows for
/// other resource holds reference by just having stable index
///
/// The max index that can be used must be one after the last used index. An attempts to violate
/// this will returns an `Err` with the failed element. If the index is already used, will also
/// returns an `Err`.
pub struct Slots<T> {
    ptr: NonNull<Option<T>>,
    len: usize,
    cap: usize,
}

impl<T> Drop for Slots<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }
        unsafe { ptr::drop_in_place(self.as_mut_entries()) };
        alloc::deallocate(self.ptr);
    }
}

impl<T> Slots<T> {
    /// Create new empty slots.
    ///
    /// This does not allocate.
    #[inline]
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    /// Create new empty buffer with given capacity.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            ptr: alloc::allocate(cap),
            len: 0,
            cap,
        }
    }

    #[cold]
    #[inline(never)]
    fn grow_one(&mut self) {
        let new_cap = alloc::calc_exp(self.cap);
        self.ptr = alloc::reallocate(self.ptr, new_cap);
        self.cap = new_cap;
    }
}

impl<T> Slots<T> {
    fn as_entries(&self) -> &[Option<T>] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    fn as_mut_entries(&mut self) -> &mut [Option<T>] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> Slots<T> {
    /// Insert element at given index.
    ///
    /// Note on max index rule in the struct [documentation][Slots].
    pub fn insert(&mut self, idx: usize, value: T) -> Result<(), T> {
        if self.len == self.cap {
            self.grow_one();
        }

        // appending can only be one index after the last used idx
        //
        // if it skips unused idx, it will left skipped id unitialized
        if idx > self.len {
            return Err(value);
        }

        if idx == self.len {
            // SAFETY: `idx == len`, one after last element always uninitialized, no drop required
            unsafe { self.ptr.add(idx).write(Some(value)) };
            self.len += 1;
        } else {
            // SAFETY: `idx < len`
            let entry_mut = unsafe { self.ptr.add(idx).as_mut() };
            if entry_mut.is_some() {
                return Err(value);
            }
            *entry_mut = Some(value);
        }

        Ok(())
    }

    /// Mark the index as used.
    ///
    /// This has the same effect of inserting the index and immediately remove it.
    ///
    /// Note that currently, this ignore if the index is already used or out of bounds.
    #[inline]
    pub fn use_one(&mut self, idx: usize) {
        if self.len == self.cap {
            self.grow_one();
        }
        if idx == self.len {
            // SAFETY: `idx == len`, one after last element always uninitialized, no drop required
            unsafe { self.ptr.add(idx).write(None) };
            self.len += 1;
        }
    }

    /// Removes and returns element with given index.
    ///
    /// The index will be released and may be used by future element.
    #[inline]
    pub fn remove(&mut self, idx: usize) -> Option<T> {
        if idx < self.len {
            // SAFETY: `idx < self.len`
            unsafe { self.ptr.add(idx).as_mut() }.take()
        } else {
            None
        }
    }
}

impl<T> Slots<T> {
    /// Returns a mutable reference of element with given index.
    ///
    /// Returns `None` if given index is out of bounds or points to removed element.
    #[inline]
    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.len {
            // SAFETY: `idx < self.len`
            unsafe { self.ptr.add(idx).as_ref().as_ref() }
        } else {
            None
        }
    }

    /// Returns a mutable reference of element with given index.
    ///
    /// Returns `None` if given index is out of bounds or points to removed element.
    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < self.len {
            // SAFETY: `idx < self.len`
            unsafe { self.ptr.add(idx).as_mut().as_mut() }
        } else {
            None
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Slots<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.as_entries().iter().filter_map(Option::as_ref))
            .finish()
    }
}

// ===== IntoIterator =====

impl<T> IntoIterator for Slots<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let slots = ManuallyDrop::new(self);
        IntoIter {
            current: slots.ptr,
            end: unsafe { slots.ptr.add(slots.cap) },
            cap: slots.cap,
        }
    }
}

pub struct IntoIter<T> {
    current: NonNull<Option<T>>,
    end: NonNull<Option<T>>,
    cap: usize,
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(
                self.current.as_ptr(),
                self.end.offset_from_unsigned(self.current),
            ));
            alloc::deallocate(self.end.sub(self.cap));
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // some entry can be `None`, loop to re-iterate if it encountered
        loop {
            if self.current == self.end {
                return None;
            }
            let old = self.current;
            let entry = unsafe {
                self.current = old.add(1);
                old.read()
            };
            if let Some(some) = entry {
                break Some(some)
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remain = self.len();
        (remain, Some(remain))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    #[inline]
    fn len(&self) -> usize {
        unsafe { self.end.offset_from_unsigned(self.current) }
    }
}

