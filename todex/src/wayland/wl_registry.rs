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
