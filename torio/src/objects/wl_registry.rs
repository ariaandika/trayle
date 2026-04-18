use tcio::bytes::ByteStr;

use crate::objects::{Message, ReadBuffer};
use crate::objects::{Object, ObjectKind, ObjectManager, Request, WriteBuffer};

pub const EVENT_GLOBAL_CODE: u16 = 0;

// ===== Object =====

#[derive(Debug)]
pub struct Registry {
    object_id: u32,
}

impl Registry {
    pub fn new_global_id() -> Self {
        Self::new(super::GlobalId::next())
    }

    pub fn new(object_id: u32) -> Self {
        Self { object_id }
    }

    pub fn with_manager(manager: &mut ObjectManager) -> Self {
        Self::new(manager.next_id(Self::KIND))
    }

    pub const fn object_id(&self) -> u32 {
        self.object_id
    }

    /// Binds a new, client-created object to the server using the specified name as the
    /// identifier.
    pub fn bind<'a>(&'a self, name: u32, interface: &'a str, version: u32, id: u32) -> Bind<'a> {
        Bind { registry: self, name, interface, version, id }
    }
}

impl Object for Registry {
    const KIND: ObjectKind = ObjectKind::Registry;
}

// ===== Bind =====

#[derive(Debug)]
pub struct Bind<'a> {
    registry: &'a Registry,
    name: u32,
    id: u32,
    interface: &'a str,
    version: u32,
}

impl Request for Bind<'_> {
    const OP_CODE: u16 = 0;

    fn object_id(&self) -> u32 {
        self.registry.object_id()
    }

    fn write_body(&self, buffer: &mut impl WriteBuffer) {
        buffer.put_uint(self.name);
        buffer.put_new_id(self.interface, self.version, self.id);
    }
}

// ===== Event =====

#[derive(Debug)]
pub enum Event {
    Global(GlobalEvent),
    GlobalRemove,
}

impl Event {
    pub fn from_message(message: Message) -> anyhow::Result<Self> {
        match message.opcode() {
            0 => Ok(Self::Global(GlobalEvent::from_message(message)?)),
            1 => Ok(Self::GlobalRemove),
            _ => Err(anyhow::anyhow!("unknown opcode")),
        }
    }
}

#[derive(Debug)]
pub struct GlobalEvent {
    pub name: u32,
    pub interface: ByteStr,
    pub version: u32,
}

impl GlobalEvent {
    fn from_message(message: Message) -> anyhow::Result<Self> {
        let mut body = message.into_body();
        Ok(Self {
            name: body.get_uint(),
            interface: ByteStr::from(body.get_string()),
            version: body.get_uint(),
        })
    }
}

