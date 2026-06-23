use std::fmt;

use crate::wayland::primitives::{AsObjectId, ObjectId, Version};
use crate::wayland::object::Handle;
use crate::wayland::{AsInterface, Interface};

/// Wayland object.
///
/// This struct can represent various stage of available information for wayland object.
///
/// # Representation
///
/// As message argument, this struct only contains object id, with type safe interface.
///
/// ```
/// # use todex::wayland::object::Object;
/// # use todex::wayland::primitives::ObjectId;
/// assert_eq!(size_of::<ObjectId>(), size_of::<Object>());
/// ```
///
/// To store object in a homogeneous storage, it needs to be untyped by converting the interface
/// type into runtime value.
///
/// ```
/// # use todex::wayland::object::Object;
/// # use todex::wayland::primitives::ObjectId;
/// # use todex::wayland::Interface;
/// assert_eq!(size_of::<(ObjectId, Interface)>(), size_of::<Object<Interface>>());
/// ```
///
/// Client created object requires to store the version that it negotiate.
///
/// ```
/// # use todex::wayland::object::Object;
/// # use todex::wayland::primitives::{ObjectId, Version};
/// # use todex::wayland::Interface;
/// assert_eq!(
///     size_of::<(ObjectId, Interface, Version)>(),
///     size_of::<Object<Interface, Version>>(),
/// );
/// ```
///
/// # Note on API
///
/// High level API should not accept this struct directly, instead it should accept a generic type
/// that is composed from available traits.
#[derive(Clone, Copy)]
pub struct Object<I = (), M = (), D = ObjectId> {
    iface: I,
    marker: M,
    id: D,
}

impl Object {
    /// Create new untyped [`Object`].
    #[inline]
    pub fn new(object_id: ObjectId) -> Self {
        Self {
            iface: (),
            marker: (),
            id: object_id,
        }
    }

    /// Convert object to typed interface object.
    ///
    /// Note that this method **add** interface information, no validation are performed.
    #[inline]
    pub fn typed<I: Default>(self) -> Object<I> {
        Object {
            iface: I::default(),
            marker: (),
            id: self.id,
        }
    }

    /// Convert object to typed interface object.
    ///
    /// Note that this method **add** interface information, no validation are performed.
    #[inline]
    pub fn typed_with<I>(self, interface: I) -> Object<I> {
        Object {
            iface: interface,
            marker: (),
            id: self.id,
        }
    }
}

impl<I, M, D> Object<I, M, D> {
    #[inline]
    pub const fn from_parts(iface: I, marker: M, id: D) -> Self {
        Self { iface, marker, id }
    }
}

impl<I: AsInterface, M, D> AsInterface for Object<I, M, D> {
    #[inline]
    fn interface(&self) -> Interface {
        self.iface.interface()
    }
}

impl<I, D> Object<I, Version, D> {
    #[inline]
    pub fn version(&self) -> Version {
        self.marker
    }
}

impl<I, M> AsObjectId for Object<I, M> {
    #[inline]
    fn object_id(&self) -> ObjectId {
        self.id
    }
}

impl<I, M> Object<I, M, Handle> {
    #[inline]
    pub const fn handle(&self) -> Handle {
        self.id
    }
}

impl<I, M> Object<I, M, &'static str> {
    #[inline]
    pub const fn name(&self) -> &'static str {
        self.id
    }
}

// ===== std traits =====

impl<I, M> fmt::Debug for Object<I, M> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}
