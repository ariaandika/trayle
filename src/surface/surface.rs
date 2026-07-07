use todex::wayland::object::Object;
use todex::wayland::interface::wl_surface::Damage;
use todex::wayland::interface::WlCallback;

use crate::handle::Handle;
use crate::shm::Buffer;
use crate::surface::{Region, Role, RoleError};

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

    pub fn is_configured(&self) -> bool {
        self.flags & IS_CONFIGURED_FLAG == IS_CONFIGURED_FLAG
    }

    pub fn set_configured(&mut self) {
        self.flags &= IS_CONFIGURED_FLAG;
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn set_role(&mut self, role: Role) -> Result<(), RoleError> {
        if self.role.is_none() && !role.is_none() {
            self.role = role;
            Ok(())
        } else {
            Err(RoleError::Overwrite)
        }
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

    pub fn attach(&mut self, buffer: Option<Handle<Buffer>>) {
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

    pub fn release_current_buffer(&mut self) -> Option<Handle<Buffer>> {
        self.current_mut().buffer.take()
    }
}

// ===== State =====

struct State {
    request_frames: Option<Object<WlCallback>>,
    buffer: Option<Handle<Buffer>>,
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
