use std::num::NonZeroU32;

// ===== traits =====

// the naming follows fd convention: FromRawFd and AsRawFd

/// Create an object with given object id.
pub trait FromObjectId {
    /// Create this object with given object id.
    fn from_object_id(id: ObjectId) -> Self;
}

/// Object that contains an object id.
pub trait AsObjectId {
    /// Returns this object id.
    fn as_object_id(&self) -> ObjectId;
}

// ===== ObjectId =====

/// Object ID.
///
/// The IDs are allocated by the entity creating the object (either client or server). IDs allocated
/// by the client are in the range `[1, 0xfeffffff]` while IDs allocated by the server are in the
/// range `[0xff000000, 0xffffffff]`.
///
/// The `0` ID is reserved to represent a null or non-existent object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ObjectId(NonZeroU32);

impl ObjectId {
    /// Creates object id from `u32`.
    ///
    /// Returns `None` if the id is `0`.
    #[inline]
    pub const fn new(id: u32) -> Option<Self> {
        match NonZeroU32::new(id) {
            Some(x) => Some(Self(x)),
            None => None
        }
    }

    /// Returns object id for `wl_display`.
    #[inline]
    pub const fn wl_display() -> Self {
        const { Self(NonZeroU32::new(1).unwrap()) }
    }

    /// Returns `true` if id is special id for `wl_display`.
    #[inline]
    pub const fn is_display(self) -> bool {
        self.0.get() == 1
    }

    /// Returns ID as `u32`.
    #[inline]
    pub const fn to_u32(self) -> u32 {
        self.0.get()
    }

    /// Returns the memory representation of this integer as a byte array in native byte order
    #[inline]
    pub const fn to_ne_bytes(self) -> [u8; 4] {
        self.0.get().to_ne_bytes()
    }
}

impl PartialEq<u32> for ObjectId {
    #[inline]
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

/// A new id for given object.
///
/// Create the actual object using [`NewId::get`].
pub struct NewId<T> {
    id: ObjectId,
    _p: std::marker::PhantomData<T>,
}

impl<T> NewId<T> {
    /// Creates `NewId` from given object id.
    #[inline]
    pub const fn new(id: ObjectId) -> Self {
        Self {
            id,
            _p: std::marker::PhantomData,
        }
    }

    /// Returns the new object id.
    #[inline]
    pub const fn object_id(&self) -> ObjectId {
        self.id
    }

    /// Create the actual object.
    #[inline]
    pub fn get(self) -> T
    where
        T: FromObjectId,
    {
        T::from_object_id(self.id)
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
