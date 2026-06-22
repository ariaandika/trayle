use crate::wayland::prelude::*;

#[derive(Interface, Debug)]
pub struct WlRegistry {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Bind,
}

#[derive(Message, Debug)]
#[message(request = WlRegistry)]
pub struct Bind<'a> {
    pub name: u32,
    pub new_id_name: &'a str,
    pub new_id_version: Version,
    pub new_id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Global,
    GlobalRemove,
}

#[derive(Message, Debug)]
#[message(event = WlRegistry)]
pub struct Global<'a> {
    pub name: u32,
    pub interface: &'a str,
    pub version: u32,
}

#[derive(Message, Debug)]
#[message(event = WlRegistry)]
pub struct GlobalRemove {
    pub name: u32,
}

// ===== BindError =====

#[derive(Debug, Clone, Copy)]
pub enum BindError {
    /// Unknown bind name.
    UnknownName,
    /// Missmatch bind name.
    MissmatchName,
    /// Unsupported bind version.
    UnsupportedVersion,
}

impl BindError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::UnknownName => "unknown bind name",
            Self::MissmatchName => "missmatch bind name",
            Self::UnsupportedVersion => "unsupported bind version"
        }
    }
}
