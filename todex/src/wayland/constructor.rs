use crate::wayland::{NewId, Version};

/// Type that creates new object.
pub trait Constructor {
    type Interface;

    /// Returns the version for the new object.
    fn new_version(&self) -> Version;

    /// Returns the object id for the new object.
    fn new_id(&self) -> NewId<Self::Interface>;
}

impl<C: Constructor> Constructor for &C {
    type Interface = C::Interface;

    #[inline]
    fn new_version(&self) -> Version {
        C::new_version(self)
    }

    #[inline]
    fn new_id(&self) -> NewId<Self::Interface> {
        C::new_id(self)
    }
}
