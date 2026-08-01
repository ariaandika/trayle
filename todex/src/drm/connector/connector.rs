use crate::drm::ioctl::*;
use crate::drm::{Handle, Encoder};
use crate::drm::resource::{ObjectType, Resource};
use crate::drm::connector::ModeInfo;

/// DRM Connector.
///
/// A connector is the final destination of pixel-data on a device, and usually connects directly to
/// an external display device like a monitor or laptop panel. A connector can only be at‐ tached to
/// one encoder at a time. The connector is also the structure where information about the attached
/// display is kept, so it contains fields for display data, EDID data, DPMS and connection status,
/// and information about modes supported on the attached displays.
#[derive(Debug)]
pub struct Connector {
    pub handle: Handle<Connector>,
    pub ty: ConnectorType,
    pub status: Status,
    pub modes: Box<[ModeInfo]>,
    pub encoders: Box<[Handle<Encoder>]>,
}

impl Connector {
    fn get_resource(handle: Handle<Self>, fd: BorrowedFd) -> Result<Self, ErrCode> {
        // FEAT: connector: provide way to get resource without force probe
        //
        // if:
        // - count_modes = 0
        // - device is DRM master
        //
        // then the kernel will perform a forced probe on the connector to refresh the connector
        // status, modes and EDID. A forced-probe can be slow, might cause flickering and the ioctl
        // will block.
        //
        // User-space needs to force-probe connectors to ensure their metadata is up-to-date at
        // startup and after receiving a hot-plug event. User-space may perform a forced-probe when
        // the user explicitly requests it. User-space shouldn't perform a forced-probe in other
        // situations.
        let mut io = drm_mode_get_connector {
            connector_id: Some(handle),
            ..<_>::default()
        };
        io.ioctl(fd)?;
        let mut modes = Box::new_uninit_slice(io.count_modes as _);
        let mut encoders = Box::new_uninit_slice(io.count_encoders as _);
        io.modes_ptr = modes.as_mut_ptr() as _;
        io.encoders_ptr = encoders.as_mut_ptr() as _;
        io.count_props = 0;
        io.ioctl(fd)?;
        unsafe {
            Ok(Self {
                handle,
                ty: io.connector_type.value,
                status: io.connection.value,
                modes: modes.assume_init(),
                encoders: encoders.assume_init(),
            })
        }
    }
}

impl Resource for Connector {
    type Error = ErrCode;

    const OBJECT_TYPE: ObjectType = ObjectType::CONNECTOR;

    #[inline]
    fn request<D: AsFd>(handle: Handle<Self>, device: &D) -> Result<Self, Self::Error> {
        Self::get_resource(handle, device.as_fd())
    }
}

// ===== Connection =====

/// Connector status.
///
/// Try to enable `Connected` connectors first. If none, then try `Unknown` connectors.
#[derive(Debug, Clone, Copy)]
pub enum Status {
    Connected = 1,
    Disconnected = 2,
    Unknown = 3,
}

impl Status {
    /// Returns `true` if status is [`Status::Connected`].
    #[inline]
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }
}

// ===== ConnectorType =====

// source: libdrm/include/drm/drm_mode.h
// `DRM_MODE_CONNECTOR_*`

#[derive(Clone, Copy)]
pub enum ConnectorType {
    Unknown = 0,
    VGA = 1,
    DVII = 2,
    DVID = 3,
    DVIA = 4,
    Composite = 5,
    SVIDEO = 6,
    LVDS = 7,
    Component = 8,
    _9PinDIN = 9,
    DisplayPort = 10,
    HDMIA = 11,
    HDMIB = 12,
    TV = 13,
    /// Embedded Display Port.
    EDP = 14,
    VIRTUAL = 15,
    DSI = 16,
    DPI = 17,
    WRITEBACK = 18,
    SPI = 19,
    USB = 20,
}

impl ConnectorType {
    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::VGA => "VGA",
            Self::DVII => "DVI-I",
            Self::DVID => "DVI-D",
            Self::DVIA => "DVI-A",
            Self::Composite => "Composite",
            Self::SVIDEO => "SVIDEO",
            Self::LVDS => "LVDS",
            Self::Component => "Component",
            Self::_9PinDIN => "DIN",
            Self::DisplayPort => "DP",
            Self::HDMIA => "HDMI-A",
            Self::HDMIB => "HDMI-B",
            Self::TV => "TV",
            Self::EDP => "eDP",
            Self::VIRTUAL => "Virtual",
            Self::DSI => "DSI",
            Self::DPI => "DPI",
            Self::WRITEBACK => "Writeback",
            Self::SPI => "SPI",
            Self::USB => "USB",
        }
    }
}

impl std::fmt::Debug for ConnectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.name(), f)
    }
}

impl std::fmt::Display for ConnectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

// ===== syscall =====

#[derive(Default)]
#[repr(C)]
struct drm_mode_get_connector {
    encoders_ptr: __u64,
    modes_ptr: __u64,
    props_ptr: __u64,
    prop_values_ptr: __u64,
    count_modes: __u32,
    count_props: __u32,
    count_encoders: __u32,
    encoder_id: __u32,
    connector_id: Option<Handle<Connector>>,
    connector_type: PadU32<ConnectorType>,
    connector_type_id: __u32,
    connection: PadU32<Status>,
    mm_width: __u32,
    mm_height: __u32,
    subpixel: __u32,
    pad: __u32,
}

impl DrmIoctl for drm_mode_get_connector {
    /// DRM_IOCTL_MODE_GETCONNECTOR
    const CODE: u32 = 0xA7;
}
