use core::hint;
use core::mem;
use core::ptr::{self, NonNull};
use core::slice;
use std::fmt;

use crate::collections::alloc;

enum Entry<T> {
    Some(T),
    None(usize)
}

pub struct Slab<T> {
    ptr: NonNull<Entry<T>>,
    len: usize,
    cap: usize,
    last_delete: usize,
}

impl<T> Drop for Slab<T> {
    fn drop(&mut self) {
        // SAFETY: `drop_in_place` called in `Drop` impl
        unsafe { ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), self.len).drop_in_place() };
        alloc::deallocate(self.ptr);
    }
}

impl<T> Slab<T> {
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            ptr: alloc::allocate(cap),
            len: 0,
            cap,
            last_delete: 0,
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self) {
        let new_cap = alloc::calc_exp(self.cap);
        self.ptr = alloc::reallocate(self.ptr, new_cap);
        self.cap = new_cap;
    }
}

impl<T> Slab<T> {
    /// Insert new value, returns the associated `key`.
    ///
    /// `key` is used for other slab operation.
    #[inline]
    pub fn insert(&mut self, value: T) -> (usize, &mut T) {
        if self.len == self.cap {
            self.grow();
        }

        // SAFETY: `last_delete <= len`, thus `last_delete <= cap`
        let mut target_ptr = unsafe { self.ptr.add(self.last_delete) };
        let key = self.last_delete;

        if self.last_delete == self.len {
            // appending
            unsafe { target_ptr.write(Entry::Some(value)) };
            self.len += 1;
            self.last_delete += 1;
        } else {
            // invariants
            debug_assert!(self.last_delete < self.len);

            // SAFETY: `last_delete < len` thus its initialized
            let old_entry = unsafe { target_ptr.replace(Entry::Some(value)) };

            let Entry::None(next_delete) = old_entry else {
                // `last_delete` contains invalid state
                unreachable!("corrupted slab");
            };
            self.last_delete = next_delete;
        }

        // SAFETY: `target_ptr` has been initialized to `Entry::Some` above
        let client_mut = unsafe {
            match target_ptr.as_mut() {
                Entry::Some(client) => client,
                Entry::None(_) => hint::unreachable_unchecked(),
            }
        };

        (key, client_mut)
    }
}

impl<T> Slab<T> {
    fn as_slice(&self) -> &[Entry<T>] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Returns shared reference of an element with associated `key`.
    #[inline]
    pub fn get(&self, key: usize) -> Option<&T> {
        if key < self.len {
            // SAFETY: `idx < self.len`
            match unsafe { self.ptr.add(key).as_ref() } {
                Entry::Some(ok) => Some(ok),
                Entry::None(_) => None,
            }
        } else {
            None
        }
    }

    /// Returns mutable reference of an element with associated `key`.
    #[inline]
    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        if key < self.len {
            // SAFETY: `idx < self.len`
            match unsafe { self.ptr.add(key).as_mut() } {
                Entry::Some(ok) => Some(ok),
                Entry::None(_) => None,
            }
        } else {
            None
        }
    }

    /// Removes and returns element with associated `key`.
    ///
    /// The `key` will be released and may be associated with future element.
    #[inline]
    pub fn remove(&mut self, key: usize) -> Option<T> {
        if key >= self.len {
            return None;
        }

        // SAFETY: `idx < self.len`
        let target = unsafe { self.ptr.add(key).as_mut() };
        let mut deleted = Entry::None(self.last_delete);
        mem::swap(target, &mut deleted);

        match deleted {
            Entry::Some(client) => {
                self.last_delete = key;
                Some(client)
            }
            Entry::None(_) => {
                // dangling id, restore the entry
                mem::swap(target, &mut deleted);
                None
            }
        }
    }

    /// Returns an iterator over the slab.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter(self.as_slice().iter().filter_map(|e| match e {
            Entry::Some(e) => Some(e),
            Entry::None(_) => None,
        }))
    }
}

impl<T: fmt::Debug> fmt::Debug for Slab<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

// ===== Iterator =====

pub struct Iter<'a, T>(IntoIter<'a, T>);

type IntoIter<'a, T> =
    std::iter::FilterMap<std::slice::Iter<'a, Entry<T>>, fn(&'a Entry<T>) -> Option<&'a T>>;

impl<'a, T> IntoIterator for &'a Slab<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
