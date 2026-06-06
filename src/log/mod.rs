//! Logging abstraction.
#![allow(static_mut_refs)]

pub use level::Level;
pub use logger::{log, error, warn, info, debug, trace};
pub use logger::{lossy, flush};

#[doc(hidden)]
pub use logger::init;

mod level;
mod buffer;

pub mod logger;
