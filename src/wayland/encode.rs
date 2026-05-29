use crate::buffer::Buffer;
use crate::wayland::{Id, OpCode, roundup4};

/// Encodable wayland message.
pub trait Encode: Sized {
    fn encode(self, encoder: Encoder);

    fn encode_to(self, write_buf: &mut Buffer) {
        self.encode(Encoder::new(write_buf));
    }
}

// ===== Encoder =====

pub struct Encoder<'a> {
    write_buf: &'a mut Buffer,
}

impl<'a> Encoder<'a> {
    fn new(write_buf: &'a mut Buffer) -> Self {
        Self { write_buf }
    }

    pub fn push_fd(&mut self, fd: i32) {
        assert!(self.write_buf.push_fd(fd), "fd capacity overflow");
    }

    /// # Safety
    ///
    /// `len` must be the exact length of the required size.
    ///
    /// Caller must ensure `len` bytes are initialized.
    pub unsafe fn encode<Op: OpCode>(self, id: Id, op: Op, len: u16) -> Writer<'a> {
        self.encode_inner(id, op.to_op(), len)
    }

    pub fn encode_one<Op: OpCode, E: PrimitiveEncode>(self, id: Id, op: Op, value: E) {
        let size = 8 + value.size();
        value.encode(&mut self.encode_inner(id, op.to_op(), size));
    }

    fn encode_inner(self, id: Id, op: u16, len: u16) -> Writer<'a> {
        self.write_buf.reserve(len as usize);
        let ptr = self.write_buf.spare_capacity_mut().as_mut_ptr();
        unsafe {
            self.write_buf.advance_mut(len as usize);
            ptr.cast::<u32>().write_unaligned(id.to_u32());
            ptr.add(4).cast::<u16>().write_unaligned(op);
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
    _p: std::marker::PhantomData<&'a mut Buffer>,
}

impl<'a> Writer<'a> {
    pub fn write<E: PrimitiveEncode>(&mut self, value: E) -> &mut Self {
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

// ===== primitive =====

pub trait PrimitiveEncode {
    fn size(&self) -> u16;

    fn encode(self, writer: &mut Writer);
}

macro_rules! impl_int {
    ($me:ty) => {
        impl PrimitiveEncode for $me {
            fn size(&self) -> u16 { 4 }

            fn encode(self, writer: &mut Writer) {
                writer.write_ne_bytes(self.to_ne_bytes());
            }
        }
    };
}

impl_int!(u32);
impl_int!(i32);
impl_int!(Id);

impl PrimitiveEncode for &str {
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

impl PrimitiveEncode for Option<&str> {
    fn size(&self) -> u16 {
        4 + match self {
            Some(s) => roundup4!(s.len() as u16 + 1),
            None => 0,
        }
    }

    fn encode(self, writer: &mut Writer) {
        match self {
            Some(s) => s.encode(writer),
            None => writer.write_ne_bytes(const { 0u32.to_ne_bytes() }),
        }
    }
}
