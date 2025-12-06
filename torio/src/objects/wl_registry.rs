use crate::objects::{Object, ObjectKind, ObjectManager};

#[derive(Debug)]
pub struct Registry {
    object_id: u32,
}

impl Registry {
    pub fn new_global_id() -> Self {
        Self::new(super::GlobalId::next())
    }

    pub fn new(object_id: u32) -> Self {
        Self { object_id }
    }

    pub fn with_manager(manager: &mut ObjectManager) -> Self {
        Self::new(manager.next_id(Self::KIND))
    }

    pub const fn object_id(&self) -> u32 {
        self.object_id
    }
}

impl Object for Registry {
    const KIND: ObjectKind = ObjectKind::Registry;
}

