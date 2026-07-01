use std::task::Poll::{self, *};

use todex::log;
use todex::sys::bytes::Bytes;
use todex::collections::slab::Slab;
use todex::wayland;
use todex::wayland::primitives::{AsObjectId, AsVersion, Version};
use todex::wayland::object::{Global, Handle, Object, global_of};
use todex::wayland::message::{Message, WlMessage};
use todex::wayland::wire::{Payload, RawMessage};
use todex::wayland::interface::{self, DisplayId, Interface, WlInterface, AsInterface};
use todex::wayland::error::WlError;

use crate::error::FatalError;
use crate::seat::Seat;
use crate::client::ClientMut;
use crate::wayland::surface::Surface;

mod prelude {
    use super::{Version, Message};

    pub(super) use todex::wayland;
    pub(super) use todex::wayland::primitives::AsVersion;
    pub(super) use todex::wayland::object::Object;
    pub(super) use todex::wayland::message::WlMessage;
    pub(super) use todex::wayland::interface::*;
    pub(super) use todex::wayland::error::WlError;

    pub(super) use crate::client::ClientMut;

    pub(super) type Op<I> = Message<I, Version>;
    pub(super) type Operation<I> = Message<I, Version>;

    pub(super) use super::Compositor;
    pub(super) use super::{RequestHandler, RequestMsg as Req};
}

mod wl_display;
mod wl_shm;
mod wl_seat;
mod wl_data_source;
mod wl_data_device_manager;
mod wl_surface;
mod xdg_shell;

// ===== traits =====

type RequestMsg<M> = Message<M, Version>;

trait RequestHandler<Request>: Sized {
    fn handle(&mut self, req: RequestMsg<Request>, client: &mut ClientMut) -> Result<(), WlError>;
}

trait BindEffect<Interface> {
    fn bind(&mut self, obj: Object<Interface>, client: &mut ClientMut) -> Result<(), WlError>;
}

// ===== globals =====

static GLOBALS: [Global; 5] = {
    use wayland::interface::*;
    [
        global_of::<WlCompositor>(),
        global_of::<WlShm>(),
        global_of::<WlDataDeviceManager>(),
        global_of::<WlSeat>(),
        global_of::<XdgWmBase>(),
    ]
};

// ===== impl =====

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

    pub fn message(
        &mut self,
        read_buf: &mut Bytes,
        client: &mut ClientMut,
    ) -> Poll<Result<(), ()>> {
        let Ready(result) = RawMessage::decode_with(read_buf) else {
            return Pending;
        };

        let msg = match result {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("client#{} failed to decode: {err}", client.id);
                client.send_error(DisplayId, err.into());
                return Ready(Err(()));
            }
        };

        let id = msg.object_id();
        let op = msg.opcode();
        let obj = match client.objects.get_anon(id) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("client#{} failed to lookup object#{id}: {err}", client.id);
                client.send_error(id, err.into());
                return Ready(Err(()));
            },
        };

        let iface = obj.interface();
        match self.route(obj, msg, client) {
            Ok(_) => Ready(Ok(())),
            Err(err) => {
                log::error!("client#{} failed to handle {iface}::{op}: {err}", client.id);
                client.send_error(id, err);
                Ready(Err(()))
            }
        }
    }

    fn route(
        &mut self,
        obj: Object<Interface, Version, Handle>,
        msg: RawMessage<'_>,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        use wayland::interface::*;
        macro_rules! dispatch {
            (
                |$ob:ident|$exp:expr,
                ||$todo:expr,
                $($iface:ident,)*
            ) => {
                match obj.interface() {
                    $(InterfaceId::$iface => {
                        let $ob = obj.with_type::<$iface>();
                        $exp
                    },)*
                    _ => $todo,
                }
            };
        }
        // see dispatcher below to lookup the handler
        dispatch! {
            |ob|self.dispatch(ob, msg.with_op()?, client),
            ||self.todo_interface(obj.interface(), msg.op(), client),
            WlDisplay,
            WlRegistry,
            WlCompositor,
            // WlShm,
            WlSeat,
            WlDataSource,
            WlDataDeviceManager,
        }
    }
}

// one can use goto definition on the method calls
dispatcher! {
    wl_display {
        Sync::handle,
        GetRegistry::handle,
    }
    wl_registry {
        Bind::handle,
    }
    wl_compositor {
        CreateSurface::handle,
        CreateRegion::handle,
        Release::handle,
    }
//     wl_shm {
//         CreatePool,
//         Release,
//     }
    // ===== seat =====
    wl_seat {
        GetPointer::handle,
        GetKeyboard::handle,
        GetTouch::handle,
        Release::handle,
    }
    // ===== data =====
    wl_data_source {
        Offer::handle,
        Destroy::handle,
        SetActions::handle,
    }
    wl_data_device_manager {
        CreateDataSource::handle,
        GetDataDevice::handle,
        Release::handle,
    }
    // // ===== surface =====
    // wl_surface {
    //     Commit::handle,
    // }
    // ===== xdg shell =====
    xdg_wm_base {
        Destroy::handle,
        CreatePositioner::handle,
        GetXdgSurface::handle,
        Pong::handle,
    }
    xdg_surface {
        Destroy::handle,
        GetToplevel::handle,
        GetPopup::handle,
        SetWindowGeometry::handle,
        AckConfigure::handle,
    }
    // xdg_toplevel {
    //     SetTitle::handle,
    //     SetAppId::handle,
    //     .. todo_interface,
    // }
}

impl Compositor {
    pub fn todo_interface<T>(
        &mut self,
        iface: Interface,
        op: u16,
        client: &mut ClientMut,
    ) -> Result<T, WlError> {
        log::error!("client#{} {iface}::{op} is not yet implemented", client.id,);
        Err(WlError::NotYetImplemented)
    }

    pub fn todo<R: WlMessage>(&mut self, req: R, client: &mut ClientMut) -> WlError {
        log::error!(
            "client#{} {}::{} is not yet implemented",
            client.id,
            R::OPNAME,
            req.interface(),
        );
        WlError::NotYetImplemented
    }
}

// ===== dispatch =====

type DispatchObj<I> = Object<I, Version, Handle>;

type DispatchMsg<'a, I> = Message<Payload<'a>, <I as WlInterface>::RequestOp>;

trait Dispatcher<Interface: WlInterface>: Sized {
    fn dispatch(
        &mut self,
        obj: DispatchObj<Interface>,
        msg: DispatchMsg<'_, Interface>,
        client: &mut ClientMut,
    ) -> Result<(), WlError>;
}

macro_rules! dispatcher {
    ($(
        $imod:ident {
            $($msg:ident::$h:ident),* $(,)?
        }
    )*) => {$(
        impl Dispatcher<interface::$imod::InterfaceType> for Compositor {
            fn dispatch(
                &mut self,
                obj: DispatchObj<interface::$imod::InterfaceType>,
                msg: DispatchMsg<'_, interface::$imod::InterfaceType>,
                client: &mut ClientMut,
            ) -> Result<(), WlError> {
                use interface::$imod::{*, RequestOp as Op};
                match msg.op() {$(
                    Op::$msg => {
                        let id = msg.object_id();
                        self.$h(msg.decode_payload::<_, $msg>(client.read_fd, obj.version())?, client)?;
                        if $msg::IS_DESTRUCTOR {
                            client.delete_id(id);
                        }
                        Ok(())
                    }
                )*}
            }
        }
    )*};
}
use dispatcher;
