use std::process::ExitCode;

use todex::sys::buffer::Buffer;
use todex::wayland::{self, AsInterface, AsOpCode, Decode, Frame, Interface, OpCode, WlError};
use todex::compositor::clients::ClientMut;
use todex::compositor::seat::Seat;
use todex::log;

mod rt;

mod prelude {
    pub(crate) use todex::wayland::{self, Interface, WlError};
    pub(crate) use todex::compositor::clients::ClientMut;

    pub(crate) use crate::{Compositor, RequestHandler};
}

mod wl_display;
mod wl_registry;
mod wl_compositor;
mod wl_seat;
mod wl_data_source;
mod wl_data_device_manager;

fn main() -> ExitCode {
    let _guard = log::init();
    <_>::from(rt::event_loop().is_err() as u8)
}

static GLOBALS: [(&str, u32, Interface); 5] = [
    ("wl_compositor", 7, Interface::WlCompositor),
    ("wl_shm", 2, Interface::WlShm),
    ("wl_data_device_manager", 4, Interface::WlDataDeviceManager),
    ("wl_seat", 10, Interface::WlSeat),
    ("xdg_wm_base", 7, Interface::XdgWmBase),
];

pub struct Compositor {
    seat: Seat,
}

pub fn router(
    read_buf: &mut Buffer,
    client: &mut ClientMut,
    compositor: &mut Compositor,
) -> Result<(), WlError> {
    use wayland::interfaces::*;

    let (id, op, frame) = Frame::new(read_buf)?;
    let interface = if id.is_display() {
        Interface::WlDisplay
    } else {
        match client.get_object(id) {
            Some(object) => object.interface(),
            None => return Err(WlError::UnknownObject),
        }
    };

    macro_rules! handle_me {
        (@OP $iface:ident { $($req:ident $call:ident),* $(, $(.. $fb:ident)? $(,)? )? }) => {
            match <_>::try_from_op(op)? {
                $(
                    $iface::RequestOp::$req => handle_me!(@CALL $iface $req $call),
                )*
                $(
                    $(
                        op => compositor.$fb(interface, op, client),
                    )?
                )?
            }
        };
        (@CALL $iface:ident $req:ident $call:ident) => {
            compositor.$call($iface::$req::decode_with(frame)?, client)
        };
        ($($iface:ident {$($tt:tt)*})*) => {
            match interface {
                $(
                    Interface::$iface => handle_me!(@OP $iface {$($tt)*}),
                )*
                iface => compositor.todo_interface(iface, op, client),
            }
        };
    }

    // one can use goto definition in the method call
    handle_me! {
        WlDisplay {
            Sync handle,
            GetRegistry handle
        }
        WlRegistry {
            Bind handle
        }
        WlCompositor {
            CreateSurface handle,
            CreateRegion todo,
            Release todo,
        }
        WlShm {
            CreatePool todo,
            Release todo,
        }
        WlSeat {
            GetPointer todo,
            GetKeyboard handle,
            // GetTouch todo,
            // Release todo,
            .. todo_interface,
        }
        WlDataSource {
            Offer handle,
            Destroy todo,
            SetActions todo,
        }
        WlDataDeviceManager {
            CreateDataSource handle,
            GetDataDevice handle,
            Release todo,
        }
    }
}

trait RequestHandler<Request>: Sized {
    fn handle(&mut self, request: Request, client: &mut ClientMut) -> Result<(), WlError>;
}

impl Compositor {
    fn todo_interface<Op: std::fmt::Display>(
        &mut self,
        interface: Interface,
        op: Op,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        client.log_error(format_args!("`{interface}::{op}` is not yet implemented"));
        WlError::todo()
    }

    fn todo<R: AsOpCode + AsInterface>(
        &mut self,
        _: R,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        let (op, iface) = (R::OPNAME, R::INTERFACE);
        client.log_error(format_args!("`{iface}::{op}` is not yet implemented"));
        WlError::todo()
    }
}

// ===== Errors =====

pub struct FatalError;

impl<E: std::fmt::Display> From<E> for FatalError {
    fn from(value: E) -> Self {
        log::error!("{value}");
        Self
    }
}
