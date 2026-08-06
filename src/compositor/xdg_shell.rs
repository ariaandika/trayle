use xdg_wm_base::GetXdgSurface;
use xdg_surface::*;
use xdg_toplevel::Destroy as ToplevelDestroy;

use crate::surface::{XdgSurface, XdgSurfaces};
use crate::compositor::prelude::*;
use crate::compositor::traits::CommitError;

// ===== xdg_wm_base =====

pub fn get_xdg_surface(
    msg: Msg<GetXdgSurface>,
    client: &mut ClientMut,
    xdg_surfaces: &mut XdgSurfaces,
) -> Result<(), UnknownId> {
    let surface = client.get_with(msg.surface)?;

    let xdg_surface_obj = Object::from_new_id(msg.new_id);
    let xdg_surface = XdgSurface::new(xdg_surface_obj, surface.handle());
    let xdg_handle = xdg_surfaces.create(xdg_surface);
    client.create_with(msg, xdg_handle);

    Ok(())
}

// ===== xdg_positioner =====

// ===== xdg_surface =====

pub fn get_toplevel(
    msg: Msg<GetToplevel>,
    client: &mut ClientMut,
    res: &mut Resources,
) -> Result<(), xdg_surface::Error> {
    let xdg_surface_handle = msg.handle();
    let xdg_surface = &mut res.xdg_surfaces[xdg_surface_handle];
    let surface = &mut res.surfaces[xdg_surface.surface()];

    let xdg_toplevel = client.objects.create_with(msg, xdg_surface_handle);
    xdg_surface.set_toplevel_role(xdg_toplevel, surface)
}

// ===== xdg_toplevel =====

pub fn toplevel_commit(
    is_configured: bool,
    obj: Object<XdgToplevel>,
    client: &mut ClientMut,
    xdg_surfaces: &mut XdgSurfaces,
) -> Result<(), CommitError> {
    if is_configured {
        client.send(obj.close());
    } else {
        let toplevel = client.objects.get_with(obj)?;
        let xdg_surface = &mut xdg_surfaces[toplevel.handle()];
        let xdg_surface_obj = xdg_surface.object();

        let serial = xdg_surface.next_ack();
        client.send(obj.configure(0, 0, &[]));
        client.send(xdg_surface_obj.configure(serial));
    }

    Ok(())
}

pub fn toplevel_destroy(msg: Msg<ToplevelDestroy>, res: &mut Resources) {
    let xdg_surface = &mut res.xdg_surfaces[msg.handle()];
    let surface = &mut res.surfaces[xdg_surface.surface()];
    xdg_surface.remove_role(surface);
}
