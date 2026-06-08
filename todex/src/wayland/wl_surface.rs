use crate::wayland::prelude::*;
use crate::wayland::wl_callback::WlCallback;
use crate::wayland::wl_output::Transform;

#[derive(Debug, Interface)]
pub struct WlSurface {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Destroy,
    Attach,
    Damage,
    Frame,
    SetOpaqueRegion,
    SetInputRegion,
    Commit,
    SetBufferTransform,
    SetBufferScale,
    DamageBuffer,
    Offset,
    GetRelease,
}

#[derive(Message, Debug)]
#[request(WlSurface)]
pub struct Destroy;

#[derive(Message, Debug)]
#[request(WlSurface)]
pub struct Attach {
    /// <wl_buffer>
    pub buffer: Option<ObjectId>,
    pub x: i32,
    pub y: i32,
}

#[derive(Message, Debug)]
#[request(WlSurface)]
pub struct Damage {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Message, Debug)]
#[request(WlSurface)]
pub struct Frame {
    pub callback: NewId<WlCallback>,
}

#[derive(Message, Debug)]
#[request(WlSurface)]
pub struct SetOpaqueRegion {
    /// <wl_region>
    pub region: Option<ObjectId>,
}

#[derive(Message, Debug)]
#[request(WlSurface)]
pub struct SetInputRegion {
    /// <wl_region>
    pub region: Option<ObjectId>,
}

#[derive(Message, Debug)]
#[request(WlSurface)]
pub struct Commit;

#[derive(Message, Debug)]
#[request(WlSurface)]
pub struct SetBufferTransform {
    pub transform: Transform,
}

#[derive(Message, Debug)]
#[request(WlSurface)]
pub struct SetBufferScale {
    pub scale: i32,
}

#[derive(Message, Debug)]
#[request(WlSurface)]
pub struct DamageBuffer {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Message, Debug)]
#[request(WlSurface)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
}

#[derive(Message, Debug)]
#[request(WlSurface)]
pub struct GetRelease {
    pub callback: NewId<WlCallback>,
}

#[derive(Message, Debug)]
#[event(WlSurface)]
pub struct Enter {
    /// <wl_output>
    pub output: ObjectId,
}

#[derive(Message, Debug)]
#[event(WlSurface)]
pub struct Leave {
    /// <wl_output>
    pub output: ObjectId,
}

#[derive(Message, Debug)]
#[event(WlSurface)]
pub struct PreferredBufferScale {
    pub factor: i32,
}

#[derive(Message, Debug)]
#[event(WlSurface)]
pub struct PreferredBufferTransform {
    pub transform: Transform,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Enter,
    Leave,
    PreferredBufferScale,
    PreferredBufferTransform,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    /// buffer scale value is invalid
    InvalidScale,
    /// buffer transform value is invalid
    InvalidTransform,
    /// buffer size is invalid
    InvalidSize,
    /// buffer offset is invalid
    InvalidOffset,
    /// surface was destroyed before its role object
    DefunctRoleObject,
    /// no buffer was attached
    NoBuffer,
}
