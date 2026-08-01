use crate::drm::ioctl::*;

/// Client Capability.
#[derive(Debug, Clone, Copy)]
pub enum ClientCapability {
    /// Stereo 3D.
    ///
    /// If set to 1, the DRM core will expose the stereo 3D capabilities of the
    /// monitor by advertising the supported 3D layouts in the flags of struct
    /// drm_mode_modeinfo. See `DRM_MODE_FLAG_3D_*`.
    ///
    /// This capability is always supported for all drivers starting from kernel
    /// version 3.13.
    Stereo3d = 1,
    /// Universal Planes.
    ///
    /// If set to 1, the DRM core will expose all planes (overlay, primary, and
    /// cursor) to userspace.
    ///
    /// This capability has been introduced in kernel version 3.15. Starting from
    /// kernel version 3.17, this capability is always supported for all drivers.
    UniversalPlanes = 2,
    /// Atomic.
    ///
    /// If set to 1, the DRM core will expose atomic properties to userspace. This
    /// implicitly enables [`Capability::UniversalPlanes`] and
    /// [`Capability::AspectRatio`].
    ///
    /// If the driver doesn't support atomic mode-setting, enabling this capability
    /// will fail with -EOPNOTSUPP.
    ///
    /// This capability has been introduced in kernel version 4.0. Starting from
    /// kernel version 4.2, this capability is always supported for atomic-capable
    /// drivers.
    Atomic = 3,
    /// Aspect Ratio.
    ///
    /// If set to 1, the DRM core will provide aspect ratio information in modes.
    /// See `DRM_MODE_FLAG_PIC_AR_*`.
    ///
    /// This capability is always supported for all drivers starting from kernel
    /// version 4.18.
    AspectRatio = 4,
    /// Writeback Connectors.
    ///
    /// If set to 1, the DRM core will expose special connectors to be used for
    /// writing back to memory the scene setup in the commit. The client must enable
    /// [`Capability::Atomic`] first.
    ///
    /// This capability is always supported for atomic-capable drivers starting from
    /// kernel version 4.19.
    WritebackConnectors = 5,
    /// Cursor Plane Hotspot.
    ///
    /// Drivers for para-virtualized hardware (e.g. vmwgfx, qxl, virtio and
    /// virtualbox) have additional restrictions for cursor planes (thus
    /// making cursor planes on those drivers not truly universal,) e.g.
    /// they need cursor planes to act like one would expect from a mouse
    /// cursor and have correctly set hotspot properties.
    /// If this client cap is not set the DRM core will hide cursor plane on
    /// those virtualized drivers because not setting it implies that the
    /// client is not capable of dealing with those extra restictions.
    /// Clients which do set cursor hotspot and treat the cursor plane
    /// like a mouse cursor should set this property.
    /// The client must enable [`Capability::Atomic`] first.
    ///
    /// Setting this property on drivers which do not special case
    /// cursor planes (i.e. non-virtualized drivers) will return
    /// EOPNOTSUPP, which can be used by userspace to gauge
    /// requirements of the hardware/drivers they're running on.
    ///
    /// This capability is always supported for atomic-capable virtualized
    /// drivers starting from kernel version 6.6.
    CursorPlaneHotspot = 6,
    /// Plane Color Pipeline.
    ///
    /// If set to 1 the DRM core will allow setting the `COLOR_PIPELINE`
    /// property on a drm_plane, as well as drm_colorop properties.
    ///
    /// Setting of these plane properties will be rejected when this client
    /// cap is set:
    /// - `COLOR_ENCODING`
    /// - `COLOR_RANGE`
    ///
    /// The client must enable [`Capability::Atomic`] first.
    PlaneColorPipeline = 7,
}

impl ClientCapability {
    /// Set device client capability.
    #[inline]
    pub fn set_capability<D: AsFd>(self, value: bool, device: &D) -> Result<(), ErrCode> {
        drm_set_client_cap {
            capability: self as _,
            value: value as _,
        }
        .ioctl(device.as_fd())
    }
}

#[repr(C)]
struct drm_set_client_cap {
    capability: __u64,
    value: __u64,
}

impl DrmIoctl for drm_set_client_cap {
    /// DRM_IOCTL_SET_CLIENT_CAP
    const CODE: u32 = 0x0d;

    const IO: IoKind = IoKind::IoW;
}
