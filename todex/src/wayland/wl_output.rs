use crate::wayland::prelude::*;

#[derive(Interface, Debug)]
pub struct WlOutput {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Release
}

#[derive(Message, Debug, Clone, Copy)]
#[message(request = WlOutput, destructor)]
pub struct Release;

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Geometry,
    Mode,
    Done,
    Scale,
    Name,
    Description,
}

#[derive(Message, Debug, Clone, Copy)]
#[message(event = WlOutput)]
pub struct Geometry<'a> {
    pub x: i32,
    pub y: i32,
    pub physical_width: i32,
    pub physical_height: i32,
    pub subpixel: Subpixel,
    pub make: &'a str,
    pub model: &'a str,
    pub transform: Transform,
}

#[derive(Message, Debug, Clone, Copy)]
#[message(event = WlOutput)]
pub struct Mode {
    pub flags: ModeFlag,
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
}

#[derive(Message, Debug, Clone, Copy)]
#[message(event = WlOutput)]
pub struct Done;

#[derive(Message, Debug, Clone, Copy)]
#[message(event = WlOutput)]
pub struct Scale {
    pub factor: i32,
}

#[derive(Message, Debug, Clone, Copy)]
#[message(event = WlOutput)]
pub struct Name<'a> {
    pub name: &'a str,
}

#[derive(Message, Debug, Clone, Copy)]
#[message(event = WlOutput)]
pub struct Description<'a> {
    pub name: &'a str,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Subpixel {
    Unknown,
    None,
    HorizontalRgb,
    HorizontalBgr,
    VerticalRgb,
    VerticalBgr,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Transform {
    Normal,
    _90,
    _180,
    _270,
    Flipped,
    Flipped90,
    Flipped180,
    Flipped270,
}

#[derive(Debug, Clone, Copy)]
pub struct ModeFlag(u32);

bitfield! {
    ModeFlag;
    /// indicates this is the current mode
    Current = 0x1,
    /// indicates this is the preferred mode
    Preferred = 0x2,
}
