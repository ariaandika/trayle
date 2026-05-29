use crate::wayland::prelude::*;

pub struct WlCallback {
    id: Id,
}

impl FromId for WlCallback {
    #[inline]
    fn from_id(id: Id) -> Self {
        Self { id }
    }
}

impl Object for WlCallback {
    const INTERFACE_ID: InterfaceId = InterfaceId::WlCallback;

    fn id(&self) -> Id {
        self.id
    }
}

impl WlCallback {
    pub fn done(&self, callback_data: u32) -> Message<Done> {
        Message::new(self, Done { callback_data })
    }
}

// ===== Op =====

opcode! {
    pub enum EventOp {
        Done,
    }
}

// ===== Done =====

pub struct Done {
    callback_data: u32,
}

impl Encode for Message<Done> {
    fn encode(self, encoder: Encoder) {
        encoder.encode_one(self.id(), EventOp::Done, self.callback_data);
    }
}
