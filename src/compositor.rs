use todex::collections::slab::Slab;
use todex::sys::bytes::Bytes;
use todex::sys::cmsg::Cmsg;
use todex::wayland::primitives::ObjectId;
use todex::wayland::global::{Global, global_of};
use todex::wayland::wl_display::Error as GlobalError;
use todex::wayland::{self, AsInterface, OpCode, Decode, WlError, WlMessage};
use todex::wayland::{DecodeError, Frame, Interface, Operation};

use crate::error::FatalError;
use crate::seat::Seat;
use crate::client::ClientMut;
use crate::log;
use crate::wayland::surface::Surface;

mod prelude {
    pub(super) use todex::wayland::{self, Interface, WlError, Operation};
    pub(super) use crate::client::ClientMut;
    pub(super) use super::{Compositor, RequestHandler};
}

mod wl_display;
mod wl_shm;
mod wl_seat;
mod wl_data_source;
mod wl_data_device_manager;
mod wl_surface;
mod xdg_shell;

// ===== traits =====

trait RequestHandler<Request>: Sized {
    fn handle(
        &mut self,
        request: Operation<Request>,
        client: &mut ClientMut,
    ) -> Result<(), WlError>;
}

trait BindEffect<Interface> {
    fn bind(&mut self, object: Interface, client: &mut ClientMut) -> Result<(), WlError>;
}

// ===== impl =====

static GLOBALS: [Global; 5] = {
    use wayland::interfaces::*;
    [
        global_of::<WlCompositor::WlCompositor>(),
        global_of::<WlShm::WlShm>(),
        global_of::<WlDataDeviceManager::WlDataDeviceManager>(),
        global_of::<WlSeat::WlSeat>(),
        global_of::<XdgWmBase::XdgWmBase>(),
    ]
};

pub struct Compositor {
    seat: Seat,
    surfaces: Slab<Surface>,
}

impl Compositor {
    pub fn new() -> Result<Self, FatalError> {
        Ok(Self {
            seat: Seat::new()?,
            surfaces: Slab::with_capacity(8),
        })
    }

    pub fn has_frame(&self, read_buf: &Bytes) -> bool {
        Frame::has_frame(read_buf)
    }

    pub fn route(
        &mut self,
        read_buf: &mut Bytes,
        read_fd: &mut Cmsg,
        client: &mut ClientMut,
    ) -> Result<(), ()> {
        match route(self, read_buf, read_fd, client) {
            Ok(true) => Ok(()),
            Ok(false) => Err(()),
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
    read_buf: &mut Bytes,
    read_fd: &mut Cmsg,
    client: &mut ClientMut,
) -> Result<bool, WlError> {
    use wayland::interfaces::*;

    let (id, op, frame) = Frame::new(read_buf, read_fd)?;
    let object = client.objects.get_mut(id)?;

    macro_rules! handle_me {
        (@OP $iface:ident { $($req:ident $($call:ident)?),* $(, $(.. $fb:ident)? $(,)? )? }) => {
            match <_>::from_op(op).ok_or(DecodeError::UnknownOpCode)? {
                $($iface::RequestOp::$req => handle_me!(@CALL $iface $req $($call)?),)*
                $($(op => return Err(compositor.$fb(object.interface(), op, client)),)?)?
            }
        };
        (@CALL $iface:ident $req:ident) => {{
            let err = compositor.todo($iface::$req::decode_with(frame)?, client);
            client.send(GlobalError::new(id, err.code(), err.message()));
            Ok(false)
        }};
        (@CALL $iface:ident $req:ident $call:ident) => {{
            let message = log::recv_message($iface::$req::decode_with(frame)?, client);
            match RequestHandler::$call(
                compositor,
                Operation::new(id, object.version(), message),
                client,
            ) {
                Ok(_) => {
                    if <$iface::$req as WlMessage>::IS_DESTRUCTOR {
                        client.delete_id(id);
                    }
                    Ok(true)
                }
                Err(err) => {
                    log::handler_error(object.interface(), op, err, client);
                    client.send(GlobalError::new(id, err.code(), err.message()));
                    Ok(false)
                }
            }
        }};
        ($($iface:ident {$($tt:tt)*})*) => {
            match object.interface() {
                $(Interface::$iface => handle_me!(@OP $iface {$($tt)*}),)*
                iface => return Err(compositor.todo_interface(iface, op, client)),
            }
        };
    }

    // one can use goto definition in the method call
    //
    // other forwarded to `Compositor::todo`
    handle_me! {
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
            CreateRegion,
            Release,
        }
        // ===== shm =====
        WlShm {
            CreatePool,
            Release,
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
            Destroy,
            SetActions,
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
    }
}

impl Compositor {
    pub fn todo_interface<Op: std::fmt::Display>(
        &mut self,
        interface: Interface,
        op: Op,
        client: &mut ClientMut,
    ) -> WlError {
        log::todo_interface(interface, op, client);
        WlError::NotYetImplemented
    }

    pub fn todo<R: WlMessage>(
        &mut self,
        req: R,
        client: &mut ClientMut,
    ) -> WlError {
        log::todo_operation(req, client);
        WlError::NotYetImplemented
    }
}
