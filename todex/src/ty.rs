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
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Id(NonZeroU32);

impl Id {
    /// Returns ID as `u32`.
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0.get()
    }
}

/// Wayland array.
#[repr(transparent)]
pub struct Array([u8]);

impl std::ops::Deref for Array {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wayland `new_id` argument without inferred interface.
#[derive(Debug)]
pub struct NewId {
    name: &'static str,
    version: u32,
    id: NonZeroU32,
}

impl NewId {
    #[inline]
    pub fn new(name: &'static str, version: u32, id: NonZeroU32) -> Self {
        Self { name, version, id }
    }

    pub(crate) fn name(&self) -> &'static str {
        self.name
    }

    pub(crate) fn version(&self) -> u32 {
        self.version
    }

    pub(crate) fn id_non_zero(&self) -> NonZeroU32 {
        self.id
    }
}

