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
    fn from_raw(raw: usize) -> Self;

    /// Convert data to raw integer.
    fn to_raw(self) -> usize;
}

impl ObjectData for () {
    fn from_raw(_: usize) -> Self {}

    fn to_raw(self) -> usize {
        0
    }
}

impl ObjectData for Version {
    fn from_raw(raw: usize) -> Self {
        Version::new(raw as u32).expect("internal error: raw object data mutated")
    }

    fn to_raw(self) -> usize {
        self.to_u32() as usize
    }
}
