use crate::wayland::prelude::*;
use crate::wayland::wl_callback::WlCallback;
use crate::wayland::wl_registry::WlRegistry;

// ===== Op =====

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Sync,
    GetRegistry,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Error,
    DeleteId,
}

// ===== Sync =====

/// `wl_display::sync` request.
#[derive(Debug)]
pub struct Sync {
    pub callback: NewId<WlCallback>,
}

impl Sync {
    /// Create `wl_display::sync` request.
    #[inline]
    pub const fn new(callback: NewId<WlCallback>) -> Self {
        Self { callback }
    }
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
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.callback);
    }
}

// ===== GetRegistry =====

/// `wl_display::get_registry` request.
#[derive(Debug)]
pub struct GetRegistry {
    pub registry: NewId<WlRegistry>,
}

impl GetRegistry {
    /// Create `wl_display::get_registry` request.
    #[inline]
    pub const fn new(registry: NewId<WlRegistry>) -> Self {
        Self { registry }
    }
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
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.registry);
    }
}

// ===== Error =====

/// `wl_display::error` enum.
#[derive(Debug, Clone, Copy)]
pub enum WlDisplayError {
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

/// `wl_display::error` event.
#[derive(Debug)]
pub struct Error<'a> {
    pub object_id: ObjectId,
    pub code: u32,
    pub message: &'a str,
}

impl<'a> Error<'a> {
    /// Create `wl_display::error` event.
    #[inline]
    pub const fn new(object_id: ObjectId, code: u32, message: &'a str) -> Self {
        Self { object_id, code, message }
    }

    /// Create `wl_display::error` event from [`WlError`].
    #[inline]
    pub fn from_wl_error(id: ObjectId, error: WlError) -> Error<'static> {
        use WlError as E;

        const MALFORMED: (ObjectId, WlDisplayError) = (ObjectId::wl_display(), WlDisplayError::InvalidMethod);
        const SEMANTIC: (ObjectId, WlDisplayError) = (ObjectId::wl_display(), WlDisplayError::InvalidObject);

        // in the future if there is id specific error.
        let _ = id;
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
            E::NotYetImplemented => (ObjectId::wl_display(), WlDisplayError::Implementation),
        };
        Error {
            object_id,
            code: code as u32,
            message: error.message(),
        }
    }
}

impl Decode for Error<'_> {
    type Output<'a> = Error<'a>;

    #[inline]
    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
        let mut reader = decoder.reader();
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
    fn encode(self, encoder: Encoder) {
        encode_me!(encoder, self, object_id, code, message);
    }
}

// ===== DeleteId =====

/// `wl_display::delete_id` event.
pub struct DeleteId {
    id: ObjectId,
}

impl DeleteId {
    /// Create `wl_display::delete_id` event.
    #[inline]
    pub fn new<O: AsObjectId>(object: &O) -> Self {
        Self { id: object.object_id() }
    }
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
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.id);
    }
}

macro_rules! msg {
    ($($name:ident$(<$l:lifetime>)?),*) => {$(
        impl AsInterface for $name$(<$l>)? {
            const INTERFACE: Interface = Interface::WlDisplay;
        }

        impl AsObjectId for $name$(<$l>)? {
            #[inline]
            fn object_id(&self) -> ObjectId {
                ObjectId::wl_display()
            }
        }
    )*};
}

msg!(Sync, GetRegistry, Error<'_>, DeleteId);
