use std::os::fd::RawFd;
use tcio::bytes::{Buf, BytesMut};

use crate::objects::{Fixed, ReadBuffer};
use crate::roundup_4;

impl ReadBuffer for BytesMut {
    fn get_int(&mut self) -> i32 {
        let b = *self.first_chunk::<4>().unwrap();
        self.advance(4);
        i32::from_ne_bytes(b)
    }

    fn get_uint(&mut self) -> u32 {
        let b = *self.first_chunk::<4>().unwrap();
        self.advance(4);
        u32::from_ne_bytes(b)
    }

    fn get_fixed(&mut self) -> Fixed {
        let b = *self.first_chunk::<4>().unwrap();
        self.advance(4);
        Fixed::from_int(i32::from_ne_bytes(b))
    }

    fn get_string(&mut self) -> String {
        let len = u32::from_ne_bytes(*self.first_chunk::<4>().unwrap()) as usize;
        let string = String::from_utf8(self[4..4 + len - 1].to_vec()).unwrap();
        self.advance(4 + roundup_4!(len));
        string
    }

    fn get_new_id(&mut self) -> (String, u32, u32) {
        (self.get_string(), self.get_uint(), self.get_uint())
    }

    fn get_array<T>(&mut self) -> Vec<T> {
        todo!()
    }

    fn get_fd(&mut self) -> RawFd {
        panic!("BytesMut cannot get fd")
    }
}
