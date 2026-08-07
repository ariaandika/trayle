pub use cmsg::{Cmsg, ReadError, WriteError};
pub use memmap::{Memmap, MapError};

pub type Bytes = crate::collections::buffer::Buffer<u8>;

mod cmsg;
mod memmap;
