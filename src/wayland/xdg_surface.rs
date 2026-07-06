use todex::wayland::object::{Handle, Object};
use todex::wayland::interface::xdg_surface::{Error, XdgSurface as IXdgSurface};
use todex::wayland::interface::XdgToplevel;

use crate::wayland::surface::{Surface, Role};

pub struct XdgSurface {
    surface_handle: Handle,
    title: Option<Box<str>>,
    app_id: Option<Box<str>>,
}

impl XdgSurface {
    pub fn new(surface_handle: Handle) -> Self {
        Self {
            surface_handle,
            title: None,
            app_id: None,
        }
    }

    pub fn get_toplevel(
        &mut self,
        xdg_toplevel: Object<XdgToplevel>,
        surface: &mut Surface,
    ) -> Result<(), Error> {
        surface
            .set_role(Role::XdgToplevel(xdg_toplevel))
            .map_err(|_| Error::AlreadyConstructed)
    }

    pub fn surface_handle(&self) -> Handle {
        self.surface_handle
    }

    pub fn set_title<S: Into<Box<str>>>(&mut self, title: S) {
        self.title = Some(title.into());
    }

    pub fn set_app_id<S: Into<Box<str>>>(&mut self, app_id: S) {
        self.app_id = Some(app_id.into());
    }
}
