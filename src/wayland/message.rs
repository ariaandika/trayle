use std::task::Poll::{self, *};

use crate::buffer::Buffer;
use crate::wayland::{Id, InterfaceId, WlError};

// ===== Object =====

pub trait Object {
    const INTERFACE_ID: InterfaceId;

    fn id(&self) -> Id;
}

macro_rules! simple_object {
    (pub struct $mod_name:ident::$struct_name:ident;) => {
        #[derive(Debug)]
        pub struct $struct_name {
            id: Id,
        }

        impl FromId for $struct_name {
            #[inline]
            fn from_id(id: Id) -> Self {
                Self { id }
            }
        }

        impl Object for $struct_name {
            const INTERFACE_ID: InterfaceId = InterfaceId::$mod_name;

            fn id(&self) -> Id {
                self.id
            }
        }
    };
}

pub(super) use simple_object;

// ===== Message =====

pub struct Message<T> {
    id: Id,
    payload: T,
}

impl<T> Message<T> {
    pub fn new<O: Object>(object: &O, payload: T) -> Self {
        Self {
            id: object.id(),
            payload,
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }
}

impl<T> std::ops::Deref for Message<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl<T> std::ops::DerefMut for Message<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.payload
    }
}

// ===== Frame =====

/// Encoded message.
pub struct Frame<'a> {
    /// - guarantee to contains one valid length message
    /// - guarantee that Id is non-zero
    read_buf: &'a mut Buffer,
}

impl<'a> Frame<'a> {
    pub fn from_bytes(read_buf: &'a mut Buffer) -> Poll<Result<Self, WlError>> {
        let Some(header) = read_buf.first_chunk::<8>() else {
            return Pending;
        };
        if header.starts_with(b"\0\0\0\0") {
            return Ready(Err(WlError::ZeroId));
        }
        let len = u16::from_ne_bytes(*header[6..8].as_array().unwrap());
        if len < 8 {
            return Ready(Err(WlError::InvalidSize));
        }
        if read_buf.len() < len as usize {
            return Pending;
        }
        Ready(Ok(Self { read_buf }))
    }

    pub fn parts(&self) -> (Id, u16) {
        // SAFETY: invariant
        unsafe {
            (
                self.read_buf.as_ptr().cast::<Id>().read_unaligned(),
                self.read_buf.as_ptr().add(4).cast::<u16>().read_unaligned(),
            )
        }
    }

    #[inline]
    pub fn pop_fd(&mut self) -> Option<i32> {
        self.read_buf.pop_front_fd()
    }

    #[inline]
    pub fn body(self) -> &'a [u8] {
        let ptr = self.read_buf.as_ptr();
        unsafe {
            // SAFETY: invariant
            let len = ptr.add(6).cast::<u16>().read_unaligned() as usize;
            // SAFETY: invariant
            self.read_buf.advance_unchecked(len);
            std::slice::from_raw_parts(ptr.add(8), len - 8)
        }
    }
}
