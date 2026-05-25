use crate::wayland::prelude::*;
use crate::wayland::wl_registry::WlRegistry;

// ===== Op =====

const ERROR_OP: u16 = 0;
const DELETE_ID_OP: u16 = 1;
const DONE_OP: u16 = 0;

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
    pub fn wl_callback_id(&self) -> Id {
        self.wl_callback_id
    }

    /// Write `wl_callback::done` and `wl_display::delete_id`  event.
    pub fn reply(self, callback_data: u32, buffer: &mut Buffer) {
        unsafe {
            // wl_callback::done(callback_data: uint)
            buffer
                .message(self.wl_callback_id, DONE_OP, 12)
                .put(callback_data);
            // wl_display::delete_id(id: uint)
            buffer
                .message(Id::wl_display(), DELETE_ID_OP, 12)
                .put(self.wl_callback_id);
        };
    }
}

impl Decode for Sync {
    type Output<'a> = Self;

    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        Ok(Self {
            wl_callback_id: reader.read()?,
        })
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
    type Output<'a> = Self;

    fn decode(reader: &mut Reader) -> Result<Self, WlError> {
        Ok(Self {
            wl_registry_id: reader.read()?,
        })
    }
}

// ===== Encode =====

/// server couldn't find object
const INVALID_OBJECT: u32 = 0;
/// method doesn't exist on the specified interface or malformed request
const INVALID_METHOD: u32 = 1;
// /// server is out of memory
// const NO_MEMORY: u32 = 2;
/// implementation error in compositor
const IMPLEMENTATION: u32 = 3;

/// Send `wl_display::error` event from `WlError`.
pub fn encode_error(_: Id, error: WlError, buffer: &mut Buffer) {
    use WlError as E;

    const MALFORMED: (Id, u32) = (Id::wl_display(), INVALID_METHOD);
    const SEMANTIC: (Id, u32) = (Id::wl_display(), INVALID_OBJECT);

    let (id, code) = match error {
        E::UnknownOp => MALFORMED,
        E::UnknownObject => SEMANTIC,
        E::UnknownBind => SEMANTIC,
        E::InvalidSize => MALFORMED,
        E::ExcessiveSize => MALFORMED,
        E::InvalidNewId => SEMANTIC,
        E::ZeroId => SEMANTIC,
        E::Null => SEMANTIC,
        E::MissingFd => MALFORMED,
        E::Internal => (Id::wl_display(), IMPLEMENTATION),
    };

    let message = error.message();
    let msg_len = message.len() as u16;
    let len = const { 8 + 4 + 4 + 4 } + roundup4!(msg_len + 1);

    unsafe {
        buffer
            .message(Id::wl_display(), ERROR_OP, len)
            .put(id)
            .put(code)
            .put(message)
    };
}
