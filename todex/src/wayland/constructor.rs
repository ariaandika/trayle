use crate::wayland::{NewId, Version};

/// Type that creates new object.
pub trait Constructor {
    type Interface;

    /// Returns the version for the new object.
    fn new_version(&self) -> Version;

    /// Returns the object id for the new object.
    fn new_id(&self) -> NewId<Self::Interface>;
}
