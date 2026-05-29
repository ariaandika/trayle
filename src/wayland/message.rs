use std::task::Poll::{self, *};

use crate::buffer::Buffer;
use crate::wayland::{Id, InterfaceId, WlError};

// ===== Object =====

pub trait Object {
    const INTERFACE_ID: InterfaceId;

    fn id(&self) -> Id;
}

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
    /// guarantee that have at least 8 + message body length
    read_buf: &'a mut Buffer,
}

impl<'a> Frame<'a> {
    pub fn from_bytes(read_buf: &'a mut Buffer) -> Poll<Result<(Id, u16, Self), WlError>> {
        let Some(header) = read_buf.first_chunk::<8>() else {
            return Pending;
        };
        // the compiler will remove all the unwraps
        let id = Id::from_ne_bytes(*header[..4].as_array().unwrap())?;
        let op = u16::from_ne_bytes(*header[4..6].as_array().unwrap());
        let len = u16::from_ne_bytes(*header[6..8].as_array().unwrap());
        if len < 8 {
            return Ready(Err(WlError::InvalidSize));
        }
        if read_buf.len() < len as usize {
            return Pending;
        }
        Ready(Ok((id, op, Self { read_buf })))
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
