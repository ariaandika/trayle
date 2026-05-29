use crate::wayland::prelude::*;
use crate::wayland::wl_callback::WlCallback;
use crate::wayland::wl_registry::Registry;

// ===== Op =====

#[derive(Debug, Clone, Copy)]
pub enum RequestOp {
    Sync,
    GetRegistry,
}

impl FromOp for RequestOp {
    fn from_op(op: u16) -> Result<Self, WlError> {
        match op {
            0 => Ok(Self::Sync),
            1 => Ok(Self::GetRegistry),
            _ => Err(WlError::UnknownOp),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EventOp {
    Error,
    DeleteId,
}

impl ToOp for EventOp {
    fn to_op(&self) -> u16 {
        *self as u16
    }
}

// ===== Sync =====

#[derive(Debug)]
pub struct Sync {
    pub callback: NewId<WlCallback>,
}

impl Decode for Sync {
    type Output<'a> = Self;

    fn decode(decoder: Decoder) -> Result<Self, WlError> {
        Ok(Self {
            callback: decoder.read()?,
        })
    }
}

// ===== GetRegistry =====

#[derive(Debug)]
pub struct GetRegistry {
    pub registry: NewId<Registry>,
}

impl Decode for GetRegistry {
    type Output<'a> = Self;

    fn decode(decoder: Decoder) -> Result<Self, WlError> {
        Ok(Self {
            registry: decoder.read()?,
        })
    }
}

// ===== Error =====

pub enum DisplayError {
    /// server couldn't find object
    InvalidObject,
    /// method doesn't exist on the specified interface or malformed request
    InvalidMethod,
    /// server is out of memory
    #[allow(unused)]
    NoMemory,
    /// implementation error in compositor
    Implementation,
}

pub fn error_from(id: Id, error: WlError) -> Error<'static> {
    use WlError as E;

    const MALFORMED: (Id, DisplayError) = (Id::wl_display(), DisplayError::InvalidMethod);
    const SEMANTIC: (Id, DisplayError) = (Id::wl_display(), DisplayError::InvalidObject);

    let (object_id, code) = match error {
        E::UnknownOp => MALFORMED,
        E::UnknownObject => SEMANTIC,
        E::UnknownBind => SEMANTIC,
        E::InvalidSize => MALFORMED,
        E::InvalidNewId => SEMANTIC,
        E::ZeroId => SEMANTIC,
        E::Null => SEMANTIC,
        E::NonUtf8 => SEMANTIC,
        E::MissingFd => MALFORMED,
        E::NotYetImplemented => (Id::wl_display(), DisplayError::Implementation),
    };
    let _ = id;
    Error {
        object_id,
        code: code as u32,
        message: error.message(),
    }
}

pub struct Error<'a> {
    object_id: Id,
    code: u32,
    message: &'a str,
}

impl Encode for Error<'_> {
    fn encode(self, encoder: Encoder) {
        let msg_len = self.message.len() as u16;
        let len = const { 8 + 4 + 4 + 4 } + roundup4!(msg_len + 1);
        unsafe {
            encoder
                .encode(Id::wl_display(), EventOp::Error, len)
                .write(self.object_id)
                .write(self.code)
                .write(self.message)
        };
    }
}

// ===== DeleteId =====

pub fn delete_id<O: Object>(object: &O) -> DeleteId {
    DeleteId { id: object.id() }
}

pub struct DeleteId {
    id: Id,
}

impl Encode for DeleteId {
    fn encode(self, encoder: Encoder) {
        encoder.encode_one(Id::wl_display(), EventOp::DeleteId, self.id);
    }
}
