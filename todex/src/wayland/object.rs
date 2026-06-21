use crate::wayland::{AsInterface, AsNewId, AsObjectId, FromObjectId, Interface, ObjectId};

// ===== trait =====

/// Type that represent a wayland object.
pub trait WlObject: FromObjectId + AsObjectId + AsInterface {}

impl<O: FromObjectId + AsObjectId + AsInterface> WlObject for O {}

// ===== object =====

/// A wayland object.
///
/// This struct can represent type safe or runtime value object.
pub struct Object<I = Any> {
    object: I,
}

/// A runtime value wayland object.
#[derive(Debug)]
pub struct Any {
    object_id: ObjectId,
    interface: Interface,
}

impl Object<Any> {
    #[inline]
    pub fn any(object_id: ObjectId, interface: Interface) -> Self {
        Self {
            object: Any {
                object_id,
                interface,
            },
        }
    }

    #[inline]
    pub fn any_from<O: WlObject>(object: O) -> Self {
        Self {
            object: Any {
                object_id: object.object_id(),
                interface: object.interface(),
            },
        }
    }
}

impl<I> Object<I> {
    #[inline]
    pub fn new(object: I) -> Object<I> {
        Object { object }
    }

    #[inline]
    pub fn into_any(self) -> Object<Any>
    where
        I: WlObject,
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

impl AsInterface for Any {
    fn interface(&self) -> Interface {
        self.interface
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

impl<T: AsNewId> AsNewId for Object<T> {
    type Interface = T::Interface;

    #[inline]
    fn new_id(&self) -> super::NewId<Self::Interface> {
        self.object.new_id()
    }
}

impl<T: AsInterface> AsInterface for Object<T> {
    #[inline]
    fn interface(&self) -> Interface {
        self.object.interface()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Object<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.object.fmt(f)
    }
}

// ===== ObjectError =====

#[derive(Debug, Clone, Copy)]
pub enum ObjectError {
    /// Unknown object id.
    UnknownId,
    /// Missmatch interface for given object id.
    InvalidId,
    /// Invalid new id of `1`.
    InvalidNewId,
    /// Out of bounds new id.
    OutOfBoundsNewId,
    /// Occupied new id.
    OccupiedNewId,
}

impl ObjectError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::UnknownId => "unknown object id",
            Self::InvalidId => "missmatch interface for given object id",
            Self::InvalidNewId => "invalid new id",
            Self::OutOfBoundsNewId => "out of bounds new id",
            Self::OccupiedNewId => "occupied new id",
        }
    }
}
