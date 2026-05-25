pub use id::Id;
pub use error::WlError;

mod id;
mod error;
mod decode;
mod encode;

pub mod wl_display;
pub mod wl_registry;

mod prelude {
    pub use super::{WlObject, Interface};
    pub use super::id::Id;
    pub use super::error::WlError;
    pub use super::decode::{Reader, Decode, Decoder};
    pub use super::encode::{PtrWrite, Encoder};
    pub use crate::buffer::Buffer;

    pub(super) use super::roundup4;
}

pub trait WlObject {
    const INTERFACE: Interface;

    fn id(&self) -> Id;
}

// commented entry are exists but never constructed
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
#[allow(dead_code)]
pub enum Interface {
    // WlDisplay,
    WlRegistry,
    // WlCallback,
    WlCompositor,
    WlShmPool,
    WlShm,
    WlBuffer,
    WlDataOffer,
    WlDataSource,
    WlDataDevice,
    WlDataDeviceManager,
    // WlShell, /// deprecated
    WlShellSurface,
    WlSurface,
    WlSeat,
    WlPointer,
    WlKeyboard,
    WlTouch,
    WlOutput,
    WlRegion,
    WlSubCompositor,
    WlSubSurface,
    WlFixes,
    ZwpLinuxDmabufV1,
    ZwpLinuxBufferParamsV1,
    ZwpLinuxDmabufFeedbackV1,
    XdgWmBase,
    XdgPositioner,
    XdgSurface,
    XdgToplevel,
    XdgPopup,
}

pub static GLOBALS: [(&str, u16, Interface); 9] = [
    ("wl_compositor", 7, Interface::WlCompositor),
    ("wl_shm", 2, Interface::WlShm),
    ("wl_data_device_manager", 4, Interface::WlDataDeviceManager),
    ("wl_seat", 10, Interface::WlSeat),
    ("wl_subcompositor", 1, Interface::WlSubCompositor),
    ("wl_fixes", 2, Interface::WlFixes),
    ("zwp_linux_dmabuf_v1", 5, Interface::ZwpLinuxDmabufV1),
    ("zwp_linux_dmabuf_feedback_v1", 5, Interface::ZwpLinuxDmabufFeedbackV1),
    ("xdg_wm_base", 7, Interface::XdgWmBase),
];

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

use roundup4;
