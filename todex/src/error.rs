
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub enum DecodeError {
    Null,
    Insufficient,
    NoNullTerm,
    NonUtf8,
}

impl std::error::Error for DecodeError { }

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "unexpected null value"),
            Self::Insufficient => write!(f, "insufficient bytes"),
            Self::NoNullTerm => write!(f, "no null termination string"),
            Self::NonUtf8 => write!(f, "non utf-8 string"),
        }
    }
}

