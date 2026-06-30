use crate::wayland::primitives::{AsObjectId, ObjectId};

#[derive(Default, Clone, Copy)]
pub struct DisplayId;

const ID: ObjectId = ObjectId::new(1).unwrap();

impl From<DisplayId> for ObjectId {
    #[inline]
    fn from(_: DisplayId) -> Self {
        ID
    }
}

impl AsObjectId for DisplayId {
    #[inline]
    fn object_id(&self) -> ObjectId {
        ID
    }
}

impl std::fmt::Display for DisplayId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        1u8.fmt(f)
    }
}

impl std::fmt::Debug for DisplayId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        1u8.fmt(f)
    }
}
