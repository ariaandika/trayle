use crate::wayland::{Op, wl_data_device_manager, wl_registry, wl_shm};

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

// commented entry are exists but never constructed
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
#[allow(dead_code)]
pub enum InterfaceOp {
    // WlDisplay,
    WlRegistry(Op<wl_registry::Op>),
    // WlCallback,
    WlCompositor(Op<wl_registry::Op>),
    WlShmPool(Op<wl_registry::Op>),
    WlShm(Op<wl_shm::Op>),
    WlBuffer(Op<wl_registry::Op>),
    WlDataOffer(Op<wl_registry::Op>),
    WlDataSource(Op<wl_registry::Op>),
    WlDataDevice(Op<wl_registry::Op>),
    WlDataDeviceManager(Op<wl_data_device_manager::Op>),
    // WlShell, /// deprecated
    WlShellSurface(Op<wl_registry::Op>),
    WlSurface(Op<wl_registry::Op>),
    WlSeat(Op<wl_registry::Op>),
    WlPointer(Op<wl_registry::Op>),
    WlKeyboard(Op<wl_registry::Op>),
    WlTouch(Op<wl_registry::Op>),
    WlOutput(Op<wl_registry::Op>),
    WlRegion(Op<wl_registry::Op>),
    WlSubCompositor(Op<wl_registry::Op>),
    WlSubSurface(Op<wl_registry::Op>),
    WlFixes(Op<wl_registry::Op>),
    ZwpLinuxDmabufV1(Op<wl_registry::Op>),
    ZwpLinuxBufferParamsV1(Op<wl_registry::Op>),
    ZwpLinuxDmabufFeedbackV1(Op<wl_registry::Op>),
    XdgWmBase(Op<wl_registry::Op>),
    XdgPositioner(Op<wl_registry::Op>),
    XdgSurface(Op<wl_registry::Op>),
    XdgToplevel(Op<wl_registry::Op>),
    XdgPopup(Op<wl_registry::Op>),
}

impl Interface {
    pub fn op(self) -> InterfaceOp {
        macro_rules! matcher {
            ($($v:ident),*) => {
                match self {
                    $(
                        Self::$v => InterfaceOp::$v(Op::new()),
                    )*
                }
            };
        }
        matcher! {
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
            XdgPopup
        }
    }
}
