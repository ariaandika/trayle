use std::ptr::NonNull;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Ptr<T>(NonNull<T>);

#[allow(clippy::wrong_self_convention)]
impl<T> Ptr<T> {
    pub fn with_capacity(cap: u32) -> Ptr<T> {
        Self(alloc::allocate(cap))
    }

    pub fn cast<U>(self) -> Ptr<U> {
        Ptr(self.0.cast())
    }

    pub fn add(self, n: u32) -> Self {
        unsafe { Ptr(self.0.add(n as usize)) }
    }

    pub fn sub_mut(&mut self, off: u32) {
        self.0 = unsafe { self.0.sub(off as usize) };
    }

    pub fn as_slice<'a>(self, len: u32) -> &'a [T] {
        unsafe { std::slice::from_raw_parts(self.0.as_ptr(), len as usize) }
    }

    pub fn as_mut_slice<'a>(self, len: u32) -> &'a mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.0.as_ptr(), len as usize) }
    }

    pub fn grow(&mut self, old_cap: u32, new_cap: u32) {
        let exp_cap = old_cap as usize * 2;
        let new_cap = exp_cap.max(new_cap as usize);
        self.0 = alloc::grow(self.0, old_cap, new_cap);
    }

    pub fn deallocate(self, cap: u32) {
        alloc::deallocate(self.0, cap);
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
        unsafe { alloc::dealloc(ptr.as_ptr().cast(), layout::<T>(cap)) };
    }
}
