use xdg_wm_base::{CreatePositioner, Destroy as XdgWmDestroy, GetXdgSurface, Pong};
use xdg_surface::*;
use xdg_toplevel::{Destroy as ToplevelDestroy, *};

use crate::surface::XdgSurface;
use crate::compositor::prelude::*;
use crate::compositor::traits::{CommitEffect, CommitError};

// ===== xdg_wm_base =====

impl MessageHandler<XdgWmDestroy> for Compositor {
    fn handle(&mut self, _: Msg<XdgWmDestroy>, _: &mut ClientMut) {
        // global object destroy
    }
}

impl MessageHandler<CreatePositioner> for Compositor {
    fn handle(&mut self, _: Msg<CreatePositioner>, _: &mut ClientMut) -> Todo<CreatePositioner> {
        Todo::new()
    }
}

impl MessageHandler<GetXdgSurface> for Compositor {
    fn handle(&mut self, msg: Msg<GetXdgSurface>, client: &mut ClientMut) -> Result<(), UnknownId> {
        let surface = client.objects.get_with(msg.surface)?;

        let xdg_surface_obj = Object::from_new_id(msg.new_id);
        let xdg_surface = XdgSurface::new(xdg_surface_obj, surface.handle());
        let xdg_handle = self.xdg_surfaces.create(xdg_surface);
        client.objects.create_with(msg, xdg_handle);

        Ok(())
    }
}

impl MessageHandler<Pong> for Compositor {
    fn handle(&mut self, _: Msg<Pong>, _: &mut ClientMut) -> Todo<Pong> {
        Todo::new()
    }
}

// ===== xdg_positioner =====

// ===== xdg_surface =====

type XdgSurfaceResult = Result<(), xdg_surface::Error>;

impl MessageHandler<xdg_surface::Destroy> for Compositor {
    fn handle(&mut self, msg: Msg<xdg_surface::Destroy>, _: &mut ClientMut) {
        // TODO: role object must have been destroyed
        self.xdg_surfaces.remove(msg.handle());
    }
}

impl MessageHandler<GetToplevel> for Compositor {
    fn handle(&mut self, msg: Msg<GetToplevel>, client: &mut ClientMut) -> XdgSurfaceResult {
        let xdg_surface_handle = msg.handle();
        let xdg_surface = &mut self.xdg_surfaces[xdg_surface_handle];
        let surface = &mut self.surfaces[xdg_surface.surface()];

        let xdg_toplevel = client.objects.create_with(msg, xdg_surface_handle);
        xdg_surface.set_toplevel_role(xdg_toplevel, surface)
    }
}

todo_handler!(GetPopup);
todo_handler!(SetWindowGeometry);

impl MessageHandler<AckConfigure> for Compositor {
    fn handle(&mut self, msg: Msg<AckConfigure>, _: &mut ClientMut) -> XdgSurfaceResult {
        self.xdg_surfaces[msg.handle()].ack(msg.serial)
    }
}

// ===== xdg_toplevel =====

impl CommitEffect<XdgToplevel> for Compositor {
    fn commit(
        &mut self,
        is_configured: bool,
        obj: Object<XdgToplevel>,
        client: &mut ClientMut,
    ) -> Result<(), CommitError> {
        if is_configured {
            client.send(obj.close());
        } else {
            let toplevel = client.objects.get_with(obj)?;
            let xdg_surface = &mut self.xdg_surfaces[toplevel.handle()];
            let xdg_surface_obj = xdg_surface.object();

            let serial = xdg_surface.next_ack();
            client.send(obj.configure(0, 0, &[]));
            client.send(xdg_surface_obj.configure(serial));
        }

        Ok(())
    }
}

impl MessageHandler<xdg_toplevel::Destroy> for Compositor {
    fn handle(&mut self, msg: Msg<ToplevelDestroy>, _: &mut ClientMut) {
        let xdg_surface = &mut self.xdg_surfaces[msg.handle()];
        let surface = &mut self.surfaces[xdg_surface.surface()];
        xdg_surface.remove_role(surface);
    }
}

todo_handler!(SetParent);

impl MessageHandler<SetTitle<'_>> for Compositor {
    fn handle(&mut self, msg: Msg<SetTitle<'_>>, _: &mut ClientMut) {
        if let Some(toplevel) = self.xdg_surfaces[msg.handle()].as_toplevel() {
            toplevel.title = Some(msg.title.into());
        }
    }
}

impl MessageHandler<SetAppId<'_>> for Compositor {
    fn handle(&mut self, msg: Msg<SetAppId<'_>>, _: &mut ClientMut) {
        if let Some(toplevel) = self.xdg_surfaces[msg.handle()].as_toplevel() {
            toplevel.app_id = Some(msg.app_id.into());
        }
    }
}

todo_handler!(ShowWindowMenu);
todo_handler!(Move);
todo_handler!(Resize);
todo_handler!(SetMaxSize);
todo_handler!(SetMinSize);
todo_handler!(SetMaximized);
todo_handler!(UnsetMaximized);
todo_handler!(SetFullscreen);
todo_handler!(UnsetFullscreen);
todo_handler!(SetMinimized);
