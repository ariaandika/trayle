use crate::Trayle;
use smithay::wayland::shm::{ShmHandler, ShmState};

smithay::delegate_shm!(@<B: 'static> Trayle<B>);

impl<B> ShmHandler for Trayle<B> {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

