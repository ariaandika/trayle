use crate::wayland::prelude::*;

#[derive(Interface, Debug)]
#[data(BindVersion)]
pub struct WlRegistry {
    id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Bind,
}

#[derive(Message, Debug)]
#[request(WlRegistry)]
pub struct Bind<'a> {
    pub name: u32,
    pub id_name: &'a str,
    pub id_version: u32,
    pub id: ObjectId,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Global,
    GlobalRemove,
}

#[derive(Message, Debug)]
#[event(WlRegistry)]
pub struct Global<'a> {
    pub name: u32,
    pub interface: &'a str,
    pub version: u32,
}

#[derive(Message, Debug)]
#[event(WlRegistry)]
pub struct GlobalRemove {
    pub name: u32,
}

// ===== impls =====

impl<'a> Bind<'a> {
    /// Create object from current bind id.
    ///
    /// Note that this does not check for matching interface.
    #[inline]
    pub fn create<O: FromObjectId>(self) -> O {
        O::from_object_id(self.id)
    }
}

// ===== Data =====

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct BindVersion(std::num::NonZeroU32);

impl ObjectData for BindVersion {
    #[inline]
    fn from_raw(raw: usize) -> Self {
        Self(
            std::num::NonZeroU32::new(raw as u32).expect("internal error: raw object data mutated"),
        )
    }

    #[inline]
    fn to_raw(self) -> usize {
        self.0.get() as usize
    }
}

impl std::cmp::PartialEq<u32> for BindVersion {
    fn eq(&self, other: &u32) -> bool {
        self.0.get().eq(other)
    }
}

impl std::cmp::PartialOrd<u32> for BindVersion {
    fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> {
        self.0.get().partial_cmp(other)
    }
}

impl std::fmt::Display for BindVersion {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
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
