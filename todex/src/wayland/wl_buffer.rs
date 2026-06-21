use crate::wayland::prelude::*;

#[derive(Interface, Debug)]
pub struct WlBuffer {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Destroy,
}

#[derive(Message, Debug)]
#[message(request = WlBuffer, destructor)]
pub struct Destroy;

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Release,
}

#[derive(Message, Debug)]
#[message(event = WlBuffer)]
pub struct Release;
