use crate::wayland::{AsObjectId, MessageBuf, NewId, ObjectId, roundup4};

/// Encodable wayland message.
pub trait Encode: Sized + AsObjectId {
    const OPCODE: u16;

    fn encode(self, encoder: Encoder);

    #[inline]
    fn encode_to(self, write_buf: &mut MessageBuf) {
        let id = self.object_id().to_u32();
        let op = Self::OPCODE;
        self.encode(Encoder { id, op, write_buf });
    }
}

// ===== Encoder =====

macro_rules! encode_me {
    ($en:ident, $me:ident, $($f:ident),*) => {{
        use super::encode::Write;
        let len = 8u16$(.wrapping_add($me.$f.size()))*;
        let mut writer = unsafe { $en.encode(len) };
        $(writer .write($me.$f);)*
    }};
}

pub(super) use encode_me;

pub struct Encoder<'a> {
    id: u32,
    op: u16,
    write_buf: &'a mut MessageBuf,
}

impl<'a> Encoder<'a> {
    pub fn push_fd(&mut self, fd: i32) {
        assert!(self.write_buf.push_fd(fd), "fd capacity overflow");
    }

    /// # Safety
    ///
    /// `len` must be the exact length of the required size.
    ///
    /// Caller must ensure `len` bytes are initialized.
    pub unsafe fn encode(self, len: u16) -> Writer<'a> {
        self.encode_inner(len)
    }

    /// Utility function to encode one argument.
    pub fn encode1<E: Write>(self, value: E) {
        let size = 8 + value.size();
        value.encode(&mut self.encode_inner(size));
    }

    fn encode_inner(self, len: u16) -> Writer<'a> {
        self.write_buf.reserve(len as usize);
        let ptr = self.write_buf.spare_capacity_mut().as_mut_ptr();
        unsafe {
            self.write_buf.advance_mut(len as usize);
            ptr.cast::<u32>().write_unaligned(self.id);
            ptr.add(4).cast::<u16>().write_unaligned(self.op);
            ptr.add(6).cast::<u16>().write_unaligned(len);
            Writer {
                ptr: ptr.add(8).cast(),
                _p: std::marker::PhantomData,
            }
        }
    }
}

// ===== Writer =====

pub struct Writer<'a> {
    ptr: *mut u8,
    _p: std::marker::PhantomData<&'a mut MessageBuf>,
}

impl<'a> Writer<'a> {
    pub fn write<E: Write>(&mut self, value: E) -> &mut Self {
        value.encode(self);
        self
    }

    fn advance(&mut self, cnt: usize) {
        self.ptr = unsafe { self.ptr.add(cnt) };
    }

    fn write_ne_bytes(&mut self, ne: [u8; 4]) {
        unsafe { self.ptr.copy_from_nonoverlapping(ne.as_ptr(), 4) };
        self.advance(4);
    }
}

// ===== Writable =====

pub trait WaylandEnum {
    fn to_u32(self) -> u32;
}

pub trait Write {
    fn size(&self) -> u16;

    fn encode(self, writer: &mut Writer);
}

impl<E: WaylandEnum> Write for E {
    #[inline]
    fn size(&self) -> u16 {
        4
    }

    #[inline]
    fn encode(self, writer: &mut Writer) {
        self.to_u32().encode(writer);
    }
}

macro_rules! impl_int {
    ($me:ty) => {
        impl Write for $me {
            #[inline]
            fn size(&self) -> u16 { 4 }

            #[inline]
            fn encode(self, writer: &mut Writer) {
                writer.write_ne_bytes(self.to_ne_bytes());
            }
        }
    };
}

impl_int!(u32);
impl_int!(i32);
impl_int!(ObjectId);

impl<T> Write for NewId<T> {
    fn size(&self) -> u16 {
        self.object_id().size()
    }

    fn encode(self, writer: &mut Writer) {
        self.object_id().encode(writer);
    }
}

impl Write for &str {
    #[inline]
    fn size(&self) -> u16 {
        4 + roundup4!(self.len() as u16 + 1)
    }

    fn encode(self, writer: &mut Writer) {
        let len = self.len() as u32;
        let len_nul = len + 1;
        writer.write_ne_bytes(len_nul.to_ne_bytes());
        unsafe {
            let len = len as usize;
            writer.ptr.copy_from_nonoverlapping(self.as_ptr(), len);
            writer.ptr.add(len).write(0);
            writer.advance(roundup4!(len_nul as u16) as usize);
        }
    }
}

impl Write for Option<&str> {
    #[inline]
    fn size(&self) -> u16 {
        4 + match self {
            Some(s) => roundup4!(s.len() as u16 + 1),
            None => 0,
        }
    }

    #[inline]
    fn encode(self, writer: &mut Writer) {
        match self {
            Some(s) => s.encode(writer),
            None => writer.write_ne_bytes(const { 0u32.to_ne_bytes() }),
        }
    }
}
