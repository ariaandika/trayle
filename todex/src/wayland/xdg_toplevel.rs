use crate::wayland::prelude::*;
use crate::wayland::wl_output::WlOutput;
use crate::wayland::wl_seat::WlSeat;

#[derive(Interface, Debug)]
pub struct XdgToplevel {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Destroy,
    SetParent,
    SetTitle,
    SetAppId,
    ShowWindowMenu,
    Move,
    Resize,
    SetMaxSize,
    SetMinSize,
    SetMaximized,
    UnsetMaximized,
    SetFullscreen,
    UnsetFullscreen,
    SetMinimized,
}

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct SetParent {
    pub parent: Option<Object<XdgToplevel>>,
}

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct SetTitle<'a> {
    pub title: &'a str,
}

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct SetAppId<'a> {
    pub app_id: &'a str,
}

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct ShowWindowMenu {
    pub seat: Object<WlSeat>,
    pub serial: u32,
    pub x: i32,
    pub y: i32,
}

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct Move {
    pub seat: Object<WlSeat>,
    pub serial: u32,
}

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct Resize {
    pub seat: Object<WlSeat>,
    pub serial: u32,
    pub edges: ResizeEdge,
}

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct SetMaxSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct SetMinSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct SetMaximized;

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct UnsetMaximized;

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct SetFullscreen {
    pub output: Option<Object<WlOutput>>,
}

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct UnsetFullscreen;

#[derive(Message, Debug)]
#[request(XdgToplevel)]
pub struct SetMinimized;

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Configure,
    Close,
    ConfigureBounds,
    WmCapabilities,
}

#[derive(Message, Debug)]
#[event(XdgToplevel)]
pub struct Configure {
    pub serial: u32,
}

#[derive(Message, Debug)]
#[event(XdgToplevel)]
pub struct Close;

#[derive(Message, Debug)]
#[event(XdgToplevel)]
pub struct ConfigureBounds {
    pub width: i32,
    pub height: i32,
}

#[derive(Message, Debug)]
#[event(XdgToplevel)]
pub struct WmCapabilities<'a> {
    pub capabilities: &'a [u8],
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    /// provided value is not a valid variant of the resize_edge enum
    InvalidResizeEdge,
    /// invalid parent toplevel
    InvalidParent,
    /// client provided an invalid min or max size
    InvalidSize,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum ResizeEdge {
    None,
    Top,
    Bottom,
    Left,
    TopLeft,
    BottomLeft,
    Right,
    TopRight,
    BottomRight,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum State {
    Maximized,
    Fullscreen,
    Resizing,
    Activated,
    TiledLeft,
    TiledRight,
    TiledTop,
    TiledBottom,
    Suspended,
    ConstrainedLeft,
    ConstrainedRight,
    ConstrainedTop,
    ConstrainedBottom,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum WmCapabilitiesEnum {
    WindowMenu,
    Maximize,
    Fullscreen,
    Minimize,
}
