use wayland::interface::AsInterface;
use wayland::object::Handle;
use wayland::error::BindError;

use wl_display::{GetRegistry, Sync};
use wl_registry::Bind;
use wl_compositor::{CreateSurface, CreateRegion, Release};

use crate::compositor::prelude::*;
use crate::compositor::{BindEffect, GLOBALS};
use crate::wayland::surface::Surface;

impl RequestHandler<Sync> for Compositor {
    fn handle(&mut self, sync: Req<Sync>, client: &mut ClientMut) -> Result<(), WlError> {
        let wl_callback = client.objects.use_one(sync.callback);
        client.send(wl_callback.done(0));
        client.delete_id(wl_callback);
        Ok(())
    }
}

impl RequestHandler<GetRegistry> for Compositor {
    fn handle(&mut self, msg: Req<GetRegistry>, client: &mut ClientMut) -> Result<(), WlError> {
        let wl_registry = client.objects.create(msg)?;
        for (global, i) in GLOBALS.iter().zip(0..) {
            client.send(wl_registry.global(i, global.name(), global.version().to_u32()));
        }
        Ok(())
    }
}

impl RequestHandler<Bind<'_>> for Compositor {
    fn handle(&mut self, bind: Req<Bind<'_>>, client: &mut ClientMut) -> Result<(), WlError> {
        let Some(global) = GLOBALS.get(bind.name as usize) else {
            return Err(BindError::UnknownName.into());
        };
        if bind.id_name != global.name() {
            return Err(BindError::MissmatchName.into());
        }
        if bind.id_version > global.version() {
            return Err(BindError::UnsupportedVersion.into());
        }

        let interface = global.interface();
        client.objects.insert_parts(
            bind.new_id,
            interface,
            bind.id_version,
            Handle::default(),
        )?;

        // some interface has side-effect after binding
        macro_rules! dispatch {
            ($($iface:ident::$bind:ident),*) => {
                match interface {
                    $(Interface::$iface => self.$bind(
                        // because `interface` has just been "matched", hopefully the panic check is
                        // trimmed by the compiler
                        Object::<$iface>::from_dynamic(bind.new_id, interface),
                        client
                    ),)*
                    _ => Ok(()),
                }
            };
        }
        dispatch!(WlSeat::bind, WlShm::bind)
    }
}

impl RequestHandler<CreateSurface> for Compositor {
    fn handle(&mut self, req: Req<CreateSurface>, client: &mut ClientMut) -> Result<(), WlError> {
        let (id, _) = self.surfaces.insert(Surface::None);
        let handle = Handle::from_idx(id);
        let _ = client.objects.create_handle(req, handle)?;
        Ok(())
    }
}

impl RequestHandler<CreateRegion> for Compositor {
    fn handle(&mut self, req: Req<CreateRegion>, client: &mut ClientMut) -> Result<(), WlError> {
        Err(self.todo(req, client))
    }
}

impl RequestHandler<Release> for Compositor {
    fn handle(&mut self, req: Req<Release>, client: &mut ClientMut) -> Result<(), WlError> {
        Err(self.todo(req, client))
    }
}
