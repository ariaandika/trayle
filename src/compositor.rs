use std::task::Poll::{self, *};

use todex::log;
use todex::sys::bytes::Bytes;
use todex::wayland::primitives::{AsObjectId, AsVersion};
use todex::wayland::object::{Global, global_of, ObjectEntry};
use todex::wayland::message::{Message, WlMessage};
use todex::wayland::interface::{self, AsInterface, DisplayId, Interface};
use todex::wayland::wire::{AsOpCode, OpCode, Payload};
use todex::wayland::error::WlError;

use crate::error::FatalError;
use crate::seat::Seat;
use crate::client::ClientMut;
use crate::wayland::{Buffers, ShmPools, Surfaces, XdgSurfaces};

use traits::MessageHandler;

mod prelude {
    pub(super) use todex::wayland;
    pub(super) use todex::wayland::primitives::AsVersion;
    pub(super) use todex::wayland::object::Object;
    pub(super) use todex::wayland::message::WlMessage;
    pub(super) use todex::wayland::interface::*;
    pub(super) use todex::wayland::error::WlError;

    pub(super) use crate::client::ClientMut;

    pub(super) use super::Compositor;
    pub(super) use super::traits::{MessageHandler, Msg, todo_handler};
}

mod traits;

mod wl_display;
mod wl_compositor;
mod wl_shm;
mod wl_seat;
mod wl_data_source;
mod wl_data_device_manager;
mod xdg_shell;

// ===== globals =====

static GLOBALS: [Global; 5] = {
    use interface::*;
    [
        global_of::<WlCompositor>(),
        global_of::<WlShm>(),
        global_of::<WlDataDeviceManager>(),
        global_of::<WlSeat>(),
        global_of::<XdgWmBase>(),
    ]
};

// ===== Compositor =====

pub struct Compositor {
    seat: Seat,
    buffers: Buffers,
    shm_pools: ShmPools,
    surfaces: Surfaces,
    xdg_surfaces: XdgSurfaces,
}

impl Compositor {
    pub fn new() -> Result<Self, FatalError> {
        Ok(Self {
            seat: Seat::new()?,
            buffers: Buffers::new(),
            shm_pools: ShmPools::new(),
            surfaces: Surfaces::new(),
            xdg_surfaces: XdgSurfaces::new(),
        })
    }

    pub fn message(
        &mut self,
        read_buf: &mut Bytes,
        client: &mut ClientMut,
    ) -> Poll<Result<(), ()>> {
        let Ready(result) = Message::get_message(read_buf) else {
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
                client.send_error(DisplayId, err.into());
                return Ready(Err(()));
            }
        };

        let iface = obj.interface();
        match self.route(obj, msg, client) {
            Ok(_) => Ready(Ok(())),
            Err(err) => {
                if !matches!(err, WlError::NotYetImplemented) {
                    log::error!("client#{} failed to handle {iface}::{op}: {err}", client.id);
                    client.send_error(id, err);
                }
                Ready(Err(()))
            }
        }
    }

    fn todo<T, M: WlMessage>(
        &mut self,
        msg: prelude::Msg<M>,
        client: &mut ClientMut,
    ) -> Result<T, WlError> {
        let iface = msg.interface();
        let op = M::OPNAME;
        log::error!("client#{} {iface}::{op} is not yet implemented", client.id);
        client.send_error(DisplayId, WlError::NotYetImplemented);
        Err(WlError::NotYetImplemented)
    }

    fn todo_interface<T>(
        &mut self,
        iface: Interface,
        op: u16,
        client: &mut ClientMut,
    ) -> Result<T, WlError> {
        log::error!("client#{} {iface}::{op} is not yet implemented", client.id);
        client.send_error(DisplayId, WlError::NotYetImplemented);
        Err(WlError::NotYetImplemented)
    }
}

// one can use goto definition on the method calls
dispatcher! {
    WlDisplay {
        Sync::handle,
        GetRegistry::handle,
    }
    WlRegistry {
        Bind::handle,
    }
    WlCompositor {
        CreateSurface::handle,
        CreateRegion::handle,
        Release::handle,
    }
    WlShmPool {
        CreateBuffer::handle,
        Destroy::handle,
        Resize::handle,
    }
    WlShm {
        CreatePool::handle,
        Release::handle,
    }
    WlBuffer {
        Destroy::handle,
    }
    WlDataSource {
        Offer::handle,
        Destroy::handle,
        SetActions::handle,
    }
    WlDataDeviceManager {
        CreateDataSource::handle,
        GetDataDevice::handle,
        Release::handle,
    }
    WlSurface {
        Destroy::handle,
        Attach::handle,
        Damage::handle,
        Frame::handle,
        SetOpaqueRegion::handle,
        SetInputRegion::handle,
        Commit::handle,
        SetBufferTransform::handle,
        SetBufferScale::handle,
        DamageBuffer::handle,
        Offset::handle,
        GetRelease::handle,
    }
    WlSeat {
        GetPointer::handle,
        GetKeyboard::handle,
        GetTouch::handle,
        Release::handle,
    }
    XdgWmBase {
        Destroy::handle,
        CreatePositioner::handle,
        GetXdgSurface::handle,
        Pong::handle,
    }
    XdgSurface {
        Destroy::handle,
        GetToplevel::handle,
        GetPopup::handle,
        SetWindowGeometry::handle,
        AckConfigure::handle,
    }
    XdgToplevel {
        Destroy::handle,
        SetParent::handle,
        SetTitle::handle,
        SetAppId::handle,
        ShowWindowMenu::handle,
        Move::handle,
        Resize::handle,
        SetMaxSize::handle,
        SetMinSize::handle,
        SetMaximized::handle,
        UnsetMaximized::handle,
        SetFullscreen::handle,
        UnsetFullscreen::handle,
        SetMinimized::handle,
    }
}

// ===== dispatch =====

macro_rules! dispatcher {
    ($(
        $iface:ident {
            $($msg:ident::$h:ident),* $(,)?
        }
    )*) => {
        impl Compositor {
            fn route(
                &mut self,
                obj: ObjectEntry,
                msg: Message<Payload<'_>, u16>,
                client: &mut ClientMut,
            ) -> Result<(), WlError> {
                use interface::*;
                match obj.interface() {
                    $(Interface::$iface => {
                        use interface::camel_cased::$iface::{*, RequestOp};
                        match <_>::try_from_op(msg.opcode())? {
                            $(RequestOp::$msg => {
                                log::debug!(
                                    "client#{} <- {}::{}(..)",
                                    client.id,
                                    Interface::$iface,
                                    $msg::OPNAME,
                                );
                                let id = msg.object_id();
                                let payload = msg.decode_payload::<_, $msg>(client.read_fd)?;
                                let msg = Message::from_parts(obj.handle(), payload, obj.version());
                                self.$h(msg, client)?;
                                if $msg::IS_DESTRUCTOR {
                                    client.delete_id(id);
                                }
                                Ok(())
                            })*
                        }
                    })*
                    _ => self.todo_interface(obj.interface(), msg.meta(), client),
                }
            }
        }
    };
}
use dispatcher;
