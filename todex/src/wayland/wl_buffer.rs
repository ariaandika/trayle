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
#[request(WlBuffer)]
#[destructor]
pub struct Destroy;

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Release,
}

#[derive(Message, Debug)]
#[event(WlBuffer)]
pub struct Release;
