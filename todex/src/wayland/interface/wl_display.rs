use super::*;

interface! {
    #[no_mod]
    pub struct WlDisplay;

    impl Request {
        pub fn sync(callback_id: new_id<wl_callback>);
        pub fn get_registry(registry_id: new_id<wl_registry>);
    }

    impl Event {
        pub fn error(object_id: object_id, code: uint, message: string);
        pub fn delete_id(id: uint);
    }

    #[error]
    pub enum DisplayError {
        /// Server could not find object.
        invalid_object = 0,
        /// Method does not exist on the specified interface or malformed request.
        invalid_method = 1,
        /// Server is out of memory.
        no_memory = 2,
        /// Implementation error in compositor.
        implementation = 3,
    }
}

impl AsObjectId for Error<'_> {
    #[inline]
    fn object_id(&self) -> ObjectId {
        ObjectId::wl_display()
    }
}

impl AsObjectId for DeleteId {
    #[inline]
    fn object_id(&self) -> ObjectId {
        ObjectId::wl_display()
    }
}

// ===== DisplayId =====

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
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        1u8.fmt(f)
    }
}

impl std::fmt::Debug for DisplayId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        1u8.fmt(f)
    }
}
