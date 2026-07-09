use xdg_wm_base::{CreatePositioner, Destroy as XdgWmDestroy, GetXdgSurface, Pong};
use xdg_surface::Destroy as XdgSurfaceDestroy;
use xdg_surface::{AckConfigure, GetPopup, GetToplevel, SetWindowGeometry};
use xdg_toplevel::{*, Destroy as ToplevelDestroy};

use crate::compositor::prelude::*;
use crate::compositor::traits::{CommitEffect, CommitError};

// ===== xdg_wm_base =====

impl MessageHandler<XdgWmDestroy> for Compositor {
    fn handle(&mut self, _: Msg<XdgWmDestroy>, _: &mut ClientMut) {
        // global object destroy
    }
}

todo_handler!(CreatePositioner);

impl MessageHandler<GetXdgSurface> for Compositor {
    fn handle(&mut self, msg: Msg<GetXdgSurface>, client: &mut ClientMut) -> Result<(), UnknownId> {
        let surface_handle = client.objects.get_with(msg.surface)?.handle();
        let xdg_handle = self.xdg_surfaces.create(surface_handle);
        client.objects.create_with(msg, xdg_handle);
        Ok(())
    }
}

impl MessageHandler<Pong> for Compositor {
    fn handle(&mut self, _: Msg<Pong>, _: &mut ClientMut) -> Todo<Pong> {
        // TODO: ping pong mechanism
        Todo::new()
    }
}

// ===== xdg_positioner =====

// ===== xdg_surface =====

impl MessageHandler<XdgSurfaceDestroy> for Compositor {
    fn handle(&mut self, msg: Msg<XdgSurfaceDestroy>, _: &mut ClientMut) {
        // TODO: role object must have been destroyed
        self.xdg_surfaces.remove(msg.handle());
    }
}

impl MessageHandler<GetToplevel> for Compositor {
    fn handle(&mut self, msg: Msg<GetToplevel>, client: &mut ClientMut) {
        let xdg_surface = &mut self.xdg_surfaces[msg.handle()];
        let surface = &mut self.surfaces[xdg_surface.surface_handle()];

        let xdg_handle = msg.handle();
        let xdg_toplevel = client.objects.create_with(msg, xdg_handle);
        // TODO: blocker: change the any error handling
        xdg_surface
            .set_toplevel(xdg_toplevel, surface)
            .expect("not yet implemented");
    }
}

todo_handler!(GetPopup);
todo_handler!(SetWindowGeometry);
todo_handler!(AckConfigure);

// ===== xdg_toplevel =====

impl CommitEffect<XdgToplevel> for Compositor {
    fn commit(&mut self, obj: Object<XdgToplevel>, client: &mut ClientMut) -> Result<(), CommitError> {
        client.send(obj.configure(1280, 720, &[]));
        // TODO: send `XdgSurface::configure`
        Ok(())
    }
}

impl MessageHandler<xdg_toplevel::Destroy> for Compositor {
    fn handle(&mut self, msg: Msg<ToplevelDestroy>, _: &mut ClientMut) {
        let xdg_surface = &mut self.xdg_surfaces[msg.handle()];
        let surface = &mut self.surfaces[xdg_surface.surface_handle()];
        xdg_surface.remove_role(surface);
    }
}

todo_handler!(SetParent);

impl MessageHandler<SetTitle<'_>> for Compositor {
    fn handle(&mut self, msg: Msg<SetTitle<'_>>, _: &mut ClientMut) {
        if let Some(toplevel) = self.xdg_surfaces[msg.handle()].as_toplevel() {
            toplevel.set_title(msg.title)
        }
    }
}

impl MessageHandler<SetAppId<'_>> for Compositor {
    fn handle(&mut self, msg: Msg<SetAppId<'_>>, _: &mut ClientMut) {
        if let Some(toplevel) = self.xdg_surfaces[msg.handle()].as_toplevel() {
            toplevel.set_app_id(msg.app_id)
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
