use crate::wayland::primitives::{AsObjectId, AsVersion, ObjectId, Version};
use crate::wayland::object::{AsNewId, NewId, Handle};
use crate::wayland::message::AsOpCode;
use crate::wayland::interface::{AsInterface, Interface, WlInterface};

// ===== trait =====

/// Type that represent wayland message.
pub trait WlMessage: AsInterface + AsOpCode {
    type WlInterface: WlInterface;

    const IS_REQUEST: bool;

    const IS_EVENT: bool = !Self::IS_REQUEST;

    const IS_DESTRUCTOR: bool = false;

    const SINCE: Version = Version::new(1).unwrap();
}

// ===== message =====

/// Wayland message.
///
/// This is detailed documentation about the `Message` struct, for practical usage see the message
/// module [documentation][mdoc].
///
/// [mdoc]: crate::wayland::message
///
/// This struct accept 1 required and 2 optional generic parameter.
///
/// The first generic should always represent message payload, whether in raw bytes or typed
/// payload. If its typed payload, `Message` delegate message related trait implementation to it,
/// like `AsNewId`, `AsInterface`, `AsOpCode`, and `WlMessage`.
///
/// The second optional generic can represent any metadata. The default is `()`. Common usage is to
/// store [`Version`], as `Message` will implement [`AsVersion`], thus can be used when creating new
/// object that intherent this version.
///
/// The third optional generic usually represent an id or index. The default is [`ObjectId`]. Other
/// usage is to store [`Handle`] that is occasionally used inside compositor logic.
#[derive(Debug)]
pub struct Message<T, M = (), D = ObjectId> {
    payload: T,
    meta: M,
    id: D,
}

impl<T, D> Message<T, (), D> {
    /// Create `Message` that contains object id and message payload.
    #[inline]
    pub fn new(object_id: D, payload: T) -> Self {
        Self::from_parts(object_id, payload, ())
    }
}

impl<T, M, D> Message<T, M, D> {
    /// Create `Message` from parts.
    #[inline]
    pub fn from_parts(id: D, payload: T, meta: M) -> Self {
        Self { payload, meta, id }
    }

    /// Returns the second generic parameter value.
    #[inline]
    pub fn meta(&self) -> M
    where
        M: Copy,
    {
        self.meta
    }

    /// Returns reference of the message payload.
    #[inline]
    pub const fn payload(&self) -> &T {
        &self.payload
    }

    /// Consume the message anr returns the payload.
    #[inline]
    pub fn into_payload(self) -> T {
        self.payload
    }

    /// Drop the second generic parameter value and replace it with [`Version`].
    #[inline]
    pub fn with_version(self, version: Version) -> Message<T, Version, D> {
        Message::from_parts(self.id, self.payload, version)
    }

    /// Drop the third generic parameter value and replace it with [`Handle`].
    #[inline]
    pub fn with_handle(self, handle: Handle) -> Message<T, M, Handle> {
        Message::from_parts(handle, self.payload, self.meta)
    }
}

impl<T, M, D: AsObjectId> AsObjectId for Message<T, M, D> {
    #[inline]
    fn object_id(&self) -> ObjectId {
        self.id.object_id()
    }
}

impl<T, M, D> std::ops::Deref for Message<T, M, D> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl<T: AsNewId, M, D> AsNewId for Message<T, M, D> {
    type Interface = T::Interface;

    #[inline]
    fn new_id(&self) -> NewId<Self::Interface> {
        self.payload.new_id()
    }
}

impl<T: AsInterface, M, D> AsInterface for Message<T, M, D> {
    #[inline]
    fn interface(&self) -> Interface {
        self.payload.interface()
    }
}

impl<T: AsOpCode, M, D> AsOpCode for Message<T, M, D> {
    type OpCode = T::OpCode;

    const OPCODE: Self::OpCode = T::OPCODE;

    const OPNAME: &str = T::OPNAME;
}

impl<T: WlMessage, M, D> WlMessage for Message<T, M, D> {
    type WlInterface = T::WlInterface;

    const IS_REQUEST: bool = T::IS_REQUEST;

    const IS_EVENT: bool = T::IS_EVENT;

    const IS_DESTRUCTOR: bool = T::IS_DESTRUCTOR;

    const SINCE: Version = T::SINCE;
}

impl<T, D> AsVersion for Message<T, Version, D> {
    #[inline]
    fn version(&self) -> Version {
        self.meta
    }
}

impl<T, M> Message<T, M, Handle> {
    /// Returns the associated [`Handle`].
    #[inline]
    pub fn handle(&self) -> Handle {
        self.id
    }
}
