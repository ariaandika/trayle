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
}

#[derive(Message, Debug)]
#[event(WlRegistry)]
pub struct Global<'a> {
    pub name: u32,
    pub interface: &'a str,
    pub version: u32,
}

// ===== impls =====

impl WlRegistry {
    #[inline]
    pub fn bind<'a>(
        &self,
        name: u32,
        id_name: &'a str,
        id_version: u32,
        id: ObjectId,
    ) -> Message<Bind<'a>> {
        Message::new(
            self,
            Bind {
                name,
                id_name,
                id_version,
                id,
            },
        )
    }

    #[inline]
    pub fn global<'a>(&self, name: u32, interface: &'a str, version: u32) -> Message<Global<'a>> {
        Message::new(
            self,
            Global {
                name,
                interface,
                version,
            },
        )
    }
}

impl<'a> Bind<'a> {
    /// Create object from current bind id.
    ///
    /// Note that this does not check for matching interface.
    #[inline]
    pub fn create<O: FromObjectId>(self) -> O {
        O::from_object_id(self.id)
    }
}
