use wayland::primitives::FromObjectId;
use wayland::object::Handle;
use wayland::AsInterface;
use wayland::wl_compositor::CreateSurface;
use wayland::wl_display::{GetRegistry, Sync};
use wayland::wl_registry::{Bind, BindError};
use wayland::wl_seat::WlSeat;
use wayland::wl_shm::WlShm;

use crate::compositor::prelude::*;
use crate::compositor::{BindEffect, GLOBALS};
use crate::wayland::surface::Surface;

impl RequestHandler<Sync> for Compositor {
    fn handle(&mut self, sync: Operation<Sync>, client: &mut ClientMut) -> Result<(), WlError> {
        let wl_callback = sync.callback.create();
        client.objects.use_one(&wl_callback);
        client.send(wl_callback.done(0));
        client.delete_id(wl_callback);
        Ok(())
    }
}

impl RequestHandler<GetRegistry> for Compositor {
    fn handle(
        &mut self,
        req: Operation<GetRegistry>,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        let wl_registry = client.objects.create(req)?;

        for (global, i) in GLOBALS.iter().zip(0..) {
            client.send(wl_registry.global(i, global.name(), global.version().to_u32()));
        }

        Ok(())
    }
}

impl RequestHandler<Bind<'_>> for Compositor {
    fn handle(&mut self, bind: Operation<Bind<'_>>, client: &mut ClientMut) -> Result<(), WlError> {
        let Some(global) = GLOBALS.get(bind.name as usize) else {
            return Err(BindError::UnknownName.into());
        };
        if bind.new_id_name != global.name() {
            return Err(BindError::MissmatchName.into());
        }
        if bind.new_id_version > global.version() {
            return Err(BindError::UnsupportedVersion.into());
        }
        client.objects.insert_parts(
            bind.new_id,
            global.interface(),
            bind.new_id_version,
            Handle::default(),
        )?;

        // some interface has side-effect after binding
        macro_rules! bind_effect {
            ($($iface:ident),*) => {
                match global.interface() {
                    $(Interface::$iface => self.bind(
                        $iface::from_object_id(bind.new_id),
                        client
                    ),)*
                    _ => Ok(()),
                }
            };
        }
        bind_effect!(WlSeat, WlShm)
    }
}

impl RequestHandler<CreateSurface> for Compositor {
    fn handle(
        &mut self,
        req: Operation<CreateSurface>,
        client: &mut ClientMut,
    ) -> Result<(), WlError> {
        let (id, _) = self.surfaces.insert(Surface::None);
        let handle = Handle::from_idx(id);
        let _ = client.objects.create_handle(req, handle)?;
        Ok(())
    }
}
