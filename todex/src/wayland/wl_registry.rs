use crate::wayland::prelude::*;

#[derive(Interface, Debug)]
#[interface(data = Version)]
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
    pub id_name: &'a str,
    pub id_version: Version,
    pub id: ObjectId,
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

// ===== impls =====

impl<'a> Bind<'a> {
    /// Create object from current bind id.
    ///
    /// Note that this does not check for matching interface.
    pub fn create<O: FromObjectId>(self) -> O {
        O::from_object_id(self.id)
    }

    /// Create [`BindData`] from current bind request.
    #[inline]
    pub fn data(&self, interface: Interface) -> BindData {
        BindData {
            interface,
            version: self.id_version,
        }
    }
}

// ===== BindData =====

#[derive(Debug)]
pub struct BindData {
    pub interface: Interface,
    pub version: Version,
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
