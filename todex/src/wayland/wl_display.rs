use crate::wayland::prelude::*;
use crate::wayland::wl_callback::WlCallback;
use crate::wayland::wl_registry::WlRegistry;

#[derive(Debug, Clone, Copy)]
pub struct WlDisplay;

impl AsInterface for WlDisplay {
    #[inline]
    fn interface(&self) -> Interface {
        Interface::WlDisplay
    }
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum RequestOp {
    Sync,
    GetRegistry,
}

#[derive(Message, Debug)]
#[request(WlDisplay)]
pub struct Sync {
    pub callback: NewId<WlCallback>,
}

#[derive(Message, Debug)]
#[request(WlDisplay)]
pub struct GetRegistry {
    pub registry: NewId<WlRegistry>,
}

#[derive(OpCode, Debug, Clone, Copy)]
pub enum EventOp {
    Error,
    DeleteId,
}

#[derive(Message, Debug)]
#[event(WlDisplay)]
pub struct Error<'a> {
    pub object_id: ObjectId,
    pub code: u32,
    pub message: &'a str,
}

#[derive(Message, Debug)]
#[event(WlDisplay)]
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
    pub fn new<O: AsObjectId>(object: O) -> Self {
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
            E::UnknownEnumEntry => SEMANTIC,
            E::InvalidObject => SEMANTIC,
            E::InvalidSize => MALFORMED,
            E::InvalidNewId => SEMANTIC,
            E::ZeroId => SEMANTIC,
            E::Null => SEMANTIC,
            E::NonUtf8 => SEMANTIC,
            E::MissingFd => MALFORMED,
            E::NotYetImplemented => (ObjectId::wl_display(), WlDisplayError::Implementation),
            _ => todo!(),
        };
        Error {
            object_id,
            code: code as u32,
            message: error.message(),
        }
    }
}

// ===== impls =====

macro_rules! static_id {
    ($($name:ty),*) => {$(
        impl AsObjectId for $name {
            #[inline]
            fn object_id(&self) -> ObjectId {
                ObjectId::wl_display()
            }
        }
    )*};
}
static_id!(WlDisplay, Sync, GetRegistry, Error<'_>, DeleteId);
