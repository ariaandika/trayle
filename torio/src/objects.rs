use std::os::fd::{AsFd, RawFd};

mod message;
mod read_buffer;

mod id;
pub mod wl_display;
pub mod wl_registry;

pub use id::{GlobalId, ObjectManager};
pub use message::{ObjectKind, Header, Message, Fixed};

pub trait Request {
    const OP_CODE: u16;

    fn object_id(&self) -> u32;

    fn write_body(&self, buffer: &mut impl WriteBuffer);
}

/// Wayland object.
pub trait Object {
    const KIND: ObjectKind;
}

/// Specialized buffer for writing wayland data types.
pub trait WriteBuffer {
    fn put_int(&mut self, int: i32);

    fn put_uint(&mut self, uint: u32);

    fn put_fixed(&mut self, fixed: Fixed);

    fn put_string(&mut self, string: &str);

    fn put_new_id(&mut self, interface: &str, version: u32, new_id: u32);

    fn put_array<T>(&mut self, array: &[T]);

    fn put_fd<Fd: AsFd>(&mut self, fd: Fd);
}

/// Specialized buffer for reading wayland data types.
pub trait ReadBuffer {
    fn get_int(&mut self) -> i32;

    fn get_uint(&mut self) -> u32;

    fn get_fixed(&mut self) -> Fixed;

    fn get_string(&mut self) -> String;

    fn get_new_id(&mut self) -> (String, u32, u32);

    fn get_array<T>(&mut self) -> Vec<T>;

    fn get_fd(&mut self) -> RawFd;
}

// ===== Utils =====

#[macro_export]
macro_rules! roundup_4 {
    ($n:expr) => {
        ((($n) + 3usize) & (usize::MAX << 2))
    };
}

pub use {roundup_4};
