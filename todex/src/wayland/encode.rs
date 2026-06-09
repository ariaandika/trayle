use crate::sys::buffer::Buffer;
use crate::wayland::{AsObjectId, Fixed, NewId, ObjectId, OpCode, WlEnum};

// ===== Encode =====

/// Encode wayland message.
///
/// Applications may accept [`EncodeMessage`] instead, wayland object have a constructor for its
/// messages associated with object id which implement it.
pub trait Encode: Sized {
    type OpCode: OpCode;

    /// The opcode of this message.
    const OPCODE: Self::OpCode;

    /// Returns the size of the payload.
    fn size(&self) -> u16;

    /// Encode message payload.
    fn encode(self, writer: Writer);

    /// Returns an iterator of `fd`-s, if available.
    fn fds(&self) -> impl IntoIterator<Item = i32> {
        []
    }

    /// Encode wayland message with given object id.
    #[inline]
    fn encode_message(self, object_id: ObjectId, write_buf: &mut Buffer) {
        for fd in self.fds() {
            assert!(write_buf.push_fd(fd));
        }
        let size = 8 + self.size() as usize;
        write_buf.reserve(size);
        unsafe {
            let writer = Writer(write_buf.spare_capacity_mut().as_mut_ptr().cast::<u8>());
            let hdr2 = (size as u32) << u16::BITS | Self::OPCODE.to_op() as u32;

            self.encode(writer.write(object_id).write(hdr2));

            // SAFETY: `Write` implementation guarantee `size` data is initialized
            write_buf.advance_mut(size);
        }
    }
}

/// Encode wayland message associated with object id.
///
/// Applications may accept this trait instead of [`Encode`].
pub trait EncodeMessage: AsObjectId {
    fn encode_message(self, write_buf: &mut Buffer);
}

impl<E: Encode + AsObjectId> EncodeMessage for E {
    #[inline]
    fn encode_message(self, write_buf: &mut Buffer) {
        let id = self.object_id();
        Encode::encode_message(self, id, write_buf);
    }
}

// ===== Encodable =====

/// Associate object id with a message payload.
///
/// Encoding a message, requires its interface object id, thus message payload alone cannot
/// implement the encoding trait. This struct wraps the payload and its associated object id, and is
/// the one implement the encoding trait.
#[derive(Debug)]
pub struct Encodable<T> {
    pub object_id: ObjectId,
    pub payload: T,
}

impl<T> Encodable<T> {
    pub fn new<O: AsObjectId>(object: &O, payload: T) -> Self {
        Self {
            object_id: object.object_id(),
            payload,
        }
    }
}

impl<T> AsObjectId for Encodable<T> {
    #[inline]
    fn object_id(&self) -> ObjectId {
        self.object_id
    }
}

impl<T: Encode> Encode for Encodable<T> {
    type OpCode = T::OpCode;

    const OPCODE: Self::OpCode = T::OPCODE;

    #[inline]
    fn size(&self) -> u16 {
        self.payload.size()
    }

    #[inline]
    fn encode(self, writer: Writer) {
        self.payload.encode(writer);
    }

    fn fds(&self) -> impl IntoIterator<Item = i32> {
        T::fds(&self.payload)
    }
}

// ===== PrimitiveWrite =====

#[derive(Clone, Copy)]
pub struct Writer(*mut u8);

impl Writer {
    fn write_raw(self, ptr: *const u8, len: usize) -> Self {
        unsafe {
            self.0.copy_from_nonoverlapping(ptr, len);
            Self(self.0.add(len))
        }
    }

    pub fn write<W: Write>(self, value: W) -> Writer {
        value.write(self)
    }
}

// ===== Write =====

mod private {
    pub trait Sealed {
        fn write(self, writer: super::Writer) -> super::Writer;
    }
}

pub trait Sized2: Sized {
    fn size(&self) -> u16;
}

/// Writable type.
pub trait Write: Sized2 + private::Sealed { }

impl<E: WlEnum> Sized2 for E {
    fn size(&self) -> u16 {
        4
    }
}

impl<E: WlEnum> private::Sealed for E {
    #[inline]
    fn write(self, writer: Writer) -> Writer {
        self.to_u32().write(writer)
    }
}

impl<W: Sized2 + private::Sealed> Write for W { }

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
sized4!(impl<T> Sized2 for NewId<T>);

impl<T> private::Sealed for NewId<T> {
    #[inline]
    fn write(self, writer: Writer) -> Writer {
        self.object_id().write(writer)
    }
}

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

// ===== arbitrary length types =====

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
        4 + match *self {
            Some(s) => roundup4!(truncate_len(s.len()) as u16 + 1),
            None => 0,
        }
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
        match self {
            Some(s) => <_>::write(s, writer),
            None => <_>::write(0, writer),
        }
    }
}
