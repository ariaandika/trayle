use crate::wayland::prelude::*;

simple_object! {
    pub struct WlCallback;
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
    const OPCODE: u16 = EventOp::Done as u16;

    fn object_id(&self) -> Id {
        self.id()
    }

    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.callback_data);
    }
}
