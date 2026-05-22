use std::alloc::{self, Layout};
use std::ptr::NonNull;

const fn layout<T>(cap: u32) -> Layout {
    unsafe { Layout::from_size_align_unchecked(cap as usize * size_of::<T>(), align_of::<T>()) }
}

pub fn allocate<T>(cap: u32) -> NonNull<T> {
    debug_assert!(cap != 0);
    unsafe {
        let layout = layout::<T>(cap);
        match NonNull::new(alloc::alloc(layout)) {
            Some(ok) => ok.cast(),
            None => alloc::handle_alloc_error(layout),
        }
    }
}

#[cold]
#[inline(never)]
#[must_use]
pub fn grow_exp<T>(ptr: &mut NonNull<T>, cap: u32) -> u32 {
    let Ok(new_cap) = u32::try_from(cap as usize * 2) else {
        panic!("capacity overflow")
    };
    *ptr = grow_inner(*ptr, cap, new_cap);
    new_cap
}

#[cold]
#[inline(never)]
#[must_use]
pub fn grow<T>(ptr: &mut NonNull<T>, cap: u32, additional: u32) -> u32 {
    let old_cap = cap as usize;
    let exp_cap = old_cap * 2;
    let new_cap = exp_cap.max(old_cap + additional as usize);
    let Ok(new_cap) = u32::try_from(new_cap) else {
        panic!("capacity overflow")
    };
    *ptr = grow_inner(*ptr, cap, new_cap);
    new_cap
}

fn grow_inner<T>(ptr: NonNull<T>, old_cap: u32, new_cap: u32) -> NonNull<T> {
    let old_layout = layout::<T>(old_cap);
    let result = unsafe { alloc::realloc(ptr.as_ptr().cast(), old_layout, new_cap as usize) };
    match NonNull::new(result) {
        Some(ok) => ok.cast(),
        None => alloc::handle_alloc_error(old_layout),
    }
}

pub fn deallocate_offset<T>(ptr: NonNull<T>, cap: u32, off: u32) {
    unsafe { deallocate(ptr.sub(off as usize), cap + off) };
}

pub fn deallocate<T>(ptr: NonNull<T>, cap: u32) {
    debug_assert!(cap != 0);
    unsafe { alloc::dealloc(ptr.as_ptr().cast(), layout::<T>(cap)) };
}
