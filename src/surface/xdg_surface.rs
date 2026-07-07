use todex::handle::Handle;
use todex::wayland::object::Object;
use todex::wayland::interface::xdg_surface::Error;
use todex::wayland::interface::XdgToplevel;

use crate::surface::{Surface, Role};

// ===== XdgSurface =====

pub struct XdgSurface {
    surface_handle: Handle<Surface>,
    kind: Kind,
}

enum Kind {
    None,
    Toplevel(Toplevel),
    // Popup,
}

impl XdgSurface {
    pub fn new(surface_handle: Handle<Surface>) -> Self {
        Self {
            surface_handle,
            kind: Kind::None,
        }
    }

    /// Set role as `XdgToplevel`.
    pub fn set_toplevel(
        &mut self,
        xdg_toplevel: Object<XdgToplevel>,
        surface: &mut Surface,
    ) -> Result<(), Error> {
        self.kind = Kind::Toplevel(Toplevel::new());
        surface
            .set_role(Role::XdgToplevel(xdg_toplevel))
            .map_err(|_| Error::AlreadyConstructed)
    }

    pub fn surface_handle(&self) -> Handle<Surface> {
        self.surface_handle
    }

    pub fn as_toplevel(&mut self) -> Option<&mut Toplevel> {
        match &mut self.kind {
            Kind::Toplevel(toplevel) => Some(toplevel),
            _ => None,
        }
    }
}

// ===== Toplevel =====

pub struct Toplevel {
    title: Option<Box<str>>,
    app_id: Option<Box<str>>,
}

impl Toplevel {
    pub fn new() -> Self {
        Self {
            title: None,
            app_id: None,
        }
    }

    pub fn set_title<S: Into<Box<str>>>(&mut self, title: S) {
        self.title = Some(title.into());
    }

    pub fn set_app_id<S: Into<Box<str>>>(&mut self, app_id: S) {
        self.app_id = Some(app_id.into());
    }
}
