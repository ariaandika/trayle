use crate::compositor::clients::ClientMut;
use crate::{Compositor, log};

use crate::wayland::{Decode, Frame, InterfaceId, OpCode, WlError};

use crate::wayland::wl_display as WlDisplay;
use crate::wayland::wl_registry as WlRegistry;
use crate::wayland::wl_compositor as WlCompositor;
use crate::wayland::wl_shm as WlShm;
use crate::wayland::wl_seat as WlSeat;
use crate::wayland::wl_data_device_manager as WlDataDeviceManager;

static GLOBALS: [(&str, u32, InterfaceId); 9] = [
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


pub fn router(frame: Frame, client: &mut ClientMut, compositor: &mut Compositor) -> Result<(), ()> {
    match router_inner(frame, client, compositor) {
        Ok(()) => Ok(()),
        Err(error) => {
            client.send_global_error(error);
            Err(())
        },
    }
}

fn router_inner(frame: Frame, client: &mut ClientMut, compositor: &mut Compositor) -> Result<(), WlError> {
    let (id, op) = frame.parts();
    let interface = if id.is_display() {
        InterfaceId::WlDisplay
    } else {
        match client.get_object(id) {
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
            compositor.todo(interface, op)
        }};
        (@CALL $iface:ident $req:ident) => {
            compositor.call_handler(interface, $iface::$req::decode_with(frame)?, client)
        };
        ($($iface:ident {$($tt:tt)*})*) => {
            match interface {
                $(
                    InterfaceId::$iface => handle_me!(@OP $iface {$($tt)*}),
                )*
                _ => compositor.todo(interface, op),
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

impl Compositor {
    fn call_handler<Request>(
        &mut self,
        _interface: InterfaceId,
        request: Request,
        client: &mut ClientMut,
    ) -> Result<(), WlError>
    where
        Self: RequestHandler<Request>,
        Request: std::fmt::Debug,
    {
        #[cfg(debug_assertions)]
        log::trace!(client, "<- {_interface:?}::{request:?}");
        self.handle(request, client)
    }

    fn todo(&mut self, interface: InterfaceId, op: u16) -> Result<(), WlError> {
        log::error!(client, "<- `{interface:?}::{op}` is not yet implemented");
        WlError::todo()
    }
}

trait RequestHandler<Request>: Sized {
    fn handle(&mut self, request: Request, client: &mut ClientMut) -> Result<(), WlError>;
}

mod wl_display {
    use super::*;
    use WlDisplay::{GetRegistry, Sync};

    impl RequestHandler<Sync> for Compositor {
        fn handle(&mut self, sync: Sync, client: &mut ClientMut) -> Result<(), WlError> {
            let callback = sync.callback.get();
            client.objects_mut().use_one(&callback)?;
            client.send(callback.done(69));
            client.send(WlDisplay::delete_id(&callback));
            Ok(())
        }
    }

    impl RequestHandler<GetRegistry> for Compositor {
        fn handle(&mut self, request: GetRegistry, client: &mut ClientMut) -> Result<(), WlError> {
            let registry = request.registry.get();
            client.insert_object(&registry)?;

            // FEAT: encode globals at startup
            for ((iface, version, _), i) in GLOBALS.iter().zip(0..) {
                client.send(registry.global(i, iface, *version));
            }

            Ok(())
        }
    }
}

mod wl_registry {
    use super::*;
    use WlRegistry::Bind;

    impl RequestHandler<Bind<'_>> for Compositor {
        fn handle(&mut self, bind: Bind<'_>, client: &mut ClientMut) -> Result<(), WlError> {
            let Some((bind_name, version, iface)) = GLOBALS.get(bind.name as usize) else {
                return Err(WlError::UnknownBind);
            };
            if bind.id_name != *bind_name {
                return Err(WlError::UnknownBind);
            }
            if bind.id_version > *version {
                return Err(WlError::UnknownBind);
            }
            client.objects_mut().insert(bind.id, *iface)?;

            // some interface has side-effect after binding
            if let InterfaceId::WlSeat = iface {
                let seat = bind.get::<WlSeat::Seat>();
                client.send(seat.capabilities(self.seat.capability()));
            }

            Ok(())
        }
    }
}

mod wl_compositor {
    use super::*;
    use WlCompositor::CreateSurface;

    impl RequestHandler<CreateSurface> for Compositor {
        fn handle(&mut self, req: CreateSurface, client: &mut ClientMut) -> Result<(), WlError> {
            let surface = req.surface.get();
            client.insert_object(&surface)
        }
    }
}

mod wl_seat {
    use super::*;
    use WlSeat::GetKeyboard;

    impl RequestHandler<GetKeyboard> for Compositor {
        fn handle(&mut self, req: GetKeyboard, client: &mut ClientMut) -> Result<(), WlError> {
            let keyboard = req.keyboard.get();
            client.insert_object(&keyboard)?;
            client.send(self.seat.to_keymap_event(&keyboard));
            Ok(())
        }
    }
}

mod wl_data_device_manager {
    use crate::wayland::InterfaceId;
    use super::*;
    use WlDataDeviceManager::GetDataDevice;

    impl RequestHandler<GetDataDevice> for Compositor {
        fn handle(&mut self, req: GetDataDevice, client: &mut ClientMut) -> Result<(), WlError> {
            let _ = req.seat;
            client
                .objects_mut()
                .insert(req.id, InterfaceId::WlDataDevice)?;
            Ok(())
        }
    }
}
