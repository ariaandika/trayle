use crate::Trayle;
use smithay::{
    backend::allocator::dmabuf::Dmabuf,
    wayland::dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier},
};

smithay::delegate_dmabuf!(Trayle);

/// required for [`DmabufState::create_global_with_default_feedback`]
///
/// in case of tty backend, when the initial configuration of primary gpu
impl DmabufHandler for Trayle {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.frontend.dmabuf_state
    }

    fn dmabuf_imported(&mut self, _global: &DmabufGlobal, dmabuf: Dmabuf, notifier: ImportNotifier) {
        match self.backend.import_dmabuf(&dmabuf) {
            Ok(_texture) => {
                match notifier.successful::<Self>() {
                    Ok(_buffer) => {},
                    Err(id) => {
                        tracing::error!(?id,"the client has died")
                    },
                }
            },
            Err(err) => {
                tracing::error!("{err:?}");
                notifier.failed();
            },
        }
    }
}

