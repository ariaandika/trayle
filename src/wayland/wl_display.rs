use crate::wayland::{Decode, Decoder, Id, WlError, Write, roundup4, wl_callback};

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

pub struct Sync {
    wl_callback_id: u32,
}

impl Sync {
    /// Write `callback::done` event.
    pub fn encode_callback(self, callback_data: u32, writer: impl Write) {
        wl_callback::done(self.wl_callback_id, callback_data, writer);
    }
}

#[derive(Debug)]
pub struct GetRegistry {
    wl_registry_id: u32,
}

// ===== Encode =====

/// Send `wl_display::error` event.
pub fn error(object_id: Id, code: u32, message: &str, mut writer: impl Write) {
    let msg_len = message.len() as u16;
    let len = 20 + roundup4!(msg_len + 1);
    // SAFETY: initialization in `error_inner`
    let ptr = unsafe { writer.spare(len as u32) };
    error_inner(object_id, code, message.as_ptr(), msg_len, len, ptr);
}

fn error_inner(object_id: Id, code: u32, msg: *const u8, msg_len: u16, len: u16, ptr: *mut u8) {
    // object_id 1, opcode 0, len placeholder
    const HEADER: u64 = 1;
    unsafe {
        ptr.cast::<u64>().write(HEADER);
        ptr.add(6).cast::<u16>().write(len);
        ptr.add(8).cast::<Id>().write(object_id);
        ptr.add(12).cast::<u32>().write(code);
        ptr.add(16).cast::<u32>().write((msg_len + 1) as u32);
        ptr.add(20).copy_from_nonoverlapping(msg, msg_len as usize);
        ptr.add((20 + msg_len) as usize).write(0);
    }
}

// ===== Decode =====

impl Decode for Sync {
    fn decode(body: &[u8]) -> Result<Self, WlError> {
        match body.as_array() {
            Some(ok) => Ok(Self {
                wl_callback_id: u32::from_ne_bytes(*ok),
            }),
            None => Err(WlError::InvalidSize),
        }
    }
}

impl Decode for GetRegistry {
    fn decode(body: &[u8]) -> Result<Self, WlError> {
        match body.as_array() {
            Some(ok) => Ok(Self {
                wl_registry_id: u32::from_ne_bytes(*ok),
            }),
            None => Err(WlError::InvalidSize),
        }
    }
}
