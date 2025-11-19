use tcio::bytes::BytesMut;

use super::Request;

const WL_DISPLAY_OBJECT_ID: u32 = 1;
const GET_REGISTRY_OPCODE: u16 = 1;

#[derive(Debug)]
pub struct GetRegistry {
    wl_registry_id: u32,
}

impl GetRegistry {
    pub fn new() -> Self {
        Self {
            wl_registry_id: super::GlobalId::next(),
        }
    }
}

impl Request for GetRegistry {
    const OP_CODE: u16 = GET_REGISTRY_OPCODE;

    fn object_id(&self) -> u32 {
        WL_DISPLAY_OBJECT_ID
    }

    fn write_body(&self, buffer: &mut BytesMut) {
        buffer.extend_from_slice(&self.wl_registry_id.to_ne_bytes());
    }
}

impl Default for GetRegistry {
    fn default() -> Self {
        Self::new()
    }
}
