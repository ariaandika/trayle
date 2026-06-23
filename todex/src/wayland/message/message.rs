use crate::wayland::message::Constructor;
use crate::wayland::primitives::{AsNewId, AsObjectId, NewId, ObjectId, Version};
use crate::wayland::wire::AsOpCode;
use crate::wayland::{AsInterface, Interface};

// ===== trait =====

/// Type that represent wayland message.
pub trait WlMessage: AsInterface + AsOpCode {
    const IS_REQUEST: bool;

    const IS_EVENT: bool = !Self::IS_REQUEST;

    const IS_DESTRUCTOR: bool = false;

    const SINCE: Version = Version::new(1).unwrap();
}

// ===== message =====

/// Associate object id with a message.
#[derive(Debug)]
pub struct Message<T, M = ()> {
    payload: T,
    object_id: ObjectId,
    marker: M,
}

impl<T> Message<T> {
    /// Create new [`Message`].
    pub fn new<O: AsObjectId>(object: &O, payload: T) -> Self {
        Self {
            payload,
            marker: (),
            object_id: object.object_id(),
        }
    }
}

impl<T, M> Message<T, M> {
    #[inline]
    pub const fn payload(&self) -> &T {
        &self.payload
    }

    #[inline]
    pub fn into_payload(self) -> T {
        self.payload
    }
}

impl<T> Message<T, Version> {
    /// Create new [`Message`].
    #[inline]
    pub fn new_versioned(object_id: ObjectId, payload: T, version: Version) -> Self {
        Self {
            payload,
            marker: version,
            object_id,
        }
    }

    #[inline]
    pub fn version(&self) -> Version {
        self.marker
    }
}

impl<T, M> std::ops::Deref for Message<T, M> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl<T, M> AsObjectId for Message<T, M> {
    #[inline]
    fn object_id(&self) -> ObjectId {
        self.object_id
    }
}

impl<T: AsNewId, M> AsNewId for Message<T, M> {
    type Interface = T::Interface;

    #[inline]
    fn new_id(&self) -> NewId<Self::Interface> {
        self.payload.new_id()
    }
}

impl<T: AsInterface, M> AsInterface for Message<T, M> {
    #[inline]
    fn interface(&self) -> Interface {
        self.payload.interface()
    }
}

impl<T: AsOpCode, M> AsOpCode for Message<T, M> {
    type OpCode = T::OpCode;

    const OPCODE: Self::OpCode = T::OPCODE;

    const OPNAME: &str = T::OPNAME;
}

impl<T: WlMessage, M> WlMessage for Message<T, M> {
    const IS_REQUEST: bool = T::IS_REQUEST;

    const IS_EVENT: bool = T::IS_EVENT;

    const IS_DESTRUCTOR: bool = T::IS_DESTRUCTOR;

    const SINCE: Version = T::SINCE;
}

impl<T: AsNewId> Constructor for Message<T, Version> {
    type Interface = T::Interface;

    #[inline]
    fn new_version(&self) -> Version {
        self.marker
    }

    #[inline]
    fn new_id(&self) -> NewId<Self::Interface> {
        T::new_id(&self.payload)
    }
}
