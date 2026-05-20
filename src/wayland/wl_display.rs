use crate::wayland::{Decode, Decoder, Id, PtrWrite, WlError, Write, roundup4, wl_callback, wl_registry::WlRegistry};

// ===== Op =====

pub enum Op {
    Sync(Decoder<Sync>),
    GetRegistry(Decoder<GetRegistry>),
}

impl Op {
    pub fn from_request(op: u16) -> Result<Op, WlError> {
        match op {
            0 => Ok(Op::Sync(Decoder::new())),
            1 => Ok(Op::GetRegistry(Decoder::new())),
            _ => Err(WlError::UnknownOp),
        }
    }
}

// ===== Sync =====

pub struct Sync {
    wl_callback_id: Id,
}

impl Sync {
    /// Write `wl_callback::done` and `wl_display::delete_id`  event.
    pub fn reply(self, callback_data: u32, mut writer: impl Write) {
        wl_callback::done(self.wl_callback_id, callback_data, &mut writer);
        delete_id(self.wl_callback_id, &mut writer);
    }
}

impl Decode for Sync {
    fn decode(body: &[u8]) -> Result<Self, WlError> {
        match body.as_array() {
            Some(ok) => Ok(Self {
                wl_callback_id: Id::from_ne_bytes(*ok)?,
            }),
            None => Err(WlError::InvalidSize),
        }
    }
}

// ===== GetRegistry =====

#[derive(Debug)]
pub struct GetRegistry {
    wl_registry_id: Id,
}

impl GetRegistry {
    pub fn wl_registry(&self) -> WlRegistry {
        WlRegistry::new(self.wl_registry_id)
    }
}

impl Decode for GetRegistry {
    fn decode(body: &[u8]) -> Result<Self, WlError> {
        match body.as_array() {
            Some(ok) => Ok(Self {
                wl_registry_id: Id::from_ne_bytes(*ok)?,
            }),
            None => Err(WlError::InvalidSize),
        }
    }
}

// ===== Encode =====

const ERROR_OP: u16 = 0;
const DELETE_ID_OP: u16 = 1;

/// Send `wl_display::error` event.
pub fn error(object_id: Id, code: u32, message: &str, mut writer: impl Write) {
    let msg_len = message.len() as u16;
    let len = const { 8 + 4 + 4 + 4 } + roundup4!(msg_len + 1);
    // SAFETY: initialization in `error_inner`
    unsafe {
        let ptr = writer.spare(len as u32);

        // object_id 1, opcode 0, len placeholder
        const HEADER: u64 = 1;
        ptr.cast::<u64>().write(HEADER);
        ptr.add(6).cast::<u16>().write(len);
        ptr.add(8).cast::<Id>().write(object_id);
        ptr.add(12).cast::<u32>().write(code);
        ptr.add(16).cast::<u32>().write((msg_len + 1) as u32);
        ptr.add(20).copy_from_nonoverlapping(message.as_ptr(), msg_len as usize);
        ptr.add((20 + msg_len) as usize).write(0);
    }
}

/// Send `wl_display::delete_id` event.
pub fn delete_id(id: Id, mut writer: impl Write) {
    const LEN: u16 = const { 8 + 4 };
    // SAFETY: initialization in `error_inner`
    unsafe {
        let ptr = writer.spare(LEN as u32);
        ptr.put(1u32);
        ptr.add(4).put(DELETE_ID_OP);
        ptr.add(6).put(LEN);
        ptr.add(8).put(id);
    }
}
