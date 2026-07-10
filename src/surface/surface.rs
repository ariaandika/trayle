use todex::wayland::interface::wl_output::Transform;
use todex::wayland::object::Object;
use todex::wayland::interface::WlCallback;

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
    states: [State; 2],
    /// [.., configured, current]
    flags: u8,
    role: RoleInner,
}

enum RoleInner {
    None,
    Role(Role),
    Removed,
}

impl RoleInner {
    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

const COMMITED_FLAG: u8 = 1;
const IS_CONFIGURED_FLAG: u8 = 1 << 1;

impl Surface {
    pub fn new() -> Self {
        Self {
            states: [State::new(), State::new()],
            flags: 0,
            role: RoleInner::None,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.flags & IS_CONFIGURED_FLAG == IS_CONFIGURED_FLAG
    }

    pub fn set_configured(&mut self) {
        self.flags &= IS_CONFIGURED_FLAG;
    }

    pub fn role(&self) -> Result<Role, RoleError> {
        match self.role {
            RoleInner::Role(role) => Ok(role),
            RoleInner::None => Err(RoleError::Unset),
            RoleInner::Removed => Err(RoleError::Removed),
        }
    }

    pub fn has_role(&self) -> bool {
        matches!(self.role, RoleInner::Role(_))
    }

    /// Must only be called by role object.
    pub(super) fn set_role(&mut self, role: Role) -> Result<(), RoleError> {
        if self.role.is_none() {
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

    pub fn commit(&mut self) {
        self.flags &= !self.flags & COMMITED_FLAG;
    }

    pub fn destroy(self) {
        // maybe called explicitly by client, or at client disconnect,
        // in latter case the role object may still exists

        // TODO: surface: on destroy
        // - what happens with the Buffer ?
        // - what happens with request frame callback ?
        // - should pending and current state treated differently ?
    }
}

/// Pending
impl Surface {
    fn pending(&self) -> &State {
        &self.states[(!self.flags & COMMITED_FLAG) as usize]
    }

    fn pending_mut(&mut self) -> &mut State {
        &mut self.states[(!self.flags & COMMITED_FLAG) as usize]
    }

    pub fn has_pending_buffer(&self) -> bool {
        self.pending().buffer.is_some()
    }

    /// Set a buffer as the content of this surface.
    pub fn attach(&mut self, buffer: Handle<Buffer>) {
        self.pending_mut().buffer = Some(buffer);
    }

    /// Remove and returns the buffer of this surface.
    pub fn unattach(&mut self) -> Option<Handle<Buffer>> {
        self.pending_mut().buffer.take()
    }

    /// Offset buffer coordinate relatively in surface-local coordinates.
    pub fn offset(&mut self, x: i32, y: i32) {
        self.pending_mut().offset.0 += x;
        self.pending_mut().offset.1 += y;
    }

    /// Describe the regions where the pending buffer is different from the current surface
    /// contents.
    pub fn damage(&mut self, region: Region) {
        self.pending_mut().damage.union(region);
    }

    pub fn request_frame(&mut self, callback: Object<WlCallback>) {
        // TODO: surface: handle stacking frame requests
        self.pending_mut().request_frames = Some(callback);
    }

    pub fn request_release(&mut self, callback: Object<WlCallback>) {
        // TODO: surface: handle stacking release requests
        self.pending_mut().request_release = Some(callback);
    }

    /// Set buffer transform.
    pub fn set_transform(&mut self, transform: Transform) {
        self.pending_mut().transform = transform;
    }

    /// Set buffer scale.
    pub fn set_scale(&mut self, scale: i32) {
        self.pending_mut().scale = scale;
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
    // callbacks
    request_frames: Option<Object<WlCallback>>,
    request_release: Option<Object<WlCallback>>,

    // buffer
    buffer: Option<Handle<Buffer>>,
    offset: (i32, i32),
    damage: Region,
    transform: Transform,
    scale: i32,
    // opaque region,
    // input region,
}

impl State {
    pub fn new() -> Self {
        Self {
            request_frames: None,
            request_release: None,
            buffer: None,
            offset: (0, 0),
            damage: Region::new(),
            transform: Transform::Normal,
            scale: 1,
        }
    }
}
