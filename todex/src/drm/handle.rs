use std::fmt;
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::os::fd::AsFd;

use crate::sys::error::ErrCode;
use crate::drm::property::{Properties, RawProperties, WithProperties};
use crate::drm::resource::Resource;

/// Resource object id.
///
/// This uses `repr(transparent)` and has the representation of a `Option<NonZeroU32>`, so it can be
/// used in FFI in places where a resource object id is used.
#[repr(transparent)]
pub struct Handle<T> {
    id: NonZeroU32,
    _p: PhantomData<fn() -> T>,
}

impl<R> Handle<R> {
    /// Create [`Handle`] from raw value.
    ///
    /// Returns [`None`] if `id` is `0`.
    pub(crate) fn from_raw(id: u32) -> Option<Self> {
        NonZeroU32::new(id).map(|id| Self {
            id,
            _p: PhantomData,
        })
    }
}

impl<R: Resource> Handle<R> {
    /// Request the actual resource.
    ///
    /// Forward call to [`Resource::get_resource`].
    #[inline]
    pub fn get_resource<D: AsFd>(self, device: &D) -> Result<R, ErrCode> {
        R::get_resource(self, device)
    }

    /// Request the typed resource properties.
    ///
    /// Forward call to [`Properties::get_properties`].
    #[inline]
    pub fn get_properties<D: AsFd>(self, device: &D) -> Result<R::Properties, ErrCode>
    where
        R: WithProperties,
    {
        R::Properties::get_properties(self, device)
    }

    /// Request the raw resource properties.
    #[inline]
    pub fn get_raw_properties<D: AsFd>(self, device: &D) -> Result<RawProperties, ErrCode> {
        RawProperties::get_properties(self, device)
    }
}

// ===== traits =====

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Display for Handle<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Handle").field(&self.id).finish()
    }
}

impl<T> From<Handle<T>> for u32 {
    #[inline]
    fn from(value: Handle<T>) -> Self {
        value.id.get()
    }
}

impl<T> From<Handle<T>> for u64 {
    #[inline]
    fn from(value: Handle<T>) -> Self {
        value.id.get().into()
    }
}

impl<T> Eq for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
