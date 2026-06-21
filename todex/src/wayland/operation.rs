use crate::wayland::{AsInterface, AsNewId, AsObjectId, AsOpCode, WlGlobal, WlMessage};
use crate::wayland::{Constructor, Interface, NewId, ObjectId, Version};

#[derive(Debug, Clone, Copy)]
pub struct Operation<M> {
    object_id: ObjectId,
    message: M,
    version: Version,
}

impl<M> Operation<M> {
    #[inline]
    pub fn new(object_id: ObjectId, message: M, version: Version) -> Self {
        Self { object_id, message, version }
    }

    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }

    #[inline]
    pub fn into_message(self) -> M {
        self.message
    }
}

impl<M> std::ops::Deref for Operation<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.message
    }
}

impl<M> std::ops::DerefMut for Operation<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.message
    }
}

// ===== traits =====

impl<M> AsObjectId for Operation<M> {
    #[inline]
    fn object_id(&self) -> ObjectId {
        self.object_id
    }
}

impl<M: AsNewId> AsNewId for Operation<M> {
    type Interface = M::Interface;

    #[inline]
    fn new_id(&self) -> NewId<Self::Interface> {
        self.message.new_id()
    }
}

impl<M: AsInterface> AsInterface for Operation<M> {
    #[inline]
    fn interface(&self) -> Interface {
        self.message.interface()
    }
}

impl<M: AsOpCode> AsOpCode for Operation<M> {
    type OpCode = M::OpCode;

    const OPCODE: Self::OpCode = M::OPCODE;

    const OPNAME: &str = M::OPNAME;
}

impl<M: WlMessage> WlMessage for Operation<M> {
    const IS_REQUEST: bool = M::IS_REQUEST;

    const IS_DESTRUCTOR: bool = M::IS_DESTRUCTOR;

    const SINCE: Version = M::SINCE;
}

impl<M: WlGlobal> WlGlobal for Operation<M> {
    const NAME: &str = M::NAME;

    const VERSION: Version = M::VERSION;

    const INTERFACE: Interface = M::INTERFACE;
}

impl<M: AsNewId> Constructor for Operation<M> {
    type Interface = M::Interface;

    #[inline]
    fn new_version(&self) -> Version {
        self.version
    }

    #[inline]
    fn new_id(&self) -> NewId<Self::Interface> {
        self.message.new_id()
    }
}
