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
