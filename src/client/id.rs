/// Client ID.
#[derive(Debug, Clone, Copy)]
pub struct ClientId(u64);

// `ClientId` contains unique identifier and flags

/// Max identifier is `u32::MAX`.
const ID_BITS: u64 = u32::MAX as u64;

// and the rest of the bits are flags

/// additionally, MSB will always be unset.
const MSB: u64 = i64::MIN as u64;

/// `pending` flag.
const PENDING_FLAG: u64 = MSB >> 1;

impl ClientId {
    pub(super) fn assert_raw_id(id: usize) {
        assert!(id as u64 & MSB == 0, "client id exhausted");
    }

    pub(super) fn idx(self) -> usize {
        (self.0 & ID_BITS) as usize
    }

    /// Restore client id from raw integer.
    ///
    /// Note that this should only be used to restore id from [`ClientId::to_raw`]. Creating client
    /// id cannot be done externally.
    pub fn from_raw(id: u64) -> Self {
        debug_assert!(id & MSB == 0);
        Self(id)
    }

    /// Convert to raw integer representation.
    ///
    /// The returned integer will always have its most significant bit unset.
    pub fn to_raw(self) -> u64 {
        self.0
    }

    /// Returns `true` if pending flag is set.
    pub fn is_pending(self) -> bool {
        self.0 & PENDING_FLAG == PENDING_FLAG
    }
}

/// `Interest` also have flag for pending but only on `write` event, thus this bitflag is needed
impl ClientId {
    /// Set pending flag.
    pub fn set_pending(self) -> Self {
        Self(self.0 | PENDING_FLAG)
    }

    /// Unset pending flag.
    pub fn unset_pending(self) -> Self {
        Self(self.0 & (!PENDING_FLAG))
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.idx().fmt(f)
    }
}
