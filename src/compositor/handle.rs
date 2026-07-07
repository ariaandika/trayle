use todex::wayland::interface::*;

use crate::handle::WithHandle;
use crate::shm::{Buffer, ShmPool};
use crate::surface::{Surface, XdgSurface as XdgSurfaceData};

macro_rules! handle {
    ($ty:ty, $h:ty) => {
        impl WithHandle for $ty {
            type Handle = $h;
        }
    };
    ($ty:ty) => {
        impl WithHandle for $ty {
            type Handle = ();
        }
    };
}
handle!(WlDisplay);
handle!(WlRegistry);
handle!(WlCallback);
handle!(WlCompositor);
handle!(WlShmPool, ShmPool);
handle!(WlShm);
handle!(WlBuffer, Buffer);
handle!(WlDataOffer);
handle!(WlDataSource);
handle!(WlDataDevice);
handle!(WlDataDeviceManager);
handle!(WlSurface, Surface);
handle!(WlSeat);
handle!(WlPointer);
handle!(WlKeyboard);
handle!(WlTouch);
handle!(WlOutput);
handle!(WlRegion);
handle!(WlSubcompositor);
handle!(WlSubsurface);
handle!(XdgWmBase);
handle!(XdgPositioner);
handle!(XdgSurface, XdgSurfaceData);
handle!(XdgToplevel, XdgSurfaceData);
handle!(XdgPopup);
