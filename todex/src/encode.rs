use crate::{Array, Id, NewId};

macro_rules! roundup_4 {
    ($n:expr) => {
        ((($n) + 3usize) & (usize::MAX << 2))
    };
}

pub trait Encode {
    fn size(&self) -> usize;

    /// Encode data to given buffer.
    ///
    /// For safe alternative, see [`encode()`].
    ///
    /// [`encode()`]: Encode::encode
    ///
    /// # Safety
    ///
    /// `buf` must be exactly [`size()`] length.
    ///
    /// [`size()`]: Encode::size
    unsafe fn encode_unchecked(&self, buf: &mut [u8]);

    #[inline]
    fn encode(&self, buf: &mut [u8]) {
        if buf.len() != self.size() {
            panic!("encode failed, required buf size missmatch")
        }
        // SAFETY: `buf.len() == self.size()`
        unsafe { self.encode_unchecked(buf) };
    }
}

impl Encode for u32 {
    #[inline]
    fn size(&self) -> usize {
        size_of::<Self>()
    }

    #[inline]
    unsafe fn encode_unchecked(&self, buf: &mut [u8]) {
        debug_assert_eq!(buf.len(), self.size());

        let b = self.to_ne_bytes();

        // SAFETY: caller guarantee that buf length is equal to size of self
        unsafe { std::ptr::copy(b.as_ptr(), buf.as_mut_ptr(), self.size()) };
    }
}

impl Encode for i32 {
    #[inline]
    fn size(&self) -> usize {
        size_of::<Self>()
    }

    #[inline]
    unsafe fn encode_unchecked(&self, buf: &mut [u8]) {
        debug_assert_eq!(buf.len(), self.size());

        let b = self.to_ne_bytes();

        // SAFETY: caller guarantee that buf length is equal to size of self
        unsafe { std::ptr::copy(b.as_ptr(), buf.as_mut_ptr(), self.size()) };
    }
}

impl Encode for f32 {
    #[inline]
    fn size(&self) -> usize {
        size_of::<Self>()
    }

    unsafe fn encode_unchecked(&self, buf: &mut [u8]) {
        debug_assert_eq!(buf.len(), self.size());

        let raw = (self * 256.0).round() as i32;
        let b = raw.to_ne_bytes();

        // SAFETY: caller guarantee that buf length is equal to size of self
        unsafe { std::ptr::copy(b.as_ptr(), buf.as_mut_ptr(), self.size()) };
    }
}

impl Encode for &str {
    #[inline]
    fn size(&self) -> usize {
        size_of::<u32>() + roundup_4!(self.len() + 1)
    }

    unsafe fn encode_unchecked(&self, buf: &mut [u8]) {
        debug_assert!(self.len() < u32::MAX as usize, "excessive string");
        debug_assert_eq!(buf.len(), self.size());

        // SAFETY: caller guarantee that buf length is equal to size of self
        unsafe {
            let len = self.len() as u32;

            let (l, string) = buf.split_at_mut_unchecked(len as usize);
            len.encode_unchecked(l);

            std::ptr::copy(self.as_ptr(), string.as_mut_ptr(), len as usize);
            string.as_mut_ptr().add(len as usize).write(b' ');
        }
    }
}

impl Encode for Option<&str> {
    #[inline]
    fn size(&self) -> usize {
        match self {
            Some(s) => s.size(),
            None => size_of::<u32>(),
        }
    }

    #[inline]
    unsafe fn encode_unchecked(&self, buf: &mut [u8]) {
        unsafe {
            match self {
                Some(s) => s.encode_unchecked(buf),
                None => 0u32.encode_unchecked(buf),
            }
        }
    }
}

impl Encode for Id {
    #[inline]
    fn size(&self) -> usize {
        size_of::<Self>()
    }

    #[inline]
    unsafe fn encode_unchecked(&self, buf: &mut [u8]) {
        // SAFETY: caller guarantee that buf length is equal to size of self
        unsafe { self.as_u32().encode_unchecked(buf) };
    }
}

impl Encode for Option<Id> {
    #[inline]
    fn size(&self) -> usize {
        size_of::<Id>()
    }

    #[inline]
    unsafe fn encode_unchecked(&self, buf: &mut [u8]) {
        let id = match self {
            Some(id) => id.as_u32(),
            None => 0u32,
        };
        // SAFETY: caller guarantee that buf length is equal to size of self
        unsafe { id.encode_unchecked(buf) };
    }
}

impl Encode for NewId {
    #[inline]
    fn size(&self) -> usize {
        self.name().size() + size_of::<u32>() + size_of::<u32>()
    }

    unsafe fn encode_unchecked(&self, buf: &mut [u8]) {
        // SAFETY: caller guarantee that buf length is equal to size of self
        unsafe {
            let (n, rest) = buf.split_at_mut_unchecked(self.name().size());
            self.name().encode_unchecked(n);

            let v = self.version().to_ne_bytes();
            std::ptr::copy(v.as_ptr(), rest.as_mut_ptr(), size_of::<u32>());
            let i = self.id_non_zero().get().to_ne_bytes();
            std::ptr::copy(
                i.as_ptr(),
                rest.as_mut_ptr().add(size_of::<u32>()),
                size_of::<u32>(),
            );
        }
    }
}

impl Encode for &Array {
    #[inline]
    fn size(&self) -> usize {
        size_of::<u32>() + roundup_4!(self.len())
    }

    unsafe fn encode_unchecked(&self, buf: &mut [u8]) {
        debug_assert!(self.len() < u32::MAX as usize, "excessive array");
        debug_assert_eq!(buf.len(), self.size());

        // SAFETY: caller guarantee that buf length is equal to size of self
        unsafe {
            let len = self.len() as u32;

            let (l, array) = buf.split_at_mut_unchecked(len as usize);
            len.encode_unchecked(l);

            std::ptr::copy(self.as_ptr(), array.as_mut_ptr(), len as usize);
            array.as_mut_ptr().add(len as usize).write(b' ');
        }
    }
}
