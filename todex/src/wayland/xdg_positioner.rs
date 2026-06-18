use crate::wayland::prelude::*;

#[derive(Interface, Debug)]
pub struct XdgPositioner {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Destroy,
    SetSize,
    SetAnchorRect,
    SetAnchor,
    SetGravity,
    SetConstraintAdjustment,
    SetOffset,
    SetReactive,
    SetParentSize,
    SetParentConfigure,
}

#[derive(Message, Debug)]
#[request(XdgPositioner)]
#[destructor]
pub struct Destroy;

#[derive(Message, Debug)]
#[request(XdgPositioner)]
pub struct SetSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Message, Debug)]
#[request(XdgPositioner)]
pub struct SetAnchorRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Message, Debug)]
#[request(XdgPositioner)]
pub struct SetAnchor {
    pub anchor: Anchor,
}

#[derive(Message, Debug)]
#[request(XdgPositioner)]
pub struct SetGravity {
    pub gravity: Gravity,
}

#[derive(Message, Debug)]
#[request(XdgPositioner)]
pub struct SetConstraintAdjustment {
    pub adjust: ConstraintAdjustment,
}

#[derive(Message, Debug)]
#[request(XdgPositioner)]
pub struct SetOffset {
    pub x: i32,
    pub y: i32,
}

#[derive(Message, Debug)]
#[request(XdgPositioner)]
pub struct SetReactive;

#[derive(Message, Debug)]
#[request(XdgPositioner)]
pub struct SetParentSize {
    pub parent_width: i32,
    pub parent_height: i32,
}

#[derive(Message, Debug)]
#[request(XdgPositioner)]
pub struct SetParentConfigure {
    pub serial: u32,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    /// invalid input provided
    InvalidInput,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Anchor {
    None,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    BottomLeft,
    TopRight,
    BottomRight,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Gravity {
    None,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    BottomLeft,
    TopRight,
    BottomRight,
}

#[derive(Debug, Clone, Copy)]
pub struct ConstraintAdjustment(u32);

bitfield! {
    ConstraintAdjustment;

    None = 0,
    SlideX = 1,
    SlideY = 2,
    FlipX = 4,
    FlipY = 8,
    ResizeX = 16,
    ResizeY = 32,
}
