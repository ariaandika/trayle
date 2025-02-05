use crate::Trayle;
use smithay::wayland::shm::{ShmHandler, ShmState};

smithay::delegate_shm!(Trayle);

impl ShmHandler for Trayle {
    fn shm_state(&self) -> &ShmState {
        &self.frontend.shm_state
    }
}

