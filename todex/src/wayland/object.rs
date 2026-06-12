use crate::wayland::{AsInterface, AsObjectId, FromObjectId, Interface, ObjectId, WlObject};

/// A wayland object.
///
/// This struct can represent type safe or runtime value object.
pub struct Object<T = Any> {
    object: T,
}

/// A runtime value wayland object.
#[derive(Debug)]
pub struct Any {
    object_id: ObjectId,
    interface: Interface,
}

impl Object<Any> {
    pub fn any(object_id: ObjectId, interface: Interface) -> Self {
        Self {
            object: Any {
                object_id,
                interface,
            },
        }
    }

    pub fn any_from<O: WlObject>(object: O) -> Self {
        Self {
            object: Any {
                object_id: object.object_id(),
                interface: O::INTERFACE,
            },
        }
    }

    pub fn try_into<T: WlObject>(self) -> Option<Object<T>> {
        if self.object.interface == T::INTERFACE {
            Some(Object {
                object: T::from_object_id(self.object.object_id),
            })
        } else {
            None
        }
    }
}

impl<T> Object<T> {
    pub fn new(object: T) -> Object<T> {
        Object { object }
    }

    pub fn into_any(self) -> Object<Any>
    where
        T: WlObject,
    {
        Object::any_from(self.object)
    }
}

// ===== impl Any =====

impl AsObjectId for Any {
    #[inline]
    fn object_id(&self) -> ObjectId {
        self.object_id
    }
}

// ===== impl Object =====

impl<T: FromObjectId> FromObjectId for Object<T> {
    #[inline]
    fn from_object_id(id: ObjectId) -> Self {
        Self::new(T::from_object_id(id))
    }
}

impl<T: AsObjectId> AsObjectId for Object<T> {
    #[inline]
    fn object_id(&self) -> ObjectId {
        T::object_id(&self.object)
    }
}

impl<T: AsInterface> AsInterface for Object<T> {
    const INTERFACE: Interface = T::INTERFACE;

    const INTERFACE_NAME: &str = T::INTERFACE_NAME;
}

impl<T: std::fmt::Debug> std::fmt::Debug for Object<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.object.fmt(f)
    }
}
