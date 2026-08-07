//! The Compositor.
//!
//! This is the mediator that route incoming messages into its respective handler.
//!
//! The entry point is [`Compositor::message`].
use todex::log;
use todex::bytes::Bytes;
use todex::wayland::primitives::{AsObjectId, AsVersion};
use todex::wayland::object::{Global, global_of};
use todex::wayland::display::AsDisplay;
use todex::wayland::message::{Message, OpCode, WlMessage};
use todex::wayland::interface::{self, AsInterface, InterfaceId};
use todex::wayland::interface::wl_display::DisplayId;
use todex::wayland::wire::{DecodePayload, Payload};

use crate::client::{ClientMut, ObjectEntry};
use crate::handle::{AsHandle, Handle, WithHandle};
use crate::seat::Seat;
use crate::shm::{Buffer, Buffers, ShmPool, ShmPools};
use crate::surface::{Region, Regions, Surface, Surfaces, XdgSurface, XdgSurfaces};

use error::{HandleResult, MessageError, Todo};

mod prelude {
    pub(super) use todex::wayland::primitives::AsVersion;
    pub(super) use todex::wayland::object::{Object, UnknownId};
    pub(super) use todex::wayland::message::WlMessage;
    pub(super) use todex::wayland::interface::*;

    pub(super) use crate::compositor::Resources;
    pub(super) use crate::compositor::traits::Msg;
    pub(super) use crate::handle::AsHandle;
    pub(super) use crate::client::ClientMut;
}

mod handle;
mod error;
mod traits;

mod wl_display;
mod wl_compositor;
mod wl_shm;
mod wl_seat;
mod wl_data;
mod xdg_shell;

// ===== globals =====

static GLOBALS: [Global; 6] = {
    use interface::*;
    [
        global_of::<WlCompositor>(),
        global_of::<WlShm>(),
        global_of::<WlDataDeviceManager>(),
        global_of::<WlSeat>(),
        global_of::<XdgWmBase>(),
        global_of::<ZwpLinuxDmabufV1>(),
    ]
};

// ===== Resources =====

pub(crate) struct Resources {
    pub buffers: Buffers,
    pub shm_pools: ShmPools,
    pub regions: Regions,
    pub surfaces: Surfaces,
    pub xdg_surfaces: XdgSurfaces,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            buffers: Buffers::new(),
            shm_pools: ShmPools::new(),
            regions: Regions::new(),
            surfaces: Surfaces::new(),
            xdg_surfaces: XdgSurfaces::new(),
        }
    }
}
trait WithResource<R> {
    fn get_mut(&mut self, handle: Handle<R>) -> &mut R;

    fn get_with<H: AsHandle<R>>(&mut self, msg: &H) -> &mut R {
        self.get_mut(msg.handle())
    }
}
macro_rules! impl_res {
    ($($ts:ty,$ty:ty,$fd:ident;)*) => {$(
        impl AsMut<$ts> for Resources {
            fn as_mut(&mut self) -> &mut $ts {
                &mut self.$fd
            }
        }
        impl WithResource<$ty> for Resources {
            fn get_mut(&mut self, handle: Handle<$ty>) -> &mut $ty {
                &mut self.$fd[handle]
            }
        }
    )*};
}
impl_res! {
    Buffers,Buffer,buffers;
    ShmPools,ShmPool,shm_pools;
    Surfaces,Surface,surfaces;
    Regions,Region,regions;
    XdgSurfaces,XdgSurface,xdg_surfaces;
}

// ===== entrypoint =====

pub fn route(read_buf: &mut Bytes, client: &mut ClientMut, seat: &mut Seat, res: &mut Resources) {
    // cope and seeth: https://github.com/rust-lang/rust/issues/31436
    let result = (|| loop {
        let Some(msg) = Message::get_message(read_buf)? else {
            return Ok::<_, MessageError>(());
        };
        let obj = client.objects.get(msg.object_id())?;
        route_me(obj, msg, client, res, seat)?;
    })();

    if let Err(err) = result {
        if matches!(err, MessageError::Disconnect) {
            read_buf.clear();
            client.disconnect();
        } else {
            log::error!("client#{} failed to handle message: {err}", client.id);
            client.send_error(DisplayId, err);
        }
    }
}

