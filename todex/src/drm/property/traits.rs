use std::os::fd::{AsFd, BorrowedFd};

use crate::drm::Handle;
use crate::sys::error::ErrCode;
use crate::drm::property::{IntoIter, NamedRawProperty, RawProperties};
use crate::drm::resource::Resource;

/// A resource that have a typed properties.
pub trait WithProperties: Sized + Resource {
    /// The properties of this resource.
    type Properties: Properties<Self>;
}

pub trait Properties<R: Resource>: Sized {
    fn from_raw_properties(props: PropertyIter<'_>) -> Result<Self, ErrCode>;

    #[inline]
    fn get_properties<D: AsFd>(handle: Handle<R>, device: &D) -> Result<Self, ErrCode> {
        RawProperties::get_properties(handle, device)
            .and_then(|p| Self::from_raw_properties(PropertyIter::new(p, device.as_fd())))
    }
}

// ===== PropertyIter =====

pub struct PropertyIter<'a> {
    iter: IntoIter,
    fd: BorrowedFd<'a>,
}

impl<'a> PropertyIter<'a> {
    fn new(props: RawProperties, fd: BorrowedFd<'a>) -> Self {
        let iter = props.into_iter();
        Self { iter, fd }
    }

    #[inline]
    #[expect(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<NamedRawProperty>, ErrCode> {
        let Some(prop) = self.iter.next() else {
            return Ok(None);
        };
        prop.get_inner(self.fd).map(Some)
    }
}
