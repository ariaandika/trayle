use crate::drm::ioctl::*;
use crate::drm::Handle;
use crate::drm::property::Property;

#[derive(Debug, Default)]
pub struct AtomicRequest {
    objects: Vec<u32>,
    prop_count: Vec<u32>,
    props: Vec<u32>,
    prop_values: Vec<u64>,
}

impl AtomicRequest {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn push<R>(&mut self, handle: Handle<R>, prop_id: u32, prop_val: u64) {
        self.push_inner(handle.into(), prop_id, prop_val);
    }

    #[inline]
    pub fn add_prop<R, P: Into<u64>>(&mut self, handle: Handle<R>, prop: Property<P>) {
        self.push_inner(handle.into(), prop.id, prop.value.into());
    }

    fn push_inner(&mut self, object_id: u32, prop_id: u32, prop_val: u64) {
        if self.objects.last() == Some(&object_id) {
            *self.prop_count.last_mut().unwrap() += 1;
        } else {
            self.objects.push(object_id);
            self.prop_count.push(1);
        }
        self.props.push(prop_id);
        self.prop_values.push(prop_val);
    }
}

impl AtomicRequest {
    #[inline]
    pub fn commit<D: AsFd>(self, flags: CommitFlags, device: &mut D) -> Result<(), ErrCode> {
        self.commit_inner(flags, device.as_fd())
    }

    #[inline]
    fn commit_inner(self, flags: CommitFlags, fd: BorrowedFd) -> Result<(), ErrCode> {
        drm_mode_atomic {
            flags,
            count_objs: self.objects.len() as _,
            objs_ptr: self.objects.as_ptr() as _,
            count_props_ptr: self.prop_count.as_ptr() as _,
            props_ptr: self.props.as_ptr() as _,
            prop_values_ptr: self.prop_values.as_ptr() as _,
            reserved: 0,
            user_data: 0,
        }
        .ioctl(fd)
    }
}

// ===== CommitFlags =====

/// Atomic Commit Flags.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CommitFlags(u32);

macro_rules! impl_ops {
    ($(impl $tr:ident::$fn:ident;)*) => {$(
        impl std::ops::$tr for CommitFlags {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self(self.0.$fn(rhs.0))
            }
        }
    )*};
}
impl_ops! {
    impl BitOr::bitor;
    impl BitXor::bitxor;
    impl BitAnd::bitand;
}

// #define DRM_MODE_ATOMIC_FLAGS (\
//         DRM_MODE_PAGE_FLIP_EVENT |\
//         DRM_MODE_PAGE_FLIP_ASYNC |\
//         DRM_MODE_ATOMIC_TEST_ONLY |\
//         DRM_MODE_ATOMIC_NONBLOCK |\
//         DRM_MODE_ATOMIC_ALLOW_MODESET)

impl CommitFlags {
    /// Request that the kernel sends back a vblank event (see struct drm_event_vblank) with the
    /// `DRM_EVENT_FLIP_COMPLETE` type when the page-flip is done.
    ///
    /// When used with atomic uAPI, one event will be delivered per CRTC included in the atomic
    /// commit. A CRTC is included in an atomic commit if one of its properties is set, or if a
    /// property is set on a connector or plane linked via the CRTC_ID property to the CRTC. At
    /// least one CRTC must be included, and all pulled in CRTCs must be either previously or newly
    /// powered on (in other words, a powered off CRTC which stays off cannot be included in the
    /// atomic commit).
    pub const PAGE_FLIP_EVENT: Self = Self(0x01);

    /// Request that the page-flip is performed as soon as possible, ie. with no
    /// delay due to waiting for vblank.
    ///
    /// This may cause tearing to be visible on the screen.
    ///
    /// When used with atomic uAPI, the driver will return an error if the hardware
    /// doesn't support performing an asynchronous page-flip for this update.
    /// User-space should handle this, e.g. by falling back to a regular page-flip.
    ///
    /// Note, some hardware might need to perform one last synchronous page-flip
    /// before being able to switch to asynchronous page-flips. As an exception,
    /// the driver will return success even though that first page-flip is not
    /// asynchronous.
    pub const PAGE_FLIP_ASYNC: Self = Self(0x02);

    /// Do not apply the atomic commit, instead check whether the hardware supports this
    /// configuration.
    pub const TEST_ONLY: Self = Self(0x0100);

    /// Do not block while applying the atomic commit.
    ///
    /// The atomic request returns immediately instead of waiting for the changes to be applied in
    /// hardware.
    ///
    /// Note, the driver will still check that the update can be applied before retuning.
    pub const NONBLOCK: Self = Self(0x0200);

    /// Allow the update to result in temporary or transient visible artifacts while the update is
    /// being applied.
    ///
    /// Applying the update may also take significantly more time than a page flip. All visual
    /// artifacts will disappear by the time the update is completed, as signalled through the
    /// vblank event's timestamp (see struct drm_event_vblank).
    ///
    /// This flag must be set when the KMS update might cause visible artifacts. Without this flag
    /// such KMS update will return a EINVAL error. What kind of update may cause visible artifacts
    /// depends on the driver and the hardware. User-space that needs to know beforehand if an
    /// update might cause visible artifacts can use [`CommitFlags::TEST_ONLY`] without
    /// [`CommitFlags::MODESET`] to see if it fails.
    ///
    /// To the best of the driver's knowledge, visual artifacts are guaranteed to not appear when
    /// this flag is not set. Some sinks might display visual artifacts outside of the driver's
    /// control.
    pub const ALLOW_MODESET: Self = Self(0x0400);
}

// ===== syscall =====

#[repr(C)]
struct drm_mode_atomic {
    flags: CommitFlags,
    count_objs: __u32,
    objs_ptr: __u64,
    count_props_ptr: __u64,
    props_ptr: __u64,
    prop_values_ptr: __u64,
    reserved: __u64,
    user_data: __u64,
}

impl DrmIoctl for drm_mode_atomic {
    /// DRM_IOCTL_MODE_ATOMIC
    const CODE: u32 = 0xBC;
}
