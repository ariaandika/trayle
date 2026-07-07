use std::fmt;

use crate::handle::Handle;
use crate::wayland::primitives::{AsObjectId, AsVersion, ObjectId, Version};
use crate::wayland::object::NewId;
use crate::wayland::interface::{InterfaceMarker, AsInterface, Interface};

// ===== Object =====

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
    /// Create new untyped `Object`.
    #[inline]
    pub const fn new(object_id: ObjectId) -> Self {
        Self {
            iface: (),
            marker: (),
            id: object_id,
        }
    }
}

impl<I> Object<I> {
    /// Create new typed `Object`.
    #[inline]
    pub fn new_typed(object_id: ObjectId, interface: I) -> Self {
        Self {
            iface: interface,
            marker: (),
            id: object_id,
        }
    }

    /// Create new typed `Object` from [`NewId`].
    #[inline]
    pub fn from_new_id(new_id: NewId<I>) -> Self {
        Self {
            iface: new_id.interface,
            marker: (),
            id: new_id.id,
        }
    }
}

impl<I: InterfaceMarker> Object<I> {
    /// Create new typed `Object` from [`NewId`] and [`Interface`].
    ///
    /// # Panics
    ///
    /// Panics if the interface type does not match with given [`Interface`].
    #[inline]
    pub fn from_dynamic(new_id: NewId, interface: Interface) -> Self {
        // yes the method name sucks
        Self {
            iface: I::from_interface(interface),
            marker: (),
            id: new_id.id,
        }
    }
}

// ===== typed interface =====

impl<I: AsInterface, M, D> AsInterface for Object<I, M, D> {
    #[inline]
    fn interface(&self) -> Interface {
        self.iface.interface()
    }
}

impl<M, D> Object<Interface, M, D> {
    /// Convert the `Interface` value to type safe interface.
    ///
    /// # Panics
    ///
    /// Panics if the interface type does not match with the contained `Interface`.
    #[inline]
    pub fn with_type<I2: InterfaceMarker>(self) -> Object<I2, M, D> {
        Object {
            iface: I2::from_interface(self.iface),
            marker: self.marker,
            id: self.id,
        }
    }
}

// ===== typed marker =====

impl<I, D> AsVersion for Object<I, Version, D> {
    #[inline]
    fn version(&self) -> Version {
        self.marker
    }
}

// ===== fully typed =====

impl<I, M, D> Object<I, M, D> {
    /// Create new `Object` from parts.
    #[inline]
    pub const fn from_parts(iface: I, marker: M, id: D) -> Self {
        Self { iface, marker, id }
    }
}

impl<I, M, H> Object<I, M, Handle<H>> {
    #[inline]
    pub const fn handle(&self) -> Handle<H> {
        self.id
    }
}

impl<I, M> Object<I, M, &'static str> {
    #[inline]
    pub const fn name(&self) -> &'static str {
        self.id
    }
}

impl<I, M, D: AsObjectId> AsObjectId for Object<I, M, D> {
    #[inline]
    fn object_id(&self) -> ObjectId {
        self.id.object_id()
    }
}

// ===== std traits =====

impl<I, M> fmt::Debug for Object<I, M> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}
