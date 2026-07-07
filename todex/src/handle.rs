pub struct Handle<T> {
    id: u32,
    // `fn() -> T` to remove bounds
    _p: std::marker::PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    #[inline]
    pub const fn from_idx(idx: usize) -> Self {
        if idx > u32::MAX as usize {
            id_overflow();
        }
        Self {
            id: idx as u32,
            _p: std::marker::PhantomData,
        }
    }

    #[inline]
    pub const fn cast<U>(self) -> Handle<U> {
        Handle {
            id: self.id,
            _p: std::marker::PhantomData,
        }
    }

    #[inline]
    pub const fn to_idx(self) -> usize {
        self.id as usize
    }
}

impl<T> Clone for Handle<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> std::fmt::Display for Handle<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

impl<T> std::fmt::Debug for Handle<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Handle").field(&self.id).finish()
    }
}

#[inline(never)]
#[cold]
const fn id_overflow() -> ! {
    panic!("id overflow")
}