fn decode<'a, const N: usize, OP, M>(
    obj: ObjectEntry,
    msg: Message<Payload<'a>, OP>,
    client: &mut ClientMut,
) -> Result<traits::Msg<M>, MessageError>
where
    M: WlMessage<WlInterface: WithHandle> + DecodePayload<'a, N>,
{
    let payload = msg.decode_payload::<_, M>(client.read_fd)?;
    let msg = Message::from_parts(obj.handle().cast(), payload, obj.version());
    log::debug!("client#{} <- {}", client.id, msg.display());
    if let Some(new_id) = msg.get_new_id() {
        client.objects.checks_id(new_id)?;
    }
    Ok(msg)
}

// ===== router =====

routes! {
    fn route_me(msg, client, res, seat);

    WlDisplay {
        Sync => wl_display::sync(msg, client),
        GetRegistry => wl_display::get_registry(msg, client),
    }
    WlRegistry {
        Bind => wl_display::bind(msg, client, seat),
    }

    // ===== wl_compositor =====

    WlCompositor {
        CreateSurface => let _ = client.create_with(msg, res.surfaces.create()),
        CreateRegion => let _ = client.create_with(msg, res.regions.create()),
        Release => ()
    }
    WlSurface {
        Attach => wl_compositor::attach(res.get_with(&msg), msg, client),
        Commit => wl_compositor::commit(msg, client, res),
        Destroy => res.surfaces.remove(msg.handle()).map(Surface::destroy),

        Frame => wl_compositor::frame(res.get_with(&msg), msg, client),
        GetRelease => wl_compositor::get_release(res.get_with(&msg), msg, client),

        // TODO: differentiate surface coordinate and buffer coordinate
        //
        // Note! New clients should not use this request. Instead damage can be posted with
        // `wl_surface::damage_buffer` which uses buffer coordinates instead of surface coordinates.
        Damage => res.get_with(&msg).pending_mut().damage.union(into_region!(msg.payload())),
        DamageBuffer => res.get_with(&msg).pending_mut().damage.union(into_region!(msg.payload())),
        Offset => wl_compositor::offset(msg, res.as_mut()),

        SetOpaqueRegion => res.get_with(&msg).pending_mut().opaque = msg.region,
        SetInputRegion => res.get_with(&msg).pending_mut().input = msg.region,
        SetBufferTransform => res.get_with(&msg).pending_mut().transform = msg.transform,
        SetBufferScale => res.get_with(&msg).pending_mut().scale = msg.scale,
    }
    WlRegion {
        Destroy => let _ = res.regions.remove(msg.handle()),
        Add => res.get_with(&msg).add(into_region!(msg.payload())),
        Subtract => res.regions[msg.handle()].subtract(into_region!(msg.payload())),
    }

    // ===== wl_shm =====

    WlShm {
        CreatePool => wl_shm::create_pool(msg, client, res.as_mut()),
        Release => (),
    }
    WlShmPool {
        CreateBuffer => wl_shm::create_buffer(msg, client, res),
        Destroy => res.shm_pools.destroy(msg.handle()),
        Resize => res.get_with(&msg).resize(msg.size),
    }
    WlBuffer {
        Destroy => res.shm_pools.destroy_buffer(res.buffers.remove(msg.handle())),
    }

    // ===== wl_seat =====

    WlSeat {
        GetPointer => wl_seat::get_pointer(msg, client, seat),
        GetKeyboard => wl_seat::get_keyboard(msg, client, seat),
        GetTouch => wl_seat::get_touch(msg, client, seat),
        Release => (),
    }

    // ===== wl_data =====

    WlDataSource {
        Offer => Todo::from(msg),
        Destroy => Todo::from(msg),
        SetActions => Todo::from(msg),
    }
    WlDataDeviceManager {
        CreateDataSource => wl_data::create_source(msg, client),
        GetDataDevice => wl_data::get_data_device(msg, client, seat),
        Release => seat.clear_data_device(),
    }

    // ===== xdg_shell =====

    XdgWmBase {
        Destroy => (),
        CreatePositioner => Todo::from(msg),
        GetXdgSurface => xdg_shell::get_xdg_surface(msg, client, res.as_mut()),
        Pong => Todo::from(msg),
    }
    XdgSurface {
        // TODO: role object must have been destroyed
        Destroy => let _ = res.xdg_surfaces.remove(msg.handle()),
        GetToplevel => xdg_shell::get_toplevel(msg, client, res),
        GetPopup => Todo::from(msg),
        SetWindowGeometry => Todo::from(msg),
        AckConfigure => res.get_with(&msg).ack(msg.serial),
    }
    XdgToplevel {
        Destroy => xdg_shell::toplevel_destroy(msg, res),
        SetParent => Todo::from(msg),
        SetTitle => if let Some(toplevel) = res.get_with(&msg).as_toplevel() {
            toplevel.title = Some(msg.title.into());
        },
        SetAppId => if let Some(toplevel) = res.get_with(&msg).as_toplevel() {
            toplevel.app_id = Some(msg.app_id.into());
        },
        ShowWindowMenu => Todo::from(msg),
        Move => Todo::from(msg),
        Resize => Todo::from(msg),
        SetMaxSize => Todo::from(msg),
        SetMinSize => Todo::from(msg),
        SetMaximized => Todo::from(msg),
        UnsetMaximized => Todo::from(msg),
        SetFullscreen => Todo::from(msg),
        UnsetFullscreen => Todo::from(msg),
        SetMinimized => Todo::from(msg),
    }

    // ===== linux-dmabuf =====

    ZwpLinuxDmabufV1 {
        Destroy => Todo::from(msg),
        CreateParams => Todo::from(msg),
        GetDefaultFeedback => Todo::from(msg),
        GetSurfaceFeedback => Todo::from(msg),
    }
    ZwpLinuxBufferParamsV1 {
        Destroy => Todo::from(msg),
        Add => Todo::from(msg),
        Create => Todo::from(msg),
        CreateImmed => Todo::from(msg),
    }
    ZwpLinuxDmabufFeedbackV1 {
        Destroy => Todo::from(msg),
    }
}
macro_rules! into_region {
    ($expr:expr) => {{
        let b = $expr;
        crate::surface::Region {
            x: b.x,
            y: b.y,
            width: b.width,
            height: b.height,
        }
    }};
}
macro_rules! routes {
    (
        fn $fn_name:ident($msg_var:ident, $client:ident, $res:ident, $seat:ident);

        $($iface:ident {
            $($msg:ident => $handler:stmt),*
            $(,)?
        })*
    ) => {
        fn $fn_name(
            obj: ObjectEntry,
            msg: Message<Payload<'_>, u16>,
            $client: &mut ClientMut,
            $res: &mut Resources,
            $seat: &mut Seat,
        ) -> Result<(), MessageError> {
            match obj.interface() {
                $(InterfaceId::$iface => {
                    use interface::camel_cased::$iface::*;
                    match <_>::try_from_op(msg.opcode())? {
                        $(RequestOp::$msg => {
                            let id = msg.object_id();
                            // decode the message
                            #[allow(unused_variables)]
                            let $msg_var = decode::<_, _, $msg>(obj, msg, $client)?;
                            // call the handler
                            { $handler }.handle_result(id, $client);
                            // remove destructed object
                            if $msg::IS_DESTRUCTOR {
                                $client.delete_id(id);
                            }
                            Ok(())
                        })*
                    }
                })*
                iface => {
                    log::error!(
                        "client#{} {iface}::{} not yet implemented",
                        $client.id,
                        msg.opcode(),
                    );
                    $client.disconnect();
                    Ok(())
                }
            }
        }
    };
}
use {into_region, routes};
