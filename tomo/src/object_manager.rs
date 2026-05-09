use std::ptr::{self, NonNull};

use crate::Id;
use crate::lookup::Interfaces;

const DEFAULT_ALLOC_SIZE: u32 = 512;
const DEFAULT_ALLOC_LEN: u32 = DEFAULT_ALLOC_SIZE / OBJECT_SIZE;
const DEFAULT_CAP_LEN: u32 = DEFAULT_ALLOC_LEN - HEADER_LEN;

const OBJECT_SIZE: u32 = size_of::<u32>() as u32;
const HEADER_LEN: u32 = 4;

/// Wayland Object Manager.
///
/// # Allocation
///
/// Uses single allocation.
///
/// ```not_rust
/// [ cap @ u32 | len @ u32 | last_deleted @ u32 | _pad @ u32 | object @ [(u32, u32)] ]
/// ```
///
/// `cap` and `len` is in `object` unit, excluding the allocation header.
///
/// To get allocation size in bytes: `cap * (size_of::<u32>() * 2) + HDR`
pub struct ObjectManager {
    ptr: NonNull<u32>,
}

impl Drop for ObjectManager {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.ptr.as_ptr();
            let cap = *ptr;
            let len = cap + HEADER_LEN;
            let slice = ptr::slice_from_raw_parts_mut(ptr, len as usize);
            drop(Box::from_raw(slice));
        }
    }
}

impl ObjectManager {
    #[inline]
    pub fn new() -> Self {
        let ptr = Box::into_raw(Box::<[u32]>::new_uninit_slice(DEFAULT_ALLOC_LEN as usize));
        unsafe {
            let ptr = NonNull::new_unchecked(ptr.cast());
            ptr.write_bytes(0, HEADER_LEN as usize);
            ptr.write(DEFAULT_CAP_LEN);
            Self { ptr }
        }
    }

    pub fn insert(&mut self, interface: Interfaces) -> Id {
        unsafe {
            let ptr = self.ptr.as_ptr();
            let cap = *ptr;
            let len = *ptr.add(1);
            if cap == len {
                todo!("reallocate")
            }
            let last_deleted = *ptr.add(2);
            if last_deleted != 0 {
                todo!("reuse ID")
            }
            // 0 is invalid id, 1 is wl_display
            let id = len + 2;
            let id = Id::new_non_zero(std::num::NonZeroU32::new_unchecked(id));

            // store the object
            ptr.add((HEADER_LEN + len) as usize).write(interface as u32);
            // update `len`
            *ptr.add(1) += 1;

            id
        }
    }

    #[inline]
    pub fn get(&self, object_id: Id) -> Option<Interfaces> {
        unsafe {
            let len = self.ptr.add(1).read();
            let idx = object_id.sub_2();
            if idx < len {
                let iface = self.ptr.add((HEADER_LEN + idx) as usize).read();
                Some(Interfaces::from_u32_unchecked(iface))
            } else {
                None
            }
        }
    }
}

impl Default for ObjectManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ObjectManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut i = 0;
        let len = unsafe { self.ptr.add(1).read() };
        f.debug_map()
            .entries(std::iter::from_fn(|| unsafe {
                if i == len {
                    return None;
                }
                let object = self.ptr.add((HEADER_LEN + i) as usize).read();
                let id = i + 2;
                i += 1;
                Some((id, object))
            }))
            .finish()
    }
}
