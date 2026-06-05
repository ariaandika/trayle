use crate::wayland::{AsInterface, AsObjectId, Interface, MessageBuf, ObjectId, WlError};

// ===== Message =====

#[derive(Debug)]
pub struct Message<T> {
    id: ObjectId,
    payload: T,
}

impl<T> Message<T> {
    #[inline]
    pub fn new<O: AsObjectId>(object: &O, payload: T) -> Self {
        Self {
            id: object.object_id(),
            payload,
        }
    }
}

impl<T> std::ops::Deref for Message<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl<T> std::ops::DerefMut for Message<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.payload
    }
}

impl<T> AsObjectId for Message<T> {
    #[inline]
    fn object_id(&self) -> ObjectId {
        self.id
    }
}

impl<T: AsInterface> AsInterface for Message<T> {
    const INTERFACE: Interface = T::INTERFACE;
}

// ===== Frame =====

/// Encoded message.
pub struct Frame<'a> {
    /// - guarantee to contains one valid length message
    read_buf: &'a mut MessageBuf,
}

impl<'a> Frame<'a> {
    #[inline]
    pub fn has_frame(read_buf: &MessageBuf) -> bool {
        let Some(header) = read_buf.first_chunk::<8>() else {
            return false;
        };
        let len = u16::from_ne_bytes(*header[6..8].as_array().unwrap()) as usize;
        read_buf.len() >= len
    }

    #[inline]
    pub fn new(read_buf: &'a mut MessageBuf) -> Result<(ObjectId, u16, Self), WlError> {
        let Some(header) = read_buf.first_chunk::<8>() else {
            return Err(WlError::InvalidSize);
        };
        let Some(id) = ObjectId::new(u32::from_ne_bytes(*header[..4].as_array().unwrap())) else {
            return Err(WlError::ZeroId);
        };
        let op = u16::from_ne_bytes(*header[4..6].as_array().unwrap());
        let len = u16::from_ne_bytes(*header[6..8].as_array().unwrap());
        if len < 8 {
            return Err(WlError::InvalidSize);
        }
        if read_buf.len() < len as usize {
            return Err(WlError::InvalidSize);
        }
        Ok((id, op, Self { read_buf }))
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
