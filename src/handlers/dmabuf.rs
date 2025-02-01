use crate::Trayle;
use smithay::{
    backend::allocator::dmabuf::Dmabuf,
    wayland::dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier},
};


smithay::delegate_dmabuf!(@<B: 'static> Trayle<B>);

impl<B> DmabufHandler for Trayle<B> {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf_state
    }

    fn dmabuf_imported(&mut self, _global: &DmabufGlobal, _dmabuf: Dmabuf, _notifier: ImportNotifier) {
        // todo!()
    }
}

