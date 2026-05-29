use crate::wayland::prelude::*;

const GLOBAL_OP: u16 = 0;

#[derive(Debug)]
pub struct Registry {
    id: Id,
}

impl Object for Registry {
    const INTERFACE_ID: InterfaceId = InterfaceId::WlRegistry;

    fn id(&self) -> Id {
        self.id
    }
}

impl Registry {
    /// Can only be created by `GetRegistry`.
    pub(super) fn new(id: Id) -> Self {
        Self { id }
    }

    /// Send `wl_registry::global` event.
    pub fn global(&self, name: u32, interface: &str, version: u32, buffer: &mut Buffer) {
        let iface_len = interface.len() as u16;
        let len = const { 8 + 4 + 4 + 4 } + roundup4!(iface_len + 1);
        unsafe {
            buffer
                .message(self.id, GLOBAL_OP, len)
                .put(name)
                .put(interface)
                .put(version)
        };
    }
}

// ===== Op =====

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
