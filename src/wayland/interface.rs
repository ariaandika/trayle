macro_rules! declare {
    ($($s:ident::$u:ident,)*;$($up:ident,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Interface {
            $($u,)*
            $($up,)*
        }

        /// Reexport interfaces as camel case.
        pub mod prelude {
            $(pub use crate::wayland::$s as $u;)*
        }
    };
}

declare! {
    wl_display::WlDisplay,
    wl_registry::WlRegistry,
    wl_callback::WlCallback,
    wl_compositor::WlCompositor,
    wl_shm::WlShm,
    wl_data_source::WlDataSource,
    wl_data_device::WlDataDevice,
    wl_data_device_manager::WlDataDeviceManager,
    wl_surface::WlSurface,
    wl_seat::WlSeat,
    wl_keyboard::WlKeyboard,
    ;
    WlShmPool,
    WlBuffer,
    WlDataOffer,
    WlPointer,
    WlTouch,
    WlOutput,
    WlRegion,
    WlSubCompositor,
    WlSubSurface,
    WlFixes,
    XdgWmBase,
    XdgPositioner,
    XdgSurface,
    XdgToplevel,
    XdgPopup,
    ZwpLinuxDmabufV1,
    ZwpLinuxBufferParamsV1,
    ZwpLinuxDmabufFeedbackV1,
}

/// Object that is a wayland interface.
pub trait AsInterface {
    /// The interface of this object is associated with.
    const INTERFACE: Interface;
}
