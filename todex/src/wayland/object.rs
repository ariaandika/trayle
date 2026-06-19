use crate::wayland::{AsGlobal, AsInterface, AsObjectId, FromObjectId, WlObject};
use crate::wayland::{Interface, ObjectId, Version};

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
                interface: object.interface(),
            },
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

// ===== Global =====

/// A runtime value global object.
#[derive(Debug)]
pub struct Global {
    pub name: &'static str,
    pub version: Version,
    pub interface: Interface,
}

impl Global {
    /// Create global from [`AsGlobal`] implementation.
    pub const fn of<G: AsGlobal>() -> Self {
        Self {
            name: G::NAME,
            version: G::VERSION,
            interface: G::INTERFACE,
        }
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
