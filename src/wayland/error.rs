

#[derive(Debug)]
pub enum WlError {
    /// Unknown op code.
    UnknownOp,
    /// Invalid size for message payload.
    InvalidSize,
}

impl std::error::Error for WlError { }

impl std::fmt::Display for WlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownOp => write!(f, "unknown op code"),
            Self::InvalidSize => write!(f, "invalid payload size"),
        }
    }
}
