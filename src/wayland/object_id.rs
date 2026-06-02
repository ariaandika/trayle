use std::num::NonZeroU32;

pub trait FromObjectId {
    fn from_id(id: ObjectId) -> Self;
}

// ===== Id =====

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
pub struct ObjectId(NonZeroU32);

impl ObjectId {
    pub const fn new(id: u32) -> Option<Self> {
        match NonZeroU32::new(id) {
            Some(x) => Some(Self(x)),
            None => None
        }
    }

    pub const fn from_ne_bytes(ne: [u8; 4]) -> Option<Self> {
        Self::new(u32::from_ne_bytes(ne))
    }

    pub const fn wl_display() -> Self {
        const { Self(NonZeroU32::new(1).unwrap()) }
    }

    /// Returns `true` if id is special id for `wl_display`.
    pub const fn is_display(self) -> bool {
        self.0.get() == 1
    }

    /// Returns ID as `u32`.
    pub const fn to_u32(self) -> u32 {
        self.0.get()
    }

    /// Returns the memory representation of this integer as a byte array in native byte order
    pub const fn to_ne_bytes(self) -> [u8; 4] {
        self.0.get().to_ne_bytes()
    }
}

impl PartialEq<u32> for ObjectId {
    fn eq(&self, other: &u32) -> bool {
        self.0.get() == *other
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// ===== NewId =====

pub struct NewId<T> {
    id: ObjectId,
    _p: std::marker::PhantomData<T>,
}

impl<T> NewId<T> {
    pub const fn new(id: ObjectId) -> Self {
        Self {
            id,
            _p: std::marker::PhantomData,
        }
    }

    pub fn from_ne_bytes(ne: [u8; 4]) -> Option<Self> {
        ObjectId::from_ne_bytes(ne).map(Self::new)
    }

    pub const fn object_id(&self) -> ObjectId {
        self.id
    }

    pub fn get(self) -> T
    where
        T: FromObjectId,
    {
        T::from_id(self.id)
    }
}

impl<T> Copy for NewId<T> {}

impl<T> Clone for NewId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> std::fmt::Debug for NewId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewId").field("id", &self.id).finish()
    }
}
