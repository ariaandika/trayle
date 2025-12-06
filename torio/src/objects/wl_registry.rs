use tcio::bytes::Buf;
use tcio::ByteStr;

use crate::objects::{Object, ObjectKind, ObjectManager};
use crate::objects::Message;

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
}

impl Object for Registry {
    const KIND: ObjectKind = ObjectKind::Registry;
}


// ===== Event =====

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

        // TODO: may panic if server sends invalid body len

        let name = u32::from_ne_bytes(*body.first_chunk::<4>().unwrap());
        let str_len = u32::from_ne_bytes(*body[4..].first_chunk::<4>().unwrap());
        let version = u32::from_ne_bytes(
            *body[super::roundup_4!(8usize + str_len as usize)..]
                .first_chunk::<4>()
                .unwrap(),
        );

        body.advance(8);
        body.truncate(str_len.strict_sub(1/*nulterm*/) as usize);
        let interface = ByteStr::from_utf8(body.freeze())?;

        Ok(Self {
            name,
            interface,
            version,
        })
    }
}

