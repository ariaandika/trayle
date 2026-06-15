use todex::sys::buffer::Buffer;
use todex::wayland::{self, AsInterface, AsOpCode, Decode, DecodeError, OpCode, WlError};
use todex::wayland::{Frame, Interface, ObjectId};
use todex::wayland::wl_display::Error as GlobalError;

use crate::error::FatalError;
use crate::seat::Seat;
use crate::client::ClientMut;
use crate::log;

mod prelude {
    pub(super) use todex::wayland::{self, Interface, WlError};
    pub(super) use crate::client::ClientMut;
    pub(super) use super::{Compositor, RequestHandler};
}

mod wl_display;
mod wl_seat;
mod wl_data_source;
mod wl_data_device_manager;
mod wl_surface;
mod xdg_shell;

trait RequestHandler<Request>: Sized {
    fn handle(&mut self, request: Request, client: &mut ClientMut) -> Result<(), WlError>;
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

impl Compositor {
    pub fn new() -> Result<Self, FatalError> {
        Ok(Self { seat: Seat::new()? })
    }

    pub fn has_frame(&self, read_buf: &Buffer) -> bool {
        Frame::has_frame(read_buf)
    }

    pub fn route(&mut self, read_buf: &mut Buffer, client: &mut ClientMut) -> Result<(), ()> {
        match route(self, read_buf, client) {
            Ok(()) => Ok(()),
            Err(err) => {
                log::malformed_message(err, client);
                client.send(GlobalError::new(
                    ObjectId::wl_display(),
                    err.code(),
                    err.message(),
                ));
                Err(())
            }
        }
    }
}

fn route(
    compositor: &mut Compositor,
    read_buf: &mut Buffer,
    client: &mut ClientMut,
) -> Result<(), WlError> {
    use wayland::interfaces::*;

    let (id, op, frame) = Frame::new(read_buf)?;
    let interface = client.objects.get_mut(id)?;

    macro_rules! handle_me {
        (@OP $iface:ident { $($req:ident $call:ident),* $(, $(.. $fb:ident)? $(,)? )? }) => {
            match <_>::from_op(op).ok_or(DecodeError::UnknownOpCode)? {
                $($iface::RequestOp::$req => handle_me!(@CALL $iface $req $call),)*
                $($(op => compositor.$fb(interface, op, client),)?)?
            }
        };
        (@CALL $iface:ident $req:ident $call:ident) => {
            compositor.$call(
                log::recv_message($iface::$req::decode_with(frame)?, client),
                client,
            )
        };
        ($($iface:ident {$($tt:tt)*})*) => {
            match interface {
                $(Interface::$iface => handle_me!(@OP $iface {$($tt)*}),)*
                iface => compositor.todo_interface(iface, op, client),
            }
        };
    }

    // one can use goto definition in the method call
    let result = handle_me! {
        // ===== core =====
        WlDisplay {
            Sync handle,
            GetRegistry handle
        }
        WlRegistry {
            Bind handle
        }
        // ===== compositor =====
        WlCompositor {
            CreateSurface handle,
            CreateRegion todo,
            Release todo,
        }
        // ===== shm =====
        WlShm {
            CreatePool todo,
            Release todo,
        }
        // ===== seat =====
        WlSeat {
            GetPointer handle,
            GetKeyboard handle,
            GetTouch handle,
            Release handle,
        }
        // ===== data =====
        WlDataSource {
            Offer handle,
            Destroy todo,
            SetActions todo,
        }
        WlDataDeviceManager {
            CreateDataSource handle,
            GetDataDevice handle,
            Release handle,
        }
        // ===== surface =====
        WlSurface {
            Commit handle,
            .. todo_interface,
        }
        // ===== xdg shell =====
        XdgWmBase {
            Destroy handle,
            CreatePositioner handle,
            GetXdgSurface handle,
            Pong handle,
        }
        XdgSurface {
            Destroy handle,
            GetToplevel handle,
            GetPopup handle,
            SetWindowGeometry handle,
            AckConfigure handle,
        }
        XdgToplevel {
            SetTitle handle,
            SetAppId handle,
            .. todo_interface,
        }
    };

    if let Err(err) = result {
        log::handler_error(interface, op, err, client);
        client.send(GlobalError::new(id, err.code(), err.message()));
    }

    Ok(())
}

impl Compositor {
    pub fn todo_interface<Op: std::fmt::Display>(
        &mut self,
        interface: Interface,
        op: Op,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        log::todo_interface(interface, op, client);
        Err(WlError::NotYetImplemented)
    }

    pub fn todo<R: AsOpCode + AsInterface>(
        &mut self,
        req: R,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        log::todo_operation(req, client);
        Err(WlError::NotYetImplemented)
    }
}
