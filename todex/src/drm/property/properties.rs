use std::ffi::CStr;
use std::{iter, slice, vec};

use crate::drm::ioctl::*;
use crate::drm::handle::Handle;
use crate::drm::property::RawProperty;
use crate::drm::resource::{ObjectType, Resource};

/// Pairs of raw property id and value.
pub struct RawProperties {
    props: Box<[u32]>,
    prop_vals: Box<[u64]>,
}

impl RawProperties {
    pub(crate) fn get_properties<R, D>(handle: Handle<R>, device: &D) -> Result<Self, ErrCode>
    where
        R: Resource,
        D: AsFd,
    {
        Self::get_properties_inner(handle.into(), device.as_fd(), R::OBJECT_TYPE)
    }

    fn get_properties_inner(id: u32, fd: BorrowedFd, ty: ObjectType) -> Result<Self, ErrCode> {
        let mut io = drm_mode_obj_get_properties {
            obj_id: id,
            obj_type: ty,
            ..<_>::default()
        };
        io.ioctl(fd)?;
        let mut props = Box::new_uninit_slice(io.count_props as _);
        let mut prop_vals = Box::new_uninit_slice(io.count_props as _);
        io.props_ptr = props.as_mut_ptr() as _;
        io.prop_values_ptr = prop_vals.as_mut_ptr() as _;
        io.ioctl(fd)?;
        unsafe {
            Ok(Self {
                props: props.assume_init(),
                prop_vals: prop_vals.assume_init(),
            })
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        self.into_iter()
    }
}

impl RawProperties {
    pub fn collect<D, O>(self, device: &D) -> Result<O, ErrCode>
    where
        D: AsFd,
        O: FromIterator<(Box<CStr>, RawProperty)>,
    {
        self.collect_inner(device.as_fd()).collect()
    }

    fn collect_inner(
        self,
        fd: BorrowedFd,
    ) -> impl Iterator<Item = Result<(Box<CStr>, RawProperty), ErrCode>> {
        self.props
            .into_iter()
            .zip(self.prop_vals)
            .map(move |(id, value)| {
                let raw_prop = RawProperty { id, value };
                let prop = raw_prop.get_inner(fd)?;
                Ok((prop.name, raw_prop))
            })
    }
}

pub type Iter<'a> = iter::Map<
    iter::Zip<slice::Iter<'a, u32>, slice::Iter<'a, u64>>,
    fn((&'a u32, &'a u64)) -> RawProperty,
>;

impl<'a> IntoIterator for &'a RawProperties {
    type Item = RawProperty;

    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.props
            .iter()
            .zip(&self.prop_vals)
            .map(|(&id, &value)| RawProperty { id, value })
    }
}

pub type IntoIter =
    iter::Map<iter::Zip<vec::IntoIter<u32>, vec::IntoIter<u64>>, fn((u32, u64)) -> RawProperty>;

impl IntoIterator for RawProperties {
    type Item = RawProperty;

    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.props
            .into_iter()
            .zip(self.prop_vals)
            .map(|(id, value)| RawProperty { id, value })
    }
}

// ===== syscall =====

#[derive(Default)]
#[repr(C)]
struct drm_mode_obj_get_properties {
    props_ptr: __u64,
    prop_values_ptr: __u64,
    count_props: __u32,
    obj_id: __u32,
    obj_type: ObjectType,
}

impl DrmIoctl for drm_mode_obj_get_properties {
    /// DRM_IOCTL_MODE_OBJ_GETPROPERTIES
    const CODE: u32 = 0xB9;
}
