use crate::wayland::prelude::*;
use crate::wayland::wl_callback::WlCallback;
use crate::wayland::wl_registry::WlRegistry;

// ===== Op =====

opcode! {
    pub enum RequestOp {
        Sync,
        GetRegistry,
    }
}

opcode! {
    pub enum EventOp {
        Error,
        DeleteId,
    }
}

// ===== Sync =====

#[derive(Debug)]
pub struct Sync {
    pub callback: NewId<WlCallback>,
}

impl Decode for Sync {
    type Output<'a> = Self;

    #[inline]
    fn decode(decoder: Decoder) -> Result<Self, WlError> {
        Ok(Self {
            callback: decoder.read()?,
        })
    }
}

impl Encode for Sync {
    const OPCODE: u16 = RequestOp::Sync as u16;

    #[inline]
    fn object_id(&self) -> Id {
        Id::wl_display()
    }

    #[inline]
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.callback);
    }
}

// ===== GetRegistry =====

#[derive(Debug)]
pub struct GetRegistry {
    pub registry: NewId<WlRegistry>,
}

impl Decode for GetRegistry {
    type Output<'a> = Self;

    #[inline]
    fn decode(decoder: Decoder) -> Result<Self, WlError> {
        Ok(Self {
            registry: decoder.read()?,
        })
    }
}

impl Encode for GetRegistry {
    const OPCODE: u16 = RequestOp::GetRegistry as u16;

    #[inline]
    fn object_id(&self) -> Id {
        Id::wl_display()
    }

    #[inline]
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.registry);
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

impl Decode for Error<'static> {
    type Output<'a> = Error<'a>;

    #[inline]
    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
        let mut reader = decoder.body();
        Ok(Error {
            object_id: reader.read()?,
            code: reader.read()?,
            message: reader.read()?,
        })
    }
}

impl Encode for Error<'_> {
    const OPCODE: u16 = EventOp::Error as u16;

    #[inline]
    fn object_id(&self) -> Id {
        Id::wl_display()
    }

    #[inline]
    fn encode(self, encoder: Encoder) {
        encode_me!(encoder, self, object_id, code, message);
    }
}

// ===== DeleteId =====

#[inline]
pub fn delete_id<O: WaylandObject>(object: &O) -> DeleteId {
    DeleteId { id: object.id() }
}

pub struct DeleteId {
    id: Id,
}

impl Decode for DeleteId {
    type Output<'a> = Self;

    #[inline]
    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
        Ok(Self {
            id: decoder.read()?,
        })
    }
}

impl Encode for DeleteId {
    const OPCODE: u16 = EventOp::DeleteId as u16;

    #[inline]
    fn object_id(&self) -> Id {
        Id::wl_display()
    }

    #[inline]
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.id);
    }
}
