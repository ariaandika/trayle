use wayland::wl_display::{GetRegistry, Sync};
use wayland::wl_registry::{Bind, BindError};
use wayland::wl_seat::WlSeat;
use wayland::wl_shm::{PixelFormat, WlShm};
use wayland::wl_compositor::CreateSurface;
use wayland::Global;

use crate::compositor::GLOBALS;
use crate::compositor::prelude::*;

impl RequestHandler<Sync> for Compositor {
    fn handle(&mut self, sync: Sync, client: &mut ClientMut) -> Result<(), WlError> {
        let wl_callback = sync.callback.create();
        client.objects.use_one(&wl_callback);
        client.send(wl_callback.done(0));
        client.delete_id(wl_callback);
        Ok(())
    }
}

impl RequestHandler<GetRegistry> for Compositor {
    fn handle(&mut self, request: GetRegistry, client: &mut ClientMut) -> Result<(), WlError> {
        let wl_registry = client.objects.create(request.registry)?;

        for (Global { name, version, .. }, i) in GLOBALS.iter().zip(0..) {
            client.send(wl_registry.global(i, name, version.to_u32()));
        }

        Ok(())
    }
}

impl RequestHandler<Bind<'_>> for Compositor {
    fn handle(&mut self, bind: Bind<'_>, client: &mut ClientMut) -> Result<(), WlError> {
        let Some(global) = GLOBALS.get(bind.name as usize) else {
            return Err(BindError::UnknownName.into());
        };
        if bind.id_name != global.name {
            return Err(BindError::MissmatchName.into());
        }
        if bind.id_version > global.version.to_u32() {
            return Err(BindError::UnsupportedVersion.into());
        }
        client.objects.insert_parts(bind.id, global.interface, bind.id_version)?;
        client.binds.push(bind.data(global.interface));

        // some interface has side-effect after binding
        match global.interface {
            Interface::WlSeat => {
                let wl_seat = bind.create::<WlSeat>();
                client.send(wl_seat.name(self.seat.name()));
                client.send(wl_seat.capabilities(self.seat.capability()));
            }
            Interface::WlShm => {
                let wl_shm = bind.create::<WlShm>();
                client.send(wl_shm.format(PixelFormat::Argb8888));
                client.send(wl_shm.format(PixelFormat::Xrgb8888));
            }
            _ => (),
        }

        Ok(())
    }
}

impl RequestHandler<CreateSurface> for Compositor {
    fn handle(&mut self, req: CreateSurface, client: &mut ClientMut) -> Result<(), WlError> {
        let _ = client.objects.create(req.surface)?;
        Ok(())
    }
}

