use crate::wayland::prelude::*;

pub struct WlCallback {
    id: Id,
}

impl WlCallback {
    #[inline]
    pub fn new(id: Id) -> Self {
        Self { id }
    }

    pub fn done(&self, callback_data: u32) -> Message<Done> {
        Message::new(self, Done { callback_data })
    }
}

impl Object for WlCallback {
    const INTERFACE_ID: InterfaceId = InterfaceId::WlCallback;

    fn id(&self) -> Id {
        self.id
    }
}

// ===== Op =====

#[derive(Debug, Clone, Copy)]
pub enum EventOp {
    Done,
}

impl ToOp for EventOp {
    fn to_op(&self) -> u16 {
        *self as u16
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
