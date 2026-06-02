use crate::wayland::prelude::*;

simple_object! {
    pub struct WlRegistry;
}

impl WlRegistry {
    /// Send `wl_registry::global` event.
    #[inline]
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
    pub id: ObjectId,
}

impl<'a> Bind<'a> {
    /// Create object from current bind id.
    ///
    /// Note that this does not check for interface correction.
    #[inline]
    pub fn get<O: FromObjectId>(self) -> O {
        O::from_id(self.id)
    }
}

impl Decode for Bind<'static> {
    type Output<'a> = Bind<'a>;

    #[inline]
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

impl Encode for Message<Bind<'_>> {
    const OPCODE: u16 = RequestOp::Bind as u16;

    #[inline]
    fn object_id(&self) -> ObjectId {
        self.id()
    }

    #[inline]
    fn encode(self, encoder: Encoder) {
        encode_me!(encoder, self, name, id_name, id_version, id);
    }
}

// ===== Global =====

pub struct Global<'a> {
    pub name: u32,
    pub interface: &'a str,
    pub version: u32,
}

impl Decode for Global<'static> {
    type Output<'a> = Global<'a>;

    #[inline]
    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
        let mut reader = decoder.body();
        Ok(Global {
            name: reader.read()?,
            interface: reader.read()?,
            version: reader.read()?,
        })
    }
}

impl Encode for Message<Global<'_>> {
    const OPCODE: u16 = EventOp::Global as u16;

    #[inline]
    fn object_id(&self) -> ObjectId {
        self.id()
    }

    #[inline]
    fn encode(self, encoder: Encoder) {
        encode_me!(encoder, self, name, interface, version);
    }
}
