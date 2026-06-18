use crate::wayland::prelude::*;
use crate::wayland::wl_data_device_manager::DndAction;

#[derive(Interface, Debug)]
pub struct WlDataOffer {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Accept,
    Receive,
    Destroy,
    Finish,
    SetActions,
}

#[derive(Message, Debug)]
#[request(WlDataOffer)]
pub struct Accept<'a> {
    pub serial: u32,
    pub mime_type: Option<&'a str>,
}

#[derive(Message, Debug)]
#[request(WlDataOffer)]
pub struct Receive<'a> {
    pub mime_type: &'a str,
    #[fd]
    pub fd: i32,
}

#[derive(Message, Debug)]
#[request(WlDataOffer)]
#[destructor]
pub struct Destroy;

#[derive(Message, Debug)]
#[request(WlDataOffer)]
pub struct Finish;

#[derive(Message, Debug)]
#[request(WlDataOffer)]
pub struct SetActions {
    pub dnd_actions: DndAction,
    pub preferred_action: DndAction,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Offer,
    SourceActions,
    Action,
}

#[derive(Message, Debug)]
#[event(WlDataOffer)]
pub struct Offer<'a> {
    pub mime_type: &'a str,
}

#[derive(Message, Debug)]
#[event(WlDataOffer)]
pub struct SourceActions {
    pub source_actions: DndAction,
}

#[derive(Message, Debug)]
#[event(WlDataOffer)]
pub struct Action {
    pub dnd_action: DndAction,
}

#[derive(WlEnum, Debug, Clone, Copy)]
pub enum Error {
    /// finish request was called untimely
    InvalidFinish,
    /// action mask contains invalid values
    InvalidActionMask,
    /// action argument has an invalid value
    InvalidAction,
    /// offer doesn't accept this request
    InvalidOffer,
}
