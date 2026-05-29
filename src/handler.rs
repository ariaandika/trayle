use crate::{State, log};

use crate::wayland::{Decode, Encode, Frame, FromOp, InterfaceId, WlError};

use crate::wayland::wl_display as WlDisplay;
use crate::wayland::wl_registry as WlRegistry;
use crate::wayland::wl_compositor as WlCompositor;
use crate::wayland::wl_shm as WlShm;
use crate::wayland::wl_seat as WlSeat;
use crate::wayland::wl_data_device_manager as WlDataDeviceManager;

static GLOBALS: [(&str, u16, InterfaceId); 9] = [
    ("wl_compositor", 7, InterfaceId::WlCompositor),
    ("wl_shm", 2, InterfaceId::WlShm),
    ("wl_data_device_manager", 4, InterfaceId::WlDataDeviceManager),
    ("wl_seat", 10, InterfaceId::WlSeat),
    ("wl_subcompositor", 1, InterfaceId::WlSubCompositor),
    ("wl_fixes", 2, InterfaceId::WlFixes),
    ("zwp_linux_dmabuf_v1", 5, InterfaceId::ZwpLinuxDmabufV1),
    ("zwp_linux_dmabuf_feedback_v1", 5, InterfaceId::ZwpLinuxDmabufFeedbackV1),
    ("xdg_wm_base", 7, InterfaceId::XdgWmBase),
];

pub fn router(frame: Frame, state: State) -> Result<(), WlError> {
    let (id, op) = frame.parts();
    let interface = if id.is_display() {
        InterfaceId::WlDisplay
    } else {
        match state.client.objects_mut().get_mut(id) {
            Some(object) => object.interface(),
            None => return Err(WlError::UnknownObject),
        }
    };

    macro_rules! handle_me {
        (@OP $iface:ident { $($req:ident $($flag:ident)?),* $(,)? }) => {
            match <_>::from_op(op)? { $(
                $iface::RequestOp::$req => handle_me!(@CALL $iface $req $($flag)?),
            )* }
        };
        (@CALL $iface:ident $req:ident todo) => {{
            state.todo(interface, op)
        }};
        (@CALL $iface:ident $req:ident) => {
            state.handle_trace(interface, $iface::$req::decode_with(frame)?)
        };
        ($($iface:ident {$($tt:tt)*})*) => {
            match interface {
                $(
                    InterfaceId::$iface => handle_me!(@OP $iface {$($tt)*}),
                )*
                _ => state.todo(interface, op),
            }
        };
    }

    handle_me! {
        WlDisplay { Sync, GetRegistry }
        WlRegistry { Bind }
        WlCompositor {
            CreateSurface
        }
        WlShm {
            CreatePool todo,
            Release todo,
        }
        WlSeat {
            GetPointer todo,
            GetKeyboard,
        }
        WlDataDeviceManager {
            CreateDataSource todo,
            GetDataDevice,
            Release todo,
        }
    }
}

// ===== handlers =====

trait RequestHandler<Request>: Sized {
    fn handle(self, request: Request) -> Result<(), WlError>;

    fn handle_trace(self, _interface: InterfaceId, request: Request) -> Result<(), WlError>
    where
        Request: std::fmt::Debug,
    {
        #[cfg(debug_assertions)]
        log::trace!(client, "<- {_interface:?}::{request:?}");
        self.handle(request)
    }
}

impl State<'_> {
    fn send<E: Encode>(&mut self, message: E) {
        message.encode_to(self.write_buffer);
    }

    fn todo(self, interface: InterfaceId, op: u16) -> Result<(), WlError> {
        log::error!(client, "<- `{interface:?}::{op}` is not yet implemented");
        WlError::todo()
    }
}

impl RequestHandler<WlDisplay::Sync> for State<'_> {
    fn handle(mut self, sync: WlDisplay::Sync) -> Result<(), WlError> {
        let callback = sync.callback.get();
        self.client.objects_mut().use_one(&callback)?;
        self.send(callback.done(69));
        self.send(WlDisplay::delete_id(&callback));
        Ok(())
    }
}

impl RequestHandler<WlDisplay::GetRegistry> for State<'_> {
    fn handle(mut self, request: WlDisplay::GetRegistry) -> Result<(), WlError> {
        let registry = request.registry.get();
        self.client.objects_mut().insert_object(&registry)?;

        // FEAT: encode globals at startup
        for ((iface, version, _), i) in GLOBALS.into_iter().zip(0..) {
            self.send(registry.global(i, iface, version as u32));
        }

        Ok(())
    }
}

impl<'a> RequestHandler<WlRegistry::Bind<'a>> for State<'a> {
    fn handle(mut self, bind: WlRegistry::Bind<'a>) -> Result<(), WlError> {
        let Some((bind_name, version, iface)) = GLOBALS.get(bind.name as usize) else {
            return Err(WlError::UnknownBind);
        };
        if bind.id_name != *bind_name {
            return Err(WlError::UnknownBind);
        }
        if bind.id_version > *version as u32 {
            return Err(WlError::UnknownBind);
        }
        self.client.objects_mut().insert(bind.id, *iface)?;

        // some interface has side-effect after binding
        if let InterfaceId::WlSeat = iface {
            let seat = bind.get::<WlSeat::Seat>();
            self.send(seat.capabilities(self.seat.capability()));
        }

        Ok(())
    }
}

impl RequestHandler<WlCompositor::CreateSurface> for State<'_> {
    fn handle(self, req: WlCompositor::CreateSurface) -> Result<(), WlError> {
        let surface = req.surface.get();
        self.client.objects_mut().insert_object(&surface)
    }
}

impl RequestHandler<WlSeat::GetKeyboard> for State<'_> {
    fn handle(mut self, req: WlSeat::GetKeyboard) -> Result<(), WlError> {
        let keyboard = req.keyboard.get();
        self.client.objects_mut().insert_object(&keyboard)?;
        self.send(keyboard.keymap_xkb_v1(self.seat));
        Ok(())
    }
}

impl RequestHandler<WlDataDeviceManager::GetDataDevice> for State<'_> {
    fn handle(self, req: WlDataDeviceManager::GetDataDevice) -> Result<(), WlError> {
        let _ = req.seat;
        self.client
            .objects_mut()
            .insert(req.id, crate::wayland::InterfaceId::WlDataDevice)?;
        Ok(())
    }
}
