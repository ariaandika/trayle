use smithay::wayland::drm_syncobj::{DrmSyncobjHandler, DrmSyncobjState};

use super::Trayle;

smithay::delegate_drm_syncobj!(Trayle);

impl DrmSyncobjHandler for Trayle {
    fn drm_syncobj_state(&mut self) -> &mut DrmSyncobjState {
        self.backend.syncobj_state.as_mut().unwrap()
    }
}

