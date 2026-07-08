//! The Compositor.
//!
//! This is the mediator that route incoming messages into its respective handler.
//!
//! The entry point is [`Compositor::message`].
use std::time::Instant;

use todex::log;
use todex::sys::bytes::Bytes;
use todex::wayland::primitives::{AsObjectId, AsVersion};
use todex::wayland::object::{Global, global_of};
use todex::wayland::display::AsDisplay;
use todex::wayland::message::{Message, OpCode, WlMessage};
use todex::wayland::interface::{self, AsInterface, InterfaceId};
use todex::wayland::interface::wl_display::DisplayId;
use todex::wayland::wire::{DecodePayload, Payload};
use todex::wayland::error::WlError;

use crate::handle::{AsHandle, WithHandle};
use crate::seat::Seat;
use crate::client::{ClientMut, ObjectEntry};
use crate::shm::{Buffers, ShmPools};
use crate::surface::{Surfaces, XdgSurfaces};
use crate::error::FatalError;

use traits::MessageHandler;
use error::HandleResult;

mod prelude {
    pub(super) use todex::wayland;
    pub(super) use todex::wayland::primitives::AsVersion;
    pub(super) use todex::wayland::object::Object;
    pub(super) use todex::wayland::message::WlMessage;
    pub(super) use todex::wayland::interface::*;
    pub(super) use todex::wayland::error::WlError;

    pub(super) use crate::handle::AsHandle;
    pub(super) use crate::client::ClientMut;

    pub(super) use super::Compositor;
    pub(super) use super::traits::{MessageHandler, Msg, todo_handler};
}

mod handle;
mod error;
mod traits;

mod wl_display;
mod wl_compositor;
mod wl_shm;
mod wl_seat;
mod wl_data_source;
mod wl_data_device_manager;
mod xdg_shell;

// ===== ClientStatus =====

pub enum ClientStatus {
    /// Client ok.
    Ok,
    /// Client wants to disconnect.
    Disconnect,
}

impl ClientStatus {
    pub fn is_disconnect(&self) -> bool {
        matches!(self, Self::Disconnect)
    }
}

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
    start: Instant,
    seat: Seat,
    buffers: Buffers,
    shm_pools: ShmPools,
    surfaces: Surfaces,
    xdg_surfaces: XdgSurfaces,
}

impl Compositor {
    pub fn new() -> Result<Self, FatalError> {
        Ok(Self {
            start: Instant::now(),
            seat: Seat::new()?,
            buffers: Buffers::new(),
            shm_pools: ShmPools::new(),
            surfaces: Surfaces::new(),
            xdg_surfaces: XdgSurfaces::new(),
        })
    }

    /// Checks and drains available messages for given client.
    pub fn message(&mut self, read_buf: &mut Bytes, client: &mut ClientMut) -> ClientStatus {
        use ClientStatus as S;
        // cope and seeth: https://github.com/rust-lang/rust/issues/31436
        let result = (|| {
            loop {
                let Some(msg) = Message::get_message(read_buf)? else {
                    return Ok(S::Ok);
                };
                let id = msg.object_id();
                let obj = client.objects.get_anon(id)?;
                if self.route(obj, msg, client)?.is_disconnect() {
                    return Ok(S::Disconnect)
                }
            }
        })();
        match result {
            Ok(status) => status,
            Err(err) => {
                log::error!("client#{} failed to decode message: {err}", client.id);
                client.send_error(DisplayId, err);
                S::Disconnect
            }
        }
    }

    fn handle_proxy<'a, const N: usize, M>(
        &mut self,
        obj: ObjectEntry,
        msg: Message<Payload<'a>, u16>,
        client: &mut ClientMut,
    ) -> Result<ClientStatus, WlError>
    where
        M: WlMessage + DecodePayload<Fd = [i32; N]>,
        M::Output<'a>: WlMessage,
        <M::Output<'a> as WlMessage>::WlInterface: WithHandle,
        Self: MessageHandler<M::Output<'a>>,
    {
        let id = msg.object_id();
        let payload = msg.decode_payload::<_, M>(client.read_fd)?;
        let msg = Message::from_parts(obj.handle().cast(), payload, obj.version());
        log::debug!("client#{} <- {}", client.id, msg.display(),);
        let status = self.handle(msg, client).handle_result(id, client);
        if M::IS_DESTRUCTOR {
            client.delete_id(id);
        }
        Ok(status)
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
            ) -> Result<ClientStatus, WlError> {
                match obj.interface() {
                    $(InterfaceId::$iface => {
                        use interface::camel_cased::$iface::*;
                        match <_>::try_from_op(msg.opcode())? {
                            $(RequestOp::$msg => {
                                #[allow(path_statements)]
                                <Self as MessageHandler<$msg>>::$h;
                                self.handle_proxy::<_, $msg>(obj, msg, client)
                            })*
                        }
                    })*
                    _ => Err(WlError::NotYetImplemented),
                }
            }
        }
    };
}
use dispatcher;
