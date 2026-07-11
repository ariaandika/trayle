use todex::wayland::object::Object;
use todex::wayland::interface::wl_output::Transform;
use todex::wayland::interface::{WlCallback, WlRegion};

use crate::handle::Handle;
use crate::shm::Buffer;
use crate::surface::{Region, Role, RoleError};

/// Wayland Surface.
///
/// Surface can:
/// - present wl_buffers
/// - receive user input
/// - define a local coordinate system
pub struct Surface {
    states: [SurfaceState; 2],
    /// [.., configured, current]
    flags: u8,
    role: RoleInner,
}

enum RoleInner {
    None,
    Role(Role),
    Removed,
}

const COMMITED_FLAG: u8 = 1;
const IS_CONFIGURED_FLAG: u8 = 1 << 1;

impl Surface {
    pub fn new() -> Self {
        Self {
            states: [SurfaceState::new(), SurfaceState::new()],
            flags: 0,
            role: RoleInner::None,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.flags & IS_CONFIGURED_FLAG == IS_CONFIGURED_FLAG
    }

    pub fn set_configured(&mut self) {
        self.flags |= IS_CONFIGURED_FLAG;
    }

    /// Returns the surface [`Role`] if any.
    pub fn role(&self) -> Option<Role> {
        match self.role {
            RoleInner::Role(role) => Some(role),
            _ => None,
        }
    }

    /// Returns the surface [`Role`] if any.
    #[expect(dead_code)]
    pub fn try_role(&self) -> Result<Role, RoleError> {
        match self.role {
            RoleInner::None => Err(RoleError::Unset),
            RoleInner::Role(role) => Ok(role),
            RoleInner::Removed => Err(RoleError::Removed),
        }
    }

    pub fn has_role(&self) -> bool {
        matches!(self.role, RoleInner::Role(_))
    }

    /// Must only be called by role object.
    pub(super) fn set_role(&mut self, role: Role) -> Result<(), RoleError> {
        if matches!(self.role, RoleInner::None) {
            self.role = RoleInner::Role(role);
            Ok(())
        } else {
            Err(RoleError::Overwrite)
        }
    }

    /// Must only be called by role object.
    pub(super) fn remove_role(&mut self) {
        self.role = RoleInner::Removed;
    }
}

impl Surface {
    pub fn swap_state(&mut self) {
        self.flags ^= COMMITED_FLAG;
    }

    /// Returns mutable refernce to the pending state.
    pub fn pending_mut(&mut self) -> &mut SurfaceState {
        &mut self.states[(!self.flags & COMMITED_FLAG) as usize]
    }

    /// Returns shared refernce to the current state.
    pub fn current(&self) -> &SurfaceState {
        &self.states[(self.flags & COMMITED_FLAG) as usize]
    }

    /// Returns mutable refernce to the current state.
    pub fn current_mut(&mut self) -> &mut SurfaceState {
        &mut self.states[(self.flags & COMMITED_FLAG) as usize]
    }
}

impl Surface {
    pub fn destroy(self) {
        // maybe called explicitly by client, or at client disconnect,
        // in latter case the role object may still exists

        // TODO: surface: on destroy
        // - what happens with the Buffer ?
        // - what happens with request frame callback ?
        // - should pending and current state treated differently ?
    }
}

// ===== State =====

pub struct SurfaceState {
    // callbacks
    pub request_frames: Option<Object<WlCallback>>,
    pub request_release: Option<Object<WlCallback>>,

    // buffer
    pub buffer: Option<Handle<Buffer>>,
    pub offset: (i32, i32),
    pub damage: Region,
    pub transform: Transform,
    pub scale: i32,
    pub opaque: Option<Object<WlRegion>>,
    pub input: Option<Object<WlRegion>>,
}

impl SurfaceState {
    pub fn new() -> Self {
        Self {
            request_frames: None,
            request_release: None,
            buffer: None,
            offset: (0, 0),
            damage: Region::new(),
            transform: Transform::Normal,
            scale: 1,
            opaque: None,
            input: None,
        }
    }
}
