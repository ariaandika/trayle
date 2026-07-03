use xdg_wm_base::{CreatePositioner, Destroy as XdgWmDestroy, GetXdgSurface, Pong};
use xdg_surface::Destroy as XdgSurfaceDestroy;
use xdg_surface::{AckConfigure, GetPopup, GetToplevel, SetWindowGeometry};
use xdg_toplevel::{*, Destroy as ToplevelDestroy};

use crate::compositor::prelude::*;

// ===== xdg_wm_base =====

impl MessageHandler<XdgWmDestroy> for Compositor {
    fn handle(&mut self, _: Msg<XdgWmDestroy>, _: &mut ClientMut) -> Result<(), WlError> {
        // global object destroy
        Ok(())
    }
}

todo_handler!(CreatePositioner);

impl MessageHandler<GetXdgSurface> for Compositor {
    fn handle(&mut self, msg: Msg<GetXdgSurface>, client: &mut ClientMut) -> Result<(), WlError> {
        let surface_handle = client.objects.get_mut(msg.surface)?.handle();
        let xdg_handle = self.xdg_surfaces.create(surface_handle);
        client.objects.create_handle(msg, xdg_handle)?;
        Ok(())
    }
}

impl MessageHandler<Pong> for Compositor {
    fn handle(&mut self, _: Msg<Pong>, _: &mut ClientMut) -> Result<(), WlError> {
        // TODO: ping pong mechanism
        Ok(())
    }
}

// ===== xdg_positioner =====

// ===== xdg_surface =====

impl MessageHandler<XdgSurfaceDestroy> for Compositor {
    fn handle(&mut self, msg: Msg<XdgSurfaceDestroy>, _: &mut ClientMut) -> Result<(), WlError> {
        // TODO: role object must have been destroyed
        self.xdg_surfaces.remove(msg.handle())?;
        Ok(())
    }
}

impl MessageHandler<GetToplevel> for Compositor {
    fn handle(&mut self, msg: Msg<GetToplevel>, client: &mut ClientMut) -> Result<(), WlError> {
        let xdg_surface = self.xdg_surfaces.get_mut(msg.handle())?;
        let surface = self.surfaces.get_mut(xdg_surface.surface_handle())?;
        // TODO: blocker: change the any error handling
        xdg_surface.get_toplevel(surface).expect("not yet implemented");
        let xdg_handle = msg.handle();
        client.objects.create_handle(msg, xdg_handle)?;
        Ok(())
    }
}

todo_handler!(GetPopup);
todo_handler!(SetWindowGeometry);
todo_handler!(AckConfigure);

// ===== xdg_toplevel =====

impl MessageHandler<ToplevelDestroy> for Compositor {
    fn handle(&mut self, msg: Msg<ToplevelDestroy>, client: &mut ClientMut) -> Result<(), WlError> {
        self.todo(msg, client)
    }
}

todo_handler!(SetParent);

impl MessageHandler<SetTitle<'_>> for Compositor {
    fn handle(&mut self, msg: Msg<SetTitle<'_>>, _: &mut ClientMut) -> Result<(), WlError> {
        self.xdg_surfaces.get_mut(msg.handle())?.set_title(msg.title);
        Ok(())
    }
}

impl MessageHandler<SetAppId<'_>> for Compositor {
    fn handle(&mut self, msg: Msg<SetAppId<'_>>, _: &mut ClientMut) -> Result<(), WlError> {
        self.xdg_surfaces.get_mut(msg.handle())?.set_app_id(msg.app_id);
        Ok(())
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
