use crate::wayland::prelude::*;
use crate::wayland::wl_surface::WlSurface;

#[derive(Interface, Debug)]
pub struct WlTouch {
    id: ObjectId
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Release,
}

#[derive(Message, Debug)]
#[request(WlTouch)]
pub struct Release;

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Down,
    Up,
    Motion,
    Frame,
    Cancel,
    Shape,
    Orientation,
}

#[derive(Message, Debug)]
#[event(WlTouch)]
pub struct Down {
    pub serial: u32,
    pub time: u32,
    pub surface: Object<WlSurface>,
    pub id: i32,
    pub x: Fixed,
    pub y: Fixed,
}

#[derive(Message, Debug)]
#[event(WlTouch)]
pub struct Up {
    pub serial: u32,
    pub time: u32,
    pub id: i32,
}

#[derive(Message, Debug)]
#[event(WlTouch)]
pub struct Motion {
    pub time: u32,
    pub id: i32,
    pub x: Fixed,
    pub y: Fixed,
}

#[derive(Message, Debug)]
#[event(WlTouch)]
pub struct Frame;

#[derive(Message, Debug)]
#[event(WlTouch)]
pub struct Cancel;

#[derive(Message, Debug)]
#[event(WlTouch)]
pub struct Shape {
    pub id: i32,
    pub major: Fixed,
    pub minor: Fixed,
}

#[derive(Message, Debug)]
#[event(WlTouch)]
pub struct Orientation {
    pub id: i32,
    pub orientation: Fixed,
}
