use crate::wayland::prelude::*;
use crate::wayland::wl_surface::WlSurface;

#[derive(Interface, Debug)]
pub struct WlPointer {
    id: ObjectId
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    SetCursor,
    Release,
}

#[derive(Message, Debug)]
#[request(WlPointer)]
pub struct SetCursor {
    pub serial: u32,
    pub surface: Object<WlSurface>,
    pub hotspot_x: i32,
    pub hotspot_y: i32,
}

#[derive(Message, Debug)]
#[request(WlPointer)]
#[destructor]
pub struct Release;

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Enter,
    Leave,
    Motion,
    Button,
    Axis,
    Frame,
    AxisSource,
    AxisStop,
    AxisDiscrete,
    AxisValue120,
    AxisRelativeDirection,
}

#[derive(Message, Debug)]
#[event(WlPointer)]
pub struct Enter {
    pub serial: u32,
    pub surface: Object<WlSurface>,
    pub surface_x: Fixed,
    pub surface_y: Fixed,
}

#[derive(Message, Debug)]
#[event(WlPointer)]
pub struct Leave {
    pub serial: u32,
    pub surface: Object<WlSurface>,
}

#[derive(Message, Debug)]
#[event(WlPointer)]
pub struct Motion {
    pub time: u32,
    pub surface_x: Fixed,
    pub surface_y: Fixed,
}

#[derive(Message, Debug)]
#[event(WlPointer)]
pub struct Button {
    pub serial: u32,
    pub time: u32,
    pub button: u32,
    pub state: ButtonState,
}

#[derive(Message, Debug)]
#[event(WlPointer)]
pub struct Axis {
    pub time: u32,
    pub axis: AxisTypes,
    pub value: Fixed,
}

#[derive(Message, Debug)]
#[event(WlPointer)]
pub struct Frame;

#[derive(Message, Debug)]
#[event(WlPointer)]
pub struct AxisSource {
    pub axis_source: AxisSourceTypes,
}

#[derive(Message, Debug)]
#[event(WlPointer)]
pub struct AxisStop {
    pub time: u32,
    pub axis: AxisTypes,
}

#[derive(Message, Debug)]
#[event(WlPointer)]
pub struct AxisDiscrete {
    pub axis: AxisTypes,
    pub discrete: i32,
}

#[derive(Message, Debug)]
#[event(WlPointer)]
pub struct AxisValue120 {
    pub axis: AxisTypes,
    pub value120: i32,
}

#[derive(Message, Debug)]
#[event(WlPointer)]
pub struct AxisRelativeDirection {
    pub axis: AxisTypes,
    pub direction: AxisRelativeDirectionEnum,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    Role,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum ButtonState {
    Released,
    Pressed,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum AxisTypes {
    VerticalScroll,
    HorizontalScroll,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum AxisSourceTypes {
    Wheel,
    Finger,
    Continuous,
    WheelTilt,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum AxisRelativeDirectionEnum {
    Identical,
    Inverted,
}
