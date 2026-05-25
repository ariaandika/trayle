pub use id::Id;
pub use op::Op;
pub use error::WlError;
pub use interface::{Interface, InterfaceOp};

mod id;
mod op;
mod error;
mod decode;
mod encode;

pub mod wl_display;
pub mod wl_registry;

mod interface;

mod prelude {
    pub use super::WlObject;
    pub use super::id::Id;
    pub use super::op::FromOpCode;
    pub use super::error::WlError;
    pub use super::decode::{Reader, Decode, Decoder};
    pub use super::encode::{PtrWrite, Encoder};
    pub use super::interface::Interface;
    pub use crate::buffer::Buffer;

    pub(super) use super::roundup4;
}

pub trait WlObject {
    const INTERFACE: Interface;

    fn id(&self) -> Id;
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
