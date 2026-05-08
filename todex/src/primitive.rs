//! Wayland primitive types.
//!
//! Contains:
//!
//! - [`Id`], wayland object ID
//! - [`Array`], wayland array type
//! - [`NewId`], wayland new_id type
//! - [`Encode`], wayland new_id type
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
    pub fn new(id: u32) -> Option<Self> {
        NonZeroU32::new(id).map(Self)
    }

    pub(crate) const fn new_non_zero(id: NonZeroU32) -> Self {
        Self(id)
    }

    pub const fn wl_display() -> Id {
        Id(NonZeroU32::new(1).unwrap())
    }

    /// Returns ID as `u32`.
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0.get()
    }
}

/// Wayland array.
#[repr(transparent)]
pub struct Array([u8]);

impl Array {
    pub(crate) const fn new(array: &[u8]) -> &Self {
        unsafe { std::mem::transmute(array) }
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Deref for Array {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ===== NewId =====

pub struct Implicit<'a> {
    name: &'a str,
    version: u32,
}

pub trait NewIdInterface {
    fn name(&self) -> &str;

    fn version(&self) -> u32;
}

impl NewIdInterface for Implicit<'_> {
    fn name(&self) -> &str {
        self.name
    }

    fn version(&self) -> u32 {
        self.version
    }
}

/// Wayland `new_id` argument without inferred interface.
#[derive(Debug)]
pub struct NewId<I> {
    interface: I,
    id: NonZeroU32,
}

impl<I> NewId<I> {
    #[inline]
    pub const fn new(id: NonZeroU32, interface: I) -> Self {
        Self { interface, id }
    }

    #[inline]
    pub const fn new_implicit<'a>(
        name: &'a str,
        version: u32,
        id: NonZeroU32,
    ) -> NewId<Implicit<'a>> {
        NewId {
            interface: Implicit { name, version },
            id,
        }
    }

    #[inline]
    pub const fn id(&self) -> Id {
        Id::new_non_zero(self.id)
    }
}

impl<I: NewIdInterface> NewId<I> {
    #[inline]
    pub fn name(&self) -> &str {
        self.interface.name()
    }

    #[inline]
    pub fn version(&self) -> u32 {
        self.interface.version()
    }
}
