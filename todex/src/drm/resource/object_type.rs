/// A DRM Resource object type id.
#[derive(Debug, Default, Clone, Copy)]
#[repr(transparent)]
pub struct ObjectType(u32);

// Source: `libdrm/drm/drm_mode.h`
#[expect(dead_code)]
impl ObjectType {
    pub(crate) const CRTC: Self = Self(0xcccccccc);
    pub(crate) const CONNECTOR: Self = Self(0xc0c0c0c0);
    pub(crate) const ENCODER: Self = Self(0xe0e0e0e0);
    pub(crate) const MODE: Self = Self(0xdededede);
    pub(crate) const PROPERTY: Self = Self(0xb0b0b0b0);
    pub(crate) const FB: Self = Self(0xfbfbfbfb);
    pub(crate) const BLOB: Self = Self(0xbbbbbbbb);
    pub(crate) const PLANE: Self = Self(0xeeeeeeee);
    pub(crate) const COLOROP: Self = Self(0xfafafafa);
    pub(crate) const ANY: Self = Self(0);
}

impl From<ObjectType> for u32 {
    #[inline]
    fn from(value: ObjectType) -> Self {
        value.0
    }
}
