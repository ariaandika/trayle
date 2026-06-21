use crate::wayland::prelude::*;
use crate::wayland::wl_surface::WlSurface;

#[derive(Interface, Debug)]
pub struct WlKeyboard {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Release,
}

#[derive(Message, Debug)]
#[message(request = WlKeyboard, destructor)]
pub struct Release;

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Keymap,
    Enter,
    Leave,
    Key,
    Modifiers,
    RepeatInfo,
}

#[derive(Message, Debug)]
#[message(event = WlKeyboard)]
pub struct Keymap {
    pub format: KeymapFormat,
    #[fd]
    pub fd: i32,
    pub size: u32,
}

#[derive(Message, Debug)]
#[message(event = WlKeyboard)]
pub struct Enter<'a> {
    pub serial: u32,
    pub surface: Object<WlSurface>,
    pub keys: &'a [u8],
}

#[derive(Message, Debug)]
#[message(event = WlKeyboard)]
pub struct Leave {
    pub serial: u32,
    pub surface: Object<WlSurface>,
}

#[derive(Message, Debug)]
#[message(event = WlKeyboard)]
pub struct Key {
    pub serial: u32,
    pub time: u32,
    pub key: u32,
    pub state: KeyState,
}

#[derive(Message, Debug)]
#[message(event = WlKeyboard)]
pub struct Modifiers {
    pub serial: u32,
    pub mods_depressed: u32,
    pub mods_latched: u32,
    pub mods_locket: u32,
    pub group: u32,
}

#[derive(Message, Debug)]
#[message(event = WlKeyboard)]
pub struct RepeatInfo {
    pub rate: i32,
    pub delay: i32,
}

impl RepeatInfo {
    pub const SINCE: u32 = 4;
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum KeymapFormat {
    NoKeymap,
    XkbV1
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum KeyState {
    /// key is not pressed
    Released,
    /// key is pressed
    Pressed,
    /// key was repeated (since 10)
    Repeated,
}
