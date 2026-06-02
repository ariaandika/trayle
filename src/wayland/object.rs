use crate::wayland::{AsObjectId, FromObjectId, interface::AsInterface};

// ===== traits =====

pub trait WlObject: FromObjectId + AsObjectId + AsInterface {}

impl<O: FromObjectId + AsObjectId + AsInterface> WlObject for O {}

// ===== macros =====

macro_rules! simple_object {
    (pub struct $struct_name:ident;) => {
        #[derive(Debug)]
        pub struct $struct_name {
            id: ObjectId,
        }

        impl FromObjectId for $struct_name {
            #[inline]
            fn from_object_id(id: ObjectId) -> Self {
                Self { id }
            }
        }

        impl AsObjectId for $struct_name {
            #[inline]
            fn as_object_id(&self) -> ObjectId {
                self.id
            }
        }

        impl AsInterface for $struct_name {
            const INTERFACE: Interface = Interface::$struct_name;
        }
    };
}

pub(super) use simple_object;
