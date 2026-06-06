use crate::wayland::prelude::*;
use crate::wayland::wl_callback::WlCallback;
use crate::wayland::wl_registry::WlRegistry;

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Sync,
    GetRegistry,
}

#[derive(Debug)]
pub struct Sync {
    pub callback: NewId<WlCallback>,
}

#[derive(Debug)]
pub struct GetRegistry {
    pub registry: NewId<WlRegistry>,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Error,
    DeleteId,
}

/// `wl_display::error` event.
#[derive(Debug)]
pub struct Error<'a> {
    pub object_id: ObjectId,
    pub code: u32,
    pub message: &'a str,
}

#[derive(Debug)]
pub struct DeleteId {
    pub id: ObjectId,
}

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

// ===== constructor =====

impl Sync {
    #[inline]
    pub const fn new(callback: NewId<WlCallback>) -> Self {
        Self { callback }
    }
}

impl GetRegistry {
    #[inline]
    pub const fn new(registry: NewId<WlRegistry>) -> Self {
        Self { registry }
    }
}

impl DeleteId {
    #[inline]
    pub fn new<O: AsObjectId>(object: &O) -> Self {
        Self { id: object.object_id() }
    }
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

// ===== impls =====

// `wl_display` does not use derive macro, it has single unique implementation than other interfaces

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

impl Decode for Sync {
    type Output<'a> = Self;

    #[inline]
    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
        Ok(Self { callback: decoder.read()? })
    }
}

impl Decode for GetRegistry {
    type Output<'a> = Self;

    #[inline]
    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
        Ok(Self { registry: decoder.read()? })
    }
}

impl Decode for Error<'_> {
    type Output<'a> = Error<'a>;

    #[inline]
    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
        let mut reader = decoder.reader();
        Ok(Self::Output {
            object_id: reader.read()?,
            code: reader.read()?,
            message: reader.read()?,
        })
    }
}

impl Decode for DeleteId {
    type Output<'a> = Self;

    #[inline]
    fn decode<'a>(decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
        Ok(Self { id: decoder.read()? })
    }
}

impl Encode for Sync {
    const OPCODE: u16 = RequestOp::Sync as u16;

    #[inline]
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.callback);
    }
}

impl Encode for GetRegistry {
    const OPCODE: u16 = RequestOp::GetRegistry as u16;

    #[inline]
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.registry);
    }
}

impl Encode for Error<'_> {
    const OPCODE: u16 = EventOp::Error as u16;

    #[inline]
    fn encode(self, encoder: Encoder) {
        use super::encode::Write;
        let len = 16 + self.message.size();
        unsafe { encoder.encode(len) }
            .write(self.object_id)
            .write(self.code)
            .write(self.message);
    }
}

impl Encode for DeleteId {
    const OPCODE: u16 = EventOp::DeleteId as u16;

    #[inline]
    fn encode(self, encoder: Encoder) {
        encoder.encode1(self.id);
    }
}

