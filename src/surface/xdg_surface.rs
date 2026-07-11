use todex::wayland::object::Object;
use todex::wayland::interface::xdg_surface::{self, Error};
use todex::wayland::interface::{XdgToplevel, XdgSurface as IXdgSurface};

use crate::handle::Handle;
use crate::surface::{Surface, Role};

// ===== XdgSurface =====

pub struct XdgSurface {
    // reference
    object: Object<IXdgSurface>,
    surface: Handle<Surface>,

    // ack
    pending_ack: u16,
    current_ack: u16,

    // role
    role: XdgRole,
}

enum XdgRole {
    None,
    Toplevel(Toplevel),
    // Popup,
    Removed,
}

impl XdgSurface {
    pub fn new(object: Object<IXdgSurface>, surface: Handle<Surface>) -> Self {
        Self {
            object,
            surface,
            pending_ack: 0,
            current_ack: 0,
            role: XdgRole::None,
        }
    }

    pub fn object(&self) -> Object<IXdgSurface> {
        self.object
    }

    pub fn surface(&self) -> Handle<Surface> {
        self.surface
    }
}

/// Role
impl XdgSurface {
    /// Set role as `XdgToplevel`.
    pub fn set_toplevel_role(
        &mut self,
        xdg_toplevel: Object<XdgToplevel>,
        surface: &mut Surface,
    ) -> Result<(), xdg_surface::Error> {
        self.role = XdgRole::Toplevel(Toplevel::new());
        surface
            .set_role(Role::XdgToplevel(xdg_toplevel))
            .map_err(|_| Error::AlreadyConstructed)
    }

    pub fn as_toplevel(&mut self) -> Option<&mut Toplevel> {
        match &mut self.role {
            XdgRole::Toplevel(toplevel) => Some(toplevel),
            _ => None,
        }
    }

    pub fn remove_role(&mut self, surface: &mut Surface) {
        self.role = XdgRole::Removed;
        surface.remove_role();
    }
}

/// Role
impl XdgSurface {
    /// Add a pending configure acknowledge, returns the serial.
    pub fn next_ack(&mut self) -> u32 {
        self.current_ack += self.current_ack;
        self.current_ack as u32
    }

    /// Confirms a configure ack with given serial.
    pub fn ack(&mut self, serial: u32) -> Result<(), xdg_surface::Error> {
        let serial = serial as u16;
        let ok = serial >= self.pending_ack && serial <= self.current_ack;
        if !ok {
            return Err(xdg_surface::Error::InvalidSerial);
        }
        self.pending_ack = serial;
        if self.pending_ack == self.current_ack {
            self.pending_ack = 0;
            self.current_ack = 0;
        }
        Ok(())
    }
}

// ===== Toplevel =====

pub struct Toplevel {
    pub title: Option<Box<str>>,
    pub app_id: Option<Box<str>>,
}

impl Toplevel {
    pub fn new() -> Self {
        Self {
            title: None,
            app_id: None,
        }
    }
}
