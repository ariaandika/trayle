use crate::wayland::prelude::*;

#[derive(Interface, Debug)]
pub struct WlRegion {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Destroy,
    Add,
    Subtract,
}

#[derive(Message, Debug)]
#[request(WlRegion)]
pub struct Destroy;

#[derive(Message, Debug)]
#[request(WlRegion)]
pub struct Add {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Message, Debug)]
#[request(WlRegion)]
pub struct Subtract {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
