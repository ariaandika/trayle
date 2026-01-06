use crate::objects::wl_registry::Registry;
use crate::objects::{WriteBuffer, Request};

// `wl_display` properties
pub const OBJECT_ID: u32 = 1;
const GET_REGISTRY_OPCODE: u16 = 1;

// ===== wl_display =====

/// `wl_display` object.
#[derive(Debug)]
#[non_exhaustive]
pub struct Display {

}

impl Display {
    pub fn new() -> Self {
        Self {  }
    }

    pub fn get_registry<'a>(&self, registry: &'a Registry) -> GetRegistry<'a> {
        GetRegistry { registry }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

// ===== wl_display::get_registry =====

#[derive(Debug)]
pub struct GetRegistry<'a> {
    registry: &'a Registry,
}

impl Request for GetRegistry<'_> {
    const OP_CODE: u16 = GET_REGISTRY_OPCODE;

    fn object_id(&self) -> u32 {
        OBJECT_ID
    }

    fn write_body(&self, buffer: &mut impl WriteBuffer) {
        buffer.put_uint(self.registry.object_id());
    }
}

