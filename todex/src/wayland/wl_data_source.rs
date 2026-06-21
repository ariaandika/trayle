use crate::wayland::prelude::*;
use crate::wayland::wl_data_device_manager::DndAction;

#[derive(Debug, Interface)]
pub struct WlDataSource {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Offer,
    Destroy,
    SetActions,
}

#[derive(Message, Debug)]
#[message(request = WlDataSource)]
pub struct Offer<'a> {
    pub mime_type: &'a str,
}

#[derive(Message, Debug)]
#[message(request = WlDataSource, destructor)]
pub struct Destroy;

#[derive(Message, Debug)]
#[message(request = WlDataSource)]
pub struct SetActions {
    pub dnd_actions: DndAction,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Target,
    Send,
    Cancelled,
    DndDropPerformed,
    DndFinished,
    Action,
}

#[derive(Message, Debug)]
#[message(event = WlDataSource)]
pub struct Target<'a> {
    pub mime_type: Option<&'a str>,
}

#[derive(Message, Debug)]
#[message(event = WlDataSource)]
pub struct Send<'a> {
    pub mime_type: &'a str,
    #[fd]
    pub fd: i32,
}

#[derive(Message, Debug)]
#[message(event = WlDataSource)]
pub struct Cancelled;

#[derive(Message, Debug)]
#[message(event = WlDataSource)]
pub struct DndDropPerformed;

#[derive(Message, Debug)]
#[message(event = WlDataSource)]
pub struct DndFinished;

#[derive(Message, Debug)]
#[message(event = WlDataSource)]
pub struct Action {
    pub dnd_action: DndAction,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    /// action mask contains invalid values
    InvalidActionMask,
    /// source doesn't accept this request
    InvalidSource,
}
