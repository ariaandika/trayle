use crate::wayland::prelude::*;

#[derive(Interface, Debug)]
pub struct WlCallback {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Done,
}

#[derive(Message, Debug)]
#[event(WlCallback)]
pub struct Done {
    pub callback_data: u32,
}

// ===== impls =====

impl WlCallback {
    #[inline]
    pub fn done(&self, callback_data: u32) -> Message<Done> {
        Message::new(self, Done { callback_data })
    }
}
