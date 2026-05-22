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
pub fn grow<T>(ptr: &mut NonNull<T>, cap: u32, additional: u32) -> u32 {
    let cap = cap as usize;
    let exp_cap = cap * 2;
    let new_cap = exp_cap.max(cap + additional as usize);
    let Ok(new_cap) = u32::try_from(new_cap) else {
        panic!("capacity overflow")
    };
    let layout = layout::<T>(new_cap);
    match unsafe {
        NonNull::new(alloc::realloc(
            ptr.as_ptr().cast(),
            layout,
            new_cap as usize,
        ))
    } {
        Some(ok) => *ptr = ok.cast(),
        None => alloc::handle_alloc_error(layout),
    };

    new_cap
}

pub fn deallocate_offset<T>(ptr: NonNull<T>, cap: u32, off: u32) {
    unsafe {
        alloc::dealloc(
            ptr.as_ptr().sub(off as usize).cast(),
            layout::<T>(cap + off),
        )
    };
}

pub fn deallocate<T>(ptr: NonNull<T>, cap: u32) {
    debug_assert!(cap != 0);
    unsafe { alloc::dealloc(ptr.as_ptr().cast(), layout::<T>(cap)) };
}
