//! Memory allocations.
use core::ffi;
use core::ptr::NonNull;

pub fn allocate<T>(cap: usize) -> NonNull<T> {
    let size = size_of::<T>()
        .checked_mul(cap)
        .unwrap_or_else(capacity_overflow);
    NonNull::new(unsafe { libc::malloc(size) })
        .unwrap_or_else(alloc_error)
        .cast()
}

pub fn reallocate<T>(ptr: NonNull<T>, new_cap: usize) -> NonNull<T> {
    debug_assert!(new_cap != 0);
    NonNull::new(unsafe { libc::realloc(ptr.as_ptr().cast(), new_cap) })
        .unwrap_or_else(alloc_error)
        .cast()
}

pub fn deallocate<T>(ptr: NonNull<T>) {
    unsafe { libc::free(ptr.as_ptr().cast()) };
}

/// Calculate exponential grow or required additional capacity.
///
/// # Panics
///
/// Panics if calculation exceeds `isize::MAX`.
pub fn calc_grow(cap: usize, additional: usize) -> usize {
    cap.checked_add(additional)
        .max(cap.checked_mul(2))
        .filter(|e| *e <= isize::MAX as usize)
        .unwrap_or_else(capacity_overflow)
}

/// Calculate exponential grow.
///
/// # Panics
///
/// Panics if calculation exceeds `isize::MAX`.
pub fn calc_exp(cap: usize) -> usize {
    cap.checked_mul(2)
        .filter(|e| *e <= isize::MAX as usize)
        .unwrap_or_else(capacity_overflow)
}

#[inline(never)]
pub fn capacity_overflow() -> usize {
    panic!("capacity overflow")
}

#[inline(never)]
fn alloc_error() -> NonNull<ffi::c_void> {
    panic!("allocation error")
}
