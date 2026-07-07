use todex::wayland::object::{Object, Handle};
use todex::wayland::interface::wl_surface::Damage;
use todex::wayland::interface::{WlCallback, XdgToplevel};

pub struct Surface {
    states: [State; 2],
    /// [.., configured, current]
    flags: u8,
    role: Role,
}

const COMMITED_FLAG: u8 = 1;
const IS_CONFIGURED_FLAG: u8 = 1 << 1;

impl Surface {
    pub fn new() -> Self {
        Self {
            states: [State::new(), State::new()],
            flags: 0,
            role: Role::None,
        }
    }

    pub fn set_role(&mut self, role: Role) -> Result<(), RoleError> {
        if self.role.is_none() && !role.is_none() {
            self.role = role;
            Ok(())
        } else {
            Err(RoleError::Overwrite)
        }
    }

    pub fn is_configured(&self) -> bool {
        self.flags & IS_CONFIGURED_FLAG != IS_CONFIGURED_FLAG
    }

    pub fn set_configured(&mut self) {
        self.flags &= IS_CONFIGURED_FLAG;
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn commit(&mut self) {
        self.flags &= !self.flags & COMMITED_FLAG;
    }
}

/// Pending
impl Surface {
    fn pending_mut(&mut self) -> &mut State {
        &mut self.states[(!self.flags & COMMITED_FLAG) as usize]
    }

    pub fn attach(&mut self, buffer: Option<Handle>) {
        self.pending_mut().buffer = buffer;
    }

    pub fn damage(&mut self, damage: Damage) {
        self.pending_mut().damage.damage(damage);
    }

    pub fn request_frame(&mut self, callback: Object<WlCallback>) {
        // TODO: surface: handle stacking frame requests
        self.pending_mut().request_frames = Some(callback);
    }
}

/// Current
impl Surface {
    fn current_mut(&mut self) -> &mut State {
        &mut self.states[(self.flags & COMMITED_FLAG) as usize]
    }

    /// Get all request frames callback id.
    pub fn request_frames(&mut self) -> impl Iterator<Item = Object<WlCallback>> {
        // TODO: surface: handle stacking frame requests
        self.current_mut().request_frames.take().into_iter()
    }

    pub fn release_current_buffer(&mut self) -> Option<Handle> {
        self.current_mut().buffer.take()
    }
}

// ===== State =====

struct State {
    request_frames: Option<Object<WlCallback>>,
    buffer: Option<Handle>,
    damage: Region,
    // opaque region,
    // input region,
    // buffer transform,
    // buffer scale,
    // damage buffer,
    // offset
}

impl State {
    pub fn new() -> Self {
        Self {
            request_frames: None,
            buffer: None,
            damage: Region::new(),
        }
    }
}

// ===== Role =====

/// Surface role.
#[derive(Debug, Clone, Copy)]
pub enum Role {
    None,
    XdgToplevel(Object<XdgToplevel>),
}

impl Role {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// An error that occur in role related operation.
#[derive(Debug)]
pub enum RoleError {
    /// Role is unset.
    Unset,
    /// Role is overwritten.
    Overwrite,
}

impl std::fmt::Display for RoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unset => write!(f, "role is unset"),
            Self::Overwrite => write!(f, "role is overwritten"),
        }
    }
}

// ===== Region =====

struct Region {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Region {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    fn damage(&mut self, damage: Damage) {
        self.x = damage.x;
        self.y = damage.y;
        self.width = damage.width;
        self.height = damage.height;
    }
}
