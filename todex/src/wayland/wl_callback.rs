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
#[destructor]
pub struct Done {
    pub callback_data: u32,
}
