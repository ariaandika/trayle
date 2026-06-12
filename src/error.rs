use crate::log;

pub struct FatalError;

impl<E: std::fmt::Display> From<E> for FatalError {
    fn from(value: E) -> Self {
        log::error!("{value}");
        Self
    }
}
