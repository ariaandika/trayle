use std::ptr::NonNull;

#[repr(transparent)]
pub struct Ptr<T>(NonNull<T>);

impl<T> Copy for Ptr<T> { }

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[allow(clippy::wrong_self_convention)]
impl<T> Ptr<T> {
    pub const fn new(ptr: *mut T) -> Option<Self> {
        match NonNull::new(ptr) {
            Some(x) => Some(Self(x)),
            None => None,
        }
    }

    pub fn with_capacity(cap: u32) -> Ptr<T> {
        Self(alloc::allocate(cap))
    }

    pub fn allocate(cap: u32) -> Ptr<T> {
        Self(alloc::allocate(cap))
    }

    pub const fn cast<U>(self) -> Ptr<U> {
        Ptr(self.0.cast())
    }

    pub const fn add(self, n: u32) -> Self {
        unsafe { Ptr(self.0.add(n as usize)) }
    }

    pub const fn sub(self, n: u32) -> Self {
        unsafe { Ptr(self.0.sub(n as usize)) }
    }

    pub const fn sub_mut(&mut self, off: u32) {
        self.0 = unsafe { self.0.sub(off as usize) };
    }

    pub const fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    pub const fn as_ref<'a>(self) -> &'a T {
        unsafe { self.0.as_ref() }
    }

    pub const fn as_mut<'a>(mut self) -> &'a mut T {
        unsafe { self.0.as_mut() }
    }

    pub const fn read(self) -> T {
        unsafe { self.0.read() }
    }

    pub const fn write(self, val: T) {
        unsafe { self.0.write(val) };
    }

    pub const fn replace(self, val: T) -> T {
        unsafe { self.0.replace(val) }
    }

    pub const fn as_slice<'a>(self, len: u32) -> &'a [T] {
        unsafe { std::slice::from_raw_parts(self.0.as_ptr(), len as usize) }
    }

    pub const fn as_mut_slice<'a>(self, len: u32) -> &'a mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.0.as_ptr(), len as usize) }
    }

    pub const fn copy_from_nonoverlapping(self, ptr: *const T, count: u32) {
        unsafe { self.0.as_ptr().copy_from_nonoverlapping(ptr, count as usize) };
    }

    #[cold]
    #[inline(never)]
    pub fn grow(&mut self, old_cap: u32, new_cap: u32) -> u32 {
        let exp_cap = old_cap as usize * 2;
        let new_cap = exp_cap.max(new_cap as usize);
        assert!(exp_cap < (u32::MAX >> 1) as usize, "max capacity exceeded");
        self.0 = alloc::grow(self.0, old_cap, new_cap);
        new_cap as u32
    }

    #[cold]
    #[inline(never)]
    pub fn grow_offset(&mut self, old_cap: u32, add: u32, offset: u32) -> u32 {
        let mut ptr = self.sub(offset);
        let cap = ptr.grow_inner(old_cap + offset, add);
        *self = ptr;
        cap
    }

    fn grow_inner(&mut self, old_cap: u32, add: u32) -> u32 {
        let exp_cap = old_cap as usize * 2;
        let new_cap = exp_cap.max((old_cap + add) as usize);
        assert!(exp_cap < (u32::MAX >> 1) as usize, "max capacity exceeded");
        self.0 = alloc::grow(self.0, old_cap, new_cap);
        new_cap as u32
    }

    pub fn deallocate(self, cap: u32) {
        alloc::deallocate(self.0, cap);
    }

    pub fn deallocate_offset(self, cap: u32, off: u32) {
        unsafe { alloc::deallocate(self.0.sub(off as usize), cap + off) };
    }
}

impl<T> std::ops::Index<u32> for Ptr<T> {
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        unsafe { self.0.add(index as usize).as_ref() }
    }
}

mod alloc {
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

    pub fn grow<T>(ptr: NonNull<T>, old_cap: u32, new_cap: usize) -> NonNull<T> {
        unsafe {
            let layout = layout::<T>(old_cap);
            match NonNull::new(alloc::realloc(ptr.as_ptr().cast(), layout, new_cap)) {
                Some(ok) => ok.cast(),
                None => alloc::handle_alloc_error(layout),
            }
        }
    }

    pub fn deallocate<T>(ptr: NonNull<T>, cap: u32) {
        debug_assert!(cap != 0);
        unsafe { alloc::dealloc(ptr.as_ptr().cast(), layout::<T>(cap)) };
    }
}
