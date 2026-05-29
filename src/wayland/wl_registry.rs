use crate::wayland::prelude::*;

#[derive(Debug)]
pub struct Registry {
    id: Id,
}

impl FromId for Registry {
    #[inline]
    fn from_id(id: Id) -> Self {
        Self { id }
    }
}

impl Object for Registry {
    const INTERFACE_ID: InterfaceId = InterfaceId::WlRegistry;

    fn id(&self) -> Id {
        self.id
    }
}

impl Registry {
    /// Send `wl_registry::global` event.
    pub fn global<'a>(&self, name: u32, interface: &'a str, version: u32) -> Message<Global<'a>> {
        Message::new(self, Global { name, interface, version })
    }
}

// ===== Op =====

#[derive(Debug, Clone, Copy)]
pub enum RequestOp {
    Bind,
}

impl FromOp for RequestOp {
    fn from_op(op: u16) -> Result<Self, WlError> {
        if op == 0 {
            Ok(Self::Bind)
        } else {
            Err(WlError::UnknownOp)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EventOp {
    Global,
}

impl ToOp for EventOp {
    fn to_op(&self) -> u16 {
        *self as u16
    }
}

// ===== Bind =====

#[derive(Debug)]
pub struct Bind<'a> {
    pub name: u32,
    pub id_name: &'a str,
    pub id_version: u32,
    pub id: Id,
}

impl Decode for Bind<'static> {
    type Output<'a> = Bind<'a>;

    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
        let mut reader = decoder.body();
        Ok(Bind {
            name: reader.read()?,
            id_name: reader.read()?,
            id_version: reader.read()?,
            id: reader.read()?,
        })
    }
}

// ===== Global =====

pub struct Global<'a> {
    pub name: u32,
    pub interface: &'a str,
    pub version: u32,
}

impl Encode for Message<Global<'_>> {
    fn encode(self, encoder: Encoder) {
        let iface_len = self.interface.len() as u16;
        let len = const { 8 + 4 + 4 + 4 } + roundup4!(iface_len + 1);
        unsafe {
            encoder.encode(self.id(), EventOp::Global, len)
                .write(self.name)
                .write(self.interface)
                .write(self.version)
        };
    }
}

