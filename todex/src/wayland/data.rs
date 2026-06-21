use crate::wayland::Version;

/// Type that is associated with an opaque object data.
///
/// Object data is an integer.
pub trait AsObjectData {
    type Data: ObjectData;
}

/// An object specific data.
///
/// The data is usually an id referencing global resource.
pub trait ObjectData: Copy {
    /// Restore data from raw integer.
    fn from_raw(raw: u32) -> Self;

    /// Convert data to raw integer.
    fn to_raw(self) -> u32;
}

impl ObjectData for () {
    fn from_raw(_: u32) -> Self {}

    fn to_raw(self) -> u32 {
        0
    }
}

impl ObjectData for Version {
    fn from_raw(raw: u32) -> Self {
        Version::new(raw).expect("internal error: raw object data mutated")
    }

    fn to_raw(self) -> u32 {
        self.to_u32()
    }
}
