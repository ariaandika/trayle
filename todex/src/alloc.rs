//! Memory allocations.
use std::ptr::NonNull;

pub fn allocate<T>(cap: usize) -> NonNull<T> {
    let Some(size) = size_of::<T>().checked_mul(cap) else {
        capacity_overflow();
    };
    cvt(unsafe { libc::malloc(size) })
}

pub fn reallocate<T>(ptr: NonNull<T>, new_cap: usize) -> NonNull<T> {
    debug_assert!(new_cap != 0);
    cvt(unsafe { libc::realloc(ptr.as_ptr().cast(), new_cap) })
}

pub fn deallocate<T>(ptr: NonNull<T>) {
    unsafe { libc::free(ptr.as_ptr().cast()) };
}

/// Calculate exponential grow or required additional capacity.
///
/// # Panics
///
/// Panics if calculation overflow.
pub fn calc_grow(cap: usize, additional: usize) -> usize {
    let req = cap.checked_add(additional);
    let exp = cap.checked_mul(2);
    match req.max(exp).filter(|e|*e <= isize::MAX as usize) {
        Some(ok) => ok,
        None => capacity_overflow()
    }
}

/// Calculate exponential grow.
///
/// # Panics
///
/// Panics if calculation overflow.
pub fn calc_exp(cap: usize) -> usize {
    match cap.checked_mul(2).filter(|e| *e <= isize::MAX as usize) {
        Some(ok) => ok,
        None => capacity_overflow(),
    }
}

fn cvt<T>(ptr: *mut std::ffi::c_void) -> NonNull<T> {
    match NonNull::new(ptr) {
        Some(ok) => ok.cast(),
        None => alloc_error()
    }
}

#[inline(never)]
pub fn capacity_overflow() -> ! {
    panic!("capacity overflow")
}

#[inline(never)]
fn alloc_error() -> ! {
    panic!("allocation error")
}
