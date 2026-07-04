use super::*;

interface! {
    #[no_mod]
    pub struct WlDisplay;

    impl Request {
        pub fn sync(callback: new_id<wl_callback>);
        pub fn get_registry(registry: new_id<wl_registry>);
    }

    impl Event {
        pub fn error(object_id: object, code: uint, message: string);
        pub fn delete_id(id: uint);
    }

    pub enum DisplayError {
        invalid_object,
        invalid_method,
        no_memory,
        implementation,
    }
}

impl<'a> Error<'a> {
    pub fn new(object_id: Object, code: u32, message: &'a str) -> Self {
        Self {
            object_id,
            code,
            message,
        }
    }
}

impl AsObjectId for Error<'_> {
    fn object_id(&self) -> ObjectId {
        ObjectId::wl_display()
    }
}

impl AsObjectId for DeleteId {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        1u8.fmt(f)
    }
}

impl std::fmt::Debug for DisplayId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        1u8.fmt(f)
    }
}
