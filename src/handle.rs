use todex::wayland::object::Object;
use todex::wayland::message::Message;

// ===== AsHandle =====

/// Type that is associated with [`Handle`].
pub trait AsHandle<T> {
    fn handle(&self) -> Handle<T>;
}

impl<I, M, H> AsHandle<H> for Object<I, M, Handle<H>> {
    fn handle(&self) -> Handle<H> {
        self.id()
    }
}

impl<I, M, H> AsHandle<H> for Message<I, M, Handle<H>> {
    fn handle(&self) -> Handle<H> {
        self.id()
    }
}

// ===== WithHandle =====

/// Type that is associated with handle type.
pub trait WithHandle {
    type Handle;
}

// ===== Handle =====

pub struct Handle<T> {
    id: u32,
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
    pub const fn to_idx(self) -> usize {
        self.id as usize
    }

    #[inline]
    pub const fn cast<U>(self) -> Handle<U> {
        Handle {
            id: self.id,
            _p: std::marker::PhantomData,
        }
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
