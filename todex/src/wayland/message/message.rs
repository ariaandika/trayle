use crate::wayland::primitives::{AsObjectId, AsVersion, ObjectId, Version};
use crate::wayland::object::{AsNewId, NewId};
use crate::wayland::wire::{AsOpCode, OpCode};
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
pub struct Message<T, M = (), D = ObjectId> {
    payload: T,
    marker: M,
    id: D,
}

impl<T, D> Message<T, (), D> {
    pub fn new(object_id: D, payload: T) -> Self {
        Self {
            payload,
            marker: (),
            id: object_id,
        }
    }
}

impl<T, M, D> Message<T, M, D> {
    #[inline]
    pub const fn payload(&self) -> &T {
        &self.payload
    }

    #[inline]
    pub fn into_payload(self) -> T {
        self.payload
    }
}

impl<T, D> AsVersion for Message<T, Version, D> {
    #[inline]
    fn version(&self) -> Version {
        self.marker
    }
}

impl<T, M, D> Message<T, M, D> {
    pub(crate) fn marker(&self) -> M
    where
        M: Copy,
    {
        self.marker
    }

    #[inline]
    pub fn from_parts(id: D, payload: T, marker: M) -> Self {
        Self {
            payload,
            marker,
            id,
        }
    }
}

impl<T, M: OpCode + Copy, D> Message<T, M, D> {
    #[inline]
    pub fn op(&self) -> M {
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

impl<T, M, D: AsObjectId> AsObjectId for Message<T, M, D> {
    #[inline]
    fn object_id(&self) -> ObjectId {
        self.id.object_id()
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
    const IS_REQUEST: bool = T::IS_REQUEST;

    const IS_EVENT: bool = T::IS_EVENT;

    const IS_DESTRUCTOR: bool = T::IS_DESTRUCTOR;

    const SINCE: Version = T::SINCE;
}
