use crate::wayland::Object;
use crate::wayland::primitives::{AsObjectId, Fixed, NewId, ObjectId, Version, WlEnum};

// ===== PrimitiveWrite =====

#[derive(Clone, Copy)]
pub struct Writer(*mut u8);

impl Writer {
    pub(super) fn new(ptr: *mut u8) -> Self {
        Self(ptr)
    }

    fn write_raw(self, ptr: *const u8, len: usize) -> Self {
        unsafe {
            self.0.copy_from_nonoverlapping(ptr, len);
            Self(self.0.add(len))
        }
    }

    pub fn write<W: Write>(self, value: W) -> Self {
        value.write(self)
    }
}

// ===== Write =====

pub trait Sized2: Sized {
    fn size(&self) -> u16;
}

/// Writable type.
pub trait Write: Sized2 + private::Sealed { }

mod private {
    pub trait Sealed {
        fn write(self, writer: super::Writer) -> super::Writer;
    }
}

// ===== implementations =====

// blanket impl
impl<E: WlEnum> Sized2 for E {
    fn size(&self) -> u16 {
        4
    }
}

impl<W: Sized2 + private::Sealed> Write for W { }

impl<E: WlEnum> private::Sealed for E {
    #[inline]
    fn write(self, writer: Writer) -> Writer {
        self.to_u32().write(writer)
    }
}

macro_rules! sized4 {
    (impl $(<$g:ident> )? Sized2 for $me:ty) => {
        impl$(<$g>)? Sized2 for $me {
            #[inline]
            fn size(&self) -> u16 { 4 }
        }
    };
}

macro_rules! impl_write_for_int {
    ($me:ty) => {
        sized4!(impl Sized2 for $me);
        impl private::Sealed for $me {
            #[inline]
            fn write(self, writer: Writer) -> Writer {
                writer.write_raw(self.to_ne_bytes().as_ptr(), 4)
            }
        }
    };
}

impl_write_for_int!(u32);
impl_write_for_int!(i32);
impl_write_for_int!(ObjectId);

sized4!(impl Sized2 for Option<ObjectId>);
sized4!(impl Sized2 for Fixed);
sized4!(impl Sized2 for Version);
sized4!(impl<T> Sized2 for NewId<T>);
sized4!(impl<T> Sized2 for Object<T>);
sized4!(impl<T> Sized2 for Option<Object<T>>);

impl private::Sealed for Option<ObjectId> {
    #[inline]
    fn write(self, writer: Writer) -> Writer {
        self.map(ObjectId::to_u32).unwrap_or_default().write(writer)
    }
}

impl private::Sealed for Fixed {
    #[inline]
    fn write(self, writer: Writer) -> Writer {
        self.to_i32().write(writer)
    }
}

impl private::Sealed for Version {
    #[inline]
    fn write(self, writer: Writer) -> Writer {
        self.to_u32().write(writer)
    }
}

impl<T: AsObjectId> private::Sealed for Object<T> {
    #[inline]
    fn write(self, writer: Writer) -> Writer {
        self.object_id().write(writer)
    }
}

impl<T: AsObjectId> private::Sealed for Option<Object<T>> {
    #[inline]
    fn write(self, writer: Writer) -> Writer {
        self.map(|e| e.object_id()).write(writer)
    }
}

impl<T> private::Sealed for NewId<T> {
    #[inline]
    fn write(self, writer: Writer) -> Writer {
        self.object_id().write(writer)
    }
}

// ===== arbitrary length =====

const MAXLEN: usize = (u16::MAX >> 1) as usize;
const ZEROS: [u8; 4] = [0; 4];

fn truncate_len(len: usize) -> usize {
    len & MAXLEN
}

fn array_padding(i: usize) -> usize {
    (4 - (i & 3)) & 3
}

fn str_padding(i: usize) -> usize {
    4 - (i & 3)
}

impl Sized2 for &[u8] {
    #[inline]
    fn size(&self) -> u16 {
        4 + roundup4!(truncate_len(self.len()) as u16)
    }
}

impl private::Sealed for &[u8] {
    #[inline]
    fn write(self, writer: Writer) -> Writer {
        let len = truncate_len(self.len());
        writer
            .write(len as u32)
            .write_raw(self.as_ptr(), len)
            .write_raw(ZEROS.as_ptr(), array_padding(len))
    }
}

impl Sized2 for &str {
    #[inline]
    fn size(&self) -> u16 {
        4 + roundup4!(truncate_len(self.len()) as u16 + 1)
    }
}

impl Sized2 for Option<&str> {
    #[inline]
    fn size(&self) -> u16 {
        4 + self.map_or(0, |s| roundup4!(truncate_len(s.len()) as u16 + 1))
    }
}

impl private::Sealed for &str {
    #[inline]
    fn write(self, writer: Writer) -> Writer {
        let len = truncate_len(self.len());
        writer
            .write(len as u32 + 1)
            .write_raw(self.as_ptr(), len)
            .write_raw(ZEROS.as_ptr(), str_padding(len))
    }
}

impl private::Sealed for Option<&str> {
    #[inline]
    fn write(self, writer: Writer) -> Writer {
        self.map_or_else(|| <_>::write(0, writer), |s| <_>::write(s, writer))
    }
}
