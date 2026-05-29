use crate::wayland::prelude::*;

simple_object! {
    pub struct WlRegistry::Registry;
}

impl Registry {
    /// Send `wl_registry::global` event.
    pub fn global<'a>(&self, name: u32, interface: &'a str, version: u32) -> Message<Global<'a>> {
        Message::new(self, Global { name, interface, version })
    }
}

// ===== Op =====

opcode! {
    pub enum RequestOp {
        Bind,
    }
}

opcode! {
    pub enum EventOp {
        Global,
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

impl<'a> Bind<'a> {
    /// Create object from current bind id.
    ///
    /// Note that this does not check for interface correction.
    pub fn get<O: FromId>(self) -> O {
        O::from_id(self.id)
    }
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

