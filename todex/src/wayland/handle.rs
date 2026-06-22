
/// Type that represent a handle.
pub trait AsHandle: Sized {
    fn from_raw_handle(id: u32) -> Self;

    fn to_raw_handle(self) -> u32;

    #[inline]
    fn to_handle(self) -> Handle {
        Handle(self.to_raw_handle())
    }
}

/// A handle that points to a resource.
#[derive(Default, Debug, Clone, Copy)]
pub struct Handle(u32);

impl Handle {
    /// Create `Handle` from collection index.
    ///
    /// # Panics
    ///
    /// Panics if index exceeds `u32::MAX`.
    #[inline]
    pub const fn from_idx(idx: usize) -> Self {
        if idx > u32::MAX as usize {
            id_overflow();
        }
        Self(idx as u32)
    }

    /// Restore handle from raw int.
    #[inline]
    pub const fn from_raw(id: u32) -> Handle {
        Self(id)
    }

    /// Convert handle to index.
    #[inline]
    pub const fn to_idx(self) -> usize {
        self.0 as usize
    }

    /// Convert to other type of handle.
    pub fn to_handle<H: AsHandle>(self) -> H {
        H::from_raw_handle(self.0)
    }
}

impl AsHandle for Handle {
    #[inline]
    fn from_raw_handle(id: u32) -> Self {
        Self(id)
    }

    #[inline]
    fn to_raw_handle(self) -> u32 {
        self.0
    }
}

#[inline(never)]
#[cold]
const fn id_overflow() -> ! {
    panic!("id overflow")
}
