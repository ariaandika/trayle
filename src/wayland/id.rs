use std::num::NonZeroU32;

/// Object ID.
///
/// The IDs are allocated by the entity creating the object (either client or server). IDs allocated
/// by the client are in the range `[1, 0xfeffffff]` while IDs allocated by the server are in the
/// range `[0xff000000, 0xffffffff]`.
///
/// The `0` ID is reserved to represent a null or non-existent object.
///
/// For efficiency purposes, the IDs are densely packed in the sense that the ID `N` will not be
/// used until `N-1` has been used. This ordering is not merely a guideline, but a strict
/// requirement, and there are implementations of the protocol that rigorously enforce this rule,
/// including the ubiquitous libwayland.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Id(NonZeroU32);

impl Id {
    pub const fn new(id: u32) -> Option<Self> {
        debug_assert!(id != 0);
        match NonZeroU32::new(id) {
            Some(x) => Some(Self(x)),
            None => None,
        }
    }

    pub const fn new_const<const ID: u32>() -> Self {
        debug_assert!(ID > 1);
        unsafe { Self(NonZeroU32::new_unchecked(ID)) }
    }

    pub const fn wl_display() -> Self {
        unsafe { Self(NonZeroU32::new_unchecked(1)) }
    }

    /// Returns `true` if id is special id for `wl_display`.
    pub const fn is_display(&self) -> bool {
        self.0.get() == 1
    }

    /// Returns ID as `u32`.
    pub const fn as_u32(&self) -> u32 {
        self.0.get()
    }

    /// Returns the memory representation of this integer as a byte array in native byte order
    pub const fn to_ne_bytes(self) -> [u8; 4] {
        self.0.get().to_ne_bytes()
    }
}

impl PartialEq<u32> for Id {
    fn eq(&self, other: &u32) -> bool {
        self.0.get() == *other
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id({})", self.0)
    }
}
