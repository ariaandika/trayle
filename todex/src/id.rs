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
#[derive(Debug)]
#[repr(transparent)]
pub struct Id(NonZeroU32);

impl Id {
    /// Returns ID as `u32`.
    pub const fn as_u32(&self) -> u32 {
        self.0.get()
    }
}

/// New ID argument without specified interface.
#[derive(Debug)]
pub struct NewId {
    id: NonZeroU32,
}

impl NewId {
    /// Returns ID as `u32`.
    pub const fn as_u32(&self) -> u32 {
        self.id.get()
    }
}

/// New ID argument with specified interface.
#[derive(Debug)]
pub struct NewIdOf<T> {
    id: NonZeroU32,
    _p: std::marker::PhantomData<T>,
}

impl<T> NewIdOf<T> {
    /// Returns ID as `u32`.
    pub const fn as_u32(&self) -> u32 {
        self.id.get()
    }
}

/// Object ID argument.
#[derive(Debug)]
pub struct ObjectId<T> {
    id: NonZeroU32,
    _p: std::marker::PhantomData<T>,
}


impl<T> ObjectId<T> {
    /// Returns ID as `u32`.
    pub const fn as_u32(&self) -> u32 {
        self.id.get()
    }
}
