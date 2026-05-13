
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Id(u64);

const MSB: u64 = !(u64::MAX >> 1);
const VALUE_MASK: u64 = u32::MAX as u64;

impl Id {
    /// Restore `Id` from raw `u64`.
    ///
    /// Note that this method is intended to **restore** `Id` from raw integer. To construct new
    /// `Id`, use [`IdManager`].
    pub fn from_u64(value: u64) -> Id {
        Self(value)
    }

    pub fn is_static(&self) -> bool {
        self.0 & MSB == MSB
    }

    // pub fn is_dynamic(&self) -> bool {
    //     self.0 & MSB == 0
    // }

    pub fn value(&self) -> u64 {
        self.0 & VALUE_MASK
    }
}

impl From<Id> for u64 {
    fn from(value: Id) -> Self {
        value.0
    }
}

impl std::ops::Deref for Id {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct IdManager {
    id: u32,
}

impl IdManager {
    pub fn new() -> Self {
        Self { id: 0  }
    }

    fn next_id(&mut self) -> u64 {
        let id = self.id;
        self.id = (self.id + 1) & i32::MAX as u32;
        (id as u64) << 4
    }

    pub const fn generate_static(value: u32) -> Id {
        Id(MSB | value as u64)
    }

    pub fn generate_dynamic(&mut self, value: u32) -> Id {
        Id(self.next_id() | value as u64)
    }
}

