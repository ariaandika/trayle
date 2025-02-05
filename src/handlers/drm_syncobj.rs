use smithay::wayland::drm_syncobj::{DrmSyncobjHandler, DrmSyncobjState};

use crate::Trayle;

smithay::delegate_drm_syncobj!(Trayle);

impl DrmSyncobjHandler for Trayle {
    fn drm_syncobj_state(&mut self) -> &mut DrmSyncobjState {
        self.backend.syncobj_state.as_mut().expect("drm syncobj not supported by gpu")
    }
}

