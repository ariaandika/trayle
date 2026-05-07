//! my_protocol
#![allow(unsafe_op_in_unsafe_fn)]
use std::slice;

use crate::error::DecodeError;
use crate::message::{DecodePayload, EncodePayload};

const fn roundup4(value: u16) -> u16 {
    (value + 3) & (u16::MAX << 2)
}

pub struct MyInterface;
pub mod my_interface {
    use super::*;
    pub const VERSION: u32 = 1;
    pub const NAME_LEN: u16 = 13;
    pub static NEW_ID: [u8; 24] = *b"\x0d\x00\x00\x00my_interface\0\0\0\0\x01\x00\x00\x00";

    /// request, opcode `0`
    pub mod test_integer {
        use super::*;
        pub const OPCODE: u16 = 0;
        pub const IS_DESTRUCTOR: bool = false;
        pub const SIZE: u16 = 20;

        pub struct TestInteger {
            pub myint: i32,
            pub myuint: u32,
            pub myfixed: f32,
            pub myobject: u32,
            pub mynullobject: u32,
        }

        impl EncodePayload for TestInteger {
            const OPCODE: u16 = 0;

            fn encoded_size(&self) -> u16 {
                20
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
                ptr.cast::<i32>().write(self.myint);
                ptr = ptr.add(4);
                ptr.cast::<u32>().write(self.myuint);
                ptr = ptr.add(4);
                ptr.cast::<i32>().write((self.myfixed * 256.0).round() as i32);
                ptr = ptr.add(4);
                ptr.cast::<u32>().write(self.myobject);
                ptr = ptr.add(4);
                ptr.cast::<u32>().write(self.mynullobject);
            }
        }

        impl<'a> DecodePayload<'a> for TestInteger {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                ptr = ptr.add(8);
                let myint = *ptr.cast::<i32>();
                ptr = ptr.add(4);
                let myuint = *ptr.cast::<u32>();
                ptr = ptr.add(4);
                let myfixed = *ptr.cast::<i32>() as f32 / 256.0;
                ptr = ptr.add(4);
                let myobject = *ptr.cast::<u32>();
                ptr = ptr.add(4);
                let mynullobject = *ptr.cast::<u32>();
                Ok(TestInteger { myint, myuint, myfixed, myobject, mynullobject, })
            }
        }

    }

    /// request, opcode `1`
    pub mod test_arr {
        use super::*;
        pub const OPCODE: u16 = 1;
        pub const IS_DESTRUCTOR: bool = false;

        pub struct TestArr<'a> {
            pub myarr: &'a [u8],
        }

        impl<'a> EncodePayload for TestArr<'a> {
            const OPCODE: u16 = 1;

            fn encoded_size(&self) -> u16 {
                4 + self.myarr.len() as u16
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
                let len = self.myarr.len() as u16;
                ptr.cast::<u32>().write(len as u32);
                ptr.add(4).copy_from_nonoverlapping(self.myarr.as_ptr(), len as usize);
            }
        }

        impl<'a> DecodePayload<'a> for TestArr<'a> {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                let mut rem = *ptr.add(6).cast::<u16>();
                ptr = ptr.add(8);
                if rem < 4 {
                    return Err(DecodeError::Insufficient);
                }
                rem -= 4;
                let myarr_len = *ptr.add(0).cast::<u32>();
                let myarr_pad_len = roundup4(myarr_len as u16);
                if rem < myarr_pad_len {
                    return Err(DecodeError::Insufficient);
                }
                rem -= myarr_pad_len;
                let myarr = slice::from_raw_parts(ptr.add(4), myarr_len as usize);
                Ok(TestArr { myarr, })
            }
        }

    }

    /// request, opcode `2`
    pub mod test_str {
        use super::*;
        pub const OPCODE: u16 = 2;
        pub const IS_DESTRUCTOR: bool = false;

        pub struct TestStr<'a> {
            pub mystr: &'a str,
        }

        impl<'a> EncodePayload for TestStr<'a> {
            const OPCODE: u16 = 2;

            fn encoded_size(&self) -> u16 {
                4 + self.mystr.len() as u16 + 1
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
                let len = self.mystr.len() as u16;
                ptr.cast::<u32>().write((len + 1) as u32);
                ptr.add(4).copy_from_nonoverlapping(self.mystr.as_ptr(), len as usize);
                ptr.add((4 + len) as usize).write(0);
            }
        }

        impl<'a> DecodePayload<'a> for TestStr<'a> {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                let mut rem = *ptr.add(6).cast::<u16>();
                ptr = ptr.add(8);
                if rem < 4 {
                    return Err(DecodeError::Insufficient);
                }
                rem -= 4;
                let mystr_len = *ptr.add(0).cast::<u32>();
                let mystr_pad_len = roundup4(mystr_len as u16);
                if rem < mystr_pad_len {
                    return Err(DecodeError::Insufficient);
                }
                rem -= mystr_pad_len;
                let [mystr @ .., 0] = slice::from_raw_parts(ptr.add(4), mystr_len as usize) else {
                    return Err(DecodeError::NoNullTerm);
                };
                let Ok(mystr) = str::from_utf8(mystr) else {
                    return Err(DecodeError::NonUtf8);
                };
                Ok(TestStr { mystr, })
            }
        }

    }

    /// request, opcode `3`
    pub mod test_opt_str {
        use super::*;
        pub const OPCODE: u16 = 3;
        pub const IS_DESTRUCTOR: bool = false;

        pub struct TestOptStr<'a> {
            pub myoptstr: Option<&'a str>,
        }

        impl<'a> EncodePayload for TestOptStr<'a> {
            const OPCODE: u16 = 3;

            fn encoded_size(&self) -> u16 {
                4 + self.myoptstr.map(|s|s.len() as u16 + 1).unwrap_or(0)
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
                match self.myoptstr {
                    Some(s) => {
                        let len = s.len() as u16;
                        ptr.cast::<u32>().write((len + 1) as u32);
                        ptr.add(4).copy_from_nonoverlapping(s.as_ptr(), len as usize);
                        ptr.add((4 + len) as usize).write(0);
                    }
                    None => {
                        ptr.cast::<u32>().write(0);
                    }
                };
            }
        }

        impl<'a> DecodePayload<'a> for TestOptStr<'a> {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                let mut rem = *ptr.add(6).cast::<u16>();
                ptr = ptr.add(8);
                if rem < 4 {
                    return Err(DecodeError::Insufficient);
                }
                rem -= 4;
                let myoptstr_len = *ptr.add(0).cast::<u32>();
                let myoptstr_pad_len = roundup4(myoptstr_len as u16);
                if rem < myoptstr_pad_len {
                    return Err(DecodeError::Insufficient);
                }
                rem -= myoptstr_pad_len;
                let myoptstr = if myoptstr_len != 0 {
                    let [myoptstr @ .., 0] = slice::from_raw_parts(ptr.add(4), myoptstr_len as usize) else {
                        return Err(DecodeError::NoNullTerm);
                    };
                    let Ok(myoptstr) = str::from_utf8(myoptstr) else {
                        return Err(DecodeError::NonUtf8);
                    };
                    Some(myoptstr)
                } else {
                    None
                };
                Ok(TestOptStr { myoptstr, })
            }
        }

    }

    /// request, opcode `4`
    pub mod test_explicit_new_id {
        use super::*;
        pub const OPCODE: u16 = 4;
        pub const IS_DESTRUCTOR: bool = false;
        pub const SIZE: u16 = 4;

        pub struct TestExplicitNewId {
            pub myexplnewid: u32,
        }

        impl EncodePayload for TestExplicitNewId {
            const OPCODE: u16 = 4;

            fn encoded_size(&self) -> u16 {
                4
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
                ptr.cast::<u32>().write(self.myexplnewid);
            }
        }

        impl<'a> DecodePayload<'a> for TestExplicitNewId {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                ptr = ptr.add(8);
                let myexplnewid = *ptr.cast::<u32>();
                Ok(TestExplicitNewId { myexplnewid, })
            }
        }

    }

    /// request, opcode `5`
    pub mod test_implicit_new_id {
        use super::*;
        pub const OPCODE: u16 = 5;
        pub const IS_DESTRUCTOR: bool = false;

        pub struct TestImplicitNewId<'a> {
            pub myimplnewid_name: &'a str,
            pub myimplnewid_version: u32,
            pub myimplnewid: u32,
        }

        impl<'a> EncodePayload for TestImplicitNewId<'a> {
            const OPCODE: u16 = 5;

            fn encoded_size(&self) -> u16 {
                12 + self.myimplnewid_name.len() as u16
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
                ptr.cast::<u32>().write(self.myimplnewid_name.len() as u32);
                ptr.add(4).copy_from_nonoverlapping(self.myimplnewid_name.as_ptr(), self.myimplnewid_name.len());
                ptr.add(4 + self.myimplnewid_name.len()).write(0);
                let myimplnewid_pad_len = roundup4(self.myimplnewid_name.len() as u16 + 1);
                ptr = ptr.add((4 + myimplnewid_pad_len) as usize);
                ptr.cast::<u32>().write(self.myimplnewid_version);
                ptr.add(4).cast::<u32>().write(self.myimplnewid);
            }
        }

        impl<'a> DecodePayload<'a> for TestImplicitNewId<'a> {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                let mut rem = *ptr.add(6).cast::<u16>();
                ptr = ptr.add(8);
                if rem < 4 {
                    return Err(DecodeError::Insufficient);
                }
                rem -= 4;
                let myimplnewid_len = *ptr.add(0).cast::<u32>();
                let myimplnewid_pad_len = roundup4(myimplnewid_len as u16);
                if rem < myimplnewid_pad_len {
                    return Err(DecodeError::Insufficient);
                }
                rem -= myimplnewid_pad_len;
                let [myimplnewid_name @ .., 0] = slice::from_raw_parts(ptr.add(4), myimplnewid_len as usize) else {
                    return Err(DecodeError::NoNullTerm);
                };
                let Ok(myimplnewid_name) = str::from_utf8(myimplnewid_name) else {
                    return Err(DecodeError::NonUtf8);
                };
                let myimplnewid_version = *ptr.add((4 + myimplnewid_pad_len) as usize).cast::<u32>();
                let myimplnewid = *ptr.add((8 + myimplnewid_pad_len) as usize).cast::<u32>();
                Ok(TestImplicitNewId { myimplnewid_name, myimplnewid_version, myimplnewid, })
            }
        }

    }

    /// request, opcode `6`
    pub mod test_mix_1 {
        use super::*;
        pub const OPCODE: u16 = 6;
        pub const IS_DESTRUCTOR: bool = false;

        pub struct TestMix1<'a> {
            pub myint: i32,
            pub myuint: u32,
            pub mystr: &'a str,
        }

        impl<'a> EncodePayload for TestMix1<'a> {
            const OPCODE: u16 = 6;

            fn encoded_size(&self) -> u16 {
                12 + self.mystr.len() as u16 + 1
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
                ptr.cast::<i32>().write(self.myint);
                ptr = ptr.add(4);
                ptr.cast::<u32>().write(self.myuint);
                ptr = ptr.add(4);
                let len = self.mystr.len() as u16;
                ptr.cast::<u32>().write((len + 1) as u32);
                ptr.add(4).copy_from_nonoverlapping(self.mystr.as_ptr(), len as usize);
                ptr.add((4 + len) as usize).write(0);
            }
        }

        impl<'a> DecodePayload<'a> for TestMix1<'a> {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                let mut rem = *ptr.add(6).cast::<u16>();
                ptr = ptr.add(8);
                if rem < 12 {
                    return Err(DecodeError::Insufficient);
                }
                rem -= 12;
                let mystr_len = *ptr.add(8).cast::<u32>();
                let mystr_pad_len = roundup4(mystr_len as u16);
                if rem < mystr_pad_len {
                    return Err(DecodeError::Insufficient);
                }
                rem -= mystr_pad_len;
                let myint = *ptr.cast::<i32>();
                ptr = ptr.add(4);
                let myuint = *ptr.cast::<u32>();
                ptr = ptr.add(4);
                let [mystr @ .., 0] = slice::from_raw_parts(ptr.add(4), mystr_len as usize) else {
                    return Err(DecodeError::NoNullTerm);
                };
                let Ok(mystr) = str::from_utf8(mystr) else {
                    return Err(DecodeError::NonUtf8);
                };
                Ok(TestMix1 { myint, myuint, mystr, })
            }
        }

    }

    /// request, opcode `7`
    pub mod test_mix_2 {
        use super::*;
        pub const OPCODE: u16 = 7;
        pub const IS_DESTRUCTOR: bool = false;

        pub struct TestMix2<'a> {
            pub mystr: &'a str,
            pub myint: i32,
            pub myuint: u32,
        }

        impl<'a> EncodePayload for TestMix2<'a> {
            const OPCODE: u16 = 7;

            fn encoded_size(&self) -> u16 {
                12 + self.mystr.len() as u16 + 1
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
                let len = self.mystr.len() as u16;
                ptr.cast::<u32>().write((len + 1) as u32);
                ptr.add(4).copy_from_nonoverlapping(self.mystr.as_ptr(), len as usize);
                ptr.add((4 + len) as usize).write(0);
                ptr = ptr.add((4 + roundup4(len + 1)) as usize);
                ptr.cast::<i32>().write(self.myint);
                ptr = ptr.add(4);
                ptr.cast::<u32>().write(self.myuint);
            }
        }

        impl<'a> DecodePayload<'a> for TestMix2<'a> {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                let mut rem = *ptr.add(6).cast::<u16>();
                ptr = ptr.add(8);
                if rem < 4 {
                    return Err(DecodeError::Insufficient);
                }
                rem -= 4;
                let mystr_len = *ptr.add(0).cast::<u32>();
                let mystr_pad_len = roundup4(mystr_len as u16);
                if rem < mystr_pad_len {
                    return Err(DecodeError::Insufficient);
                }
                rem -= mystr_pad_len;
                let [mystr @ .., 0] = slice::from_raw_parts(ptr.add(4), mystr_len as usize) else {
                    return Err(DecodeError::NoNullTerm);
                };
                let Ok(mystr) = str::from_utf8(mystr) else {
                    return Err(DecodeError::NonUtf8);
                };
                ptr = ptr.add((4 + mystr_pad_len) as usize);
                let myint = *ptr.cast::<i32>();
                ptr = ptr.add(4);
                let myuint = *ptr.cast::<u32>();
                Ok(TestMix2 { mystr, myint, myuint, })
            }
        }

    }

    /// request, opcode `8`
    pub mod test_mix_3 {
        use super::*;
        pub const OPCODE: u16 = 8;
        pub const IS_DESTRUCTOR: bool = false;

        pub struct TestMix3<'a> {
            pub mystr: &'a str,
            pub myint: i32,
            pub myuint: u32,
            pub mystr2: &'a str,
        }

        impl<'a> EncodePayload for TestMix3<'a> {
            const OPCODE: u16 = 8;

            fn encoded_size(&self) -> u16 {
                16 + self.mystr.len() as u16 + 1 + self.mystr2.len() as u16 + 1
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
                let len = self.mystr.len() as u16;
                ptr.cast::<u32>().write((len + 1) as u32);
                ptr.add(4).copy_from_nonoverlapping(self.mystr.as_ptr(), len as usize);
                ptr.add((4 + len) as usize).write(0);
                ptr = ptr.add((4 + roundup4(len + 1)) as usize);
                ptr.cast::<i32>().write(self.myint);
                ptr = ptr.add(4);
                ptr.cast::<u32>().write(self.myuint);
                ptr = ptr.add(4);
                let len = self.mystr2.len() as u16;
                ptr.cast::<u32>().write((len + 1) as u32);
                ptr.add(4).copy_from_nonoverlapping(self.mystr2.as_ptr(), len as usize);
                ptr.add((4 + len) as usize).write(0);
            }
        }

        impl<'a> DecodePayload<'a> for TestMix3<'a> {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                let mut rem = *ptr.add(6).cast::<u16>();
                ptr = ptr.add(8);
                if rem < 4 {
                    return Err(DecodeError::Insufficient);
                }
                rem -= 4;
                let mystr_len = *ptr.add(0).cast::<u32>();
                let mystr_pad_len = roundup4(mystr_len as u16);
                if rem < mystr_pad_len {
                    return Err(DecodeError::Insufficient);
                }
                rem -= mystr_pad_len;
                if rem < 12 {
                    return Err(DecodeError::Insufficient);
                }
                rem -= 12;
                let mystr2_len = *ptr.add(8).cast::<u32>();
                let mystr2_pad_len = roundup4(mystr2_len as u16);
                if rem < mystr2_pad_len {
                    return Err(DecodeError::Insufficient);
                }
                rem -= mystr2_pad_len;
                let [mystr @ .., 0] = slice::from_raw_parts(ptr.add(4), mystr_len as usize) else {
                    return Err(DecodeError::NoNullTerm);
                };
                let Ok(mystr) = str::from_utf8(mystr) else {
                    return Err(DecodeError::NonUtf8);
                };
                ptr = ptr.add((4 + mystr_pad_len) as usize);
                let myint = *ptr.cast::<i32>();
                ptr = ptr.add(4);
                let myuint = *ptr.cast::<u32>();
                ptr = ptr.add(4);
                let [mystr2 @ .., 0] = slice::from_raw_parts(ptr.add(4), mystr2_len as usize) else {
                    return Err(DecodeError::NoNullTerm);
                };
                let Ok(mystr2) = str::from_utf8(mystr2) else {
                    return Err(DecodeError::NonUtf8);
                };
                Ok(TestMix3 { mystr, myint, myuint, mystr2, })
            }
        }

    }

    /// request, opcode `9`
    pub mod test_mix_4 {
        use super::*;
        pub const OPCODE: u16 = 9;
        pub const IS_DESTRUCTOR: bool = false;

        pub struct TestMix4<'a> {
            pub myint: i32,
            pub mystr: &'a str,
            pub mystr2: &'a str,
            pub myuint: u32,
        }

        impl<'a> EncodePayload for TestMix4<'a> {
            const OPCODE: u16 = 9;

            fn encoded_size(&self) -> u16 {
                16 + self.mystr.len() as u16 + 1 + self.mystr2.len() as u16 + 1
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
                ptr.cast::<i32>().write(self.myint);
                ptr = ptr.add(4);
                let len = self.mystr.len() as u16;
                ptr.cast::<u32>().write((len + 1) as u32);
                ptr.add(4).copy_from_nonoverlapping(self.mystr.as_ptr(), len as usize);
                ptr.add((4 + len) as usize).write(0);
                ptr = ptr.add((4 + roundup4(len + 1)) as usize);
                let len = self.mystr2.len() as u16;
                ptr.cast::<u32>().write((len + 1) as u32);
                ptr.add(4).copy_from_nonoverlapping(self.mystr2.as_ptr(), len as usize);
                ptr.add((4 + len) as usize).write(0);
                ptr = ptr.add((4 + roundup4(len + 1)) as usize);
                ptr.cast::<u32>().write(self.myuint);
            }
        }

        impl<'a> DecodePayload<'a> for TestMix4<'a> {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                let mut rem = *ptr.add(6).cast::<u16>();
                ptr = ptr.add(8);
                if rem < 8 {
                    return Err(DecodeError::Insufficient);
                }
                rem -= 8;
                let mystr_len = *ptr.add(4).cast::<u32>();
                let mystr_pad_len = roundup4(mystr_len as u16);
                if rem < mystr_pad_len {
                    return Err(DecodeError::Insufficient);
                }
                rem -= mystr_pad_len;
                if rem < 4 {
                    return Err(DecodeError::Insufficient);
                }
                rem -= 4;
                let mystr2_len = *ptr.add(0).cast::<u32>();
                let mystr2_pad_len = roundup4(mystr2_len as u16);
                if rem < mystr2_pad_len {
                    return Err(DecodeError::Insufficient);
                }
                rem -= mystr2_pad_len;
                let myint = *ptr.cast::<i32>();
                ptr = ptr.add(4);
                let [mystr @ .., 0] = slice::from_raw_parts(ptr.add(4), mystr_len as usize) else {
                    return Err(DecodeError::NoNullTerm);
                };
                let Ok(mystr) = str::from_utf8(mystr) else {
                    return Err(DecodeError::NonUtf8);
                };
                ptr = ptr.add((4 + mystr_pad_len) as usize);
                let [mystr2 @ .., 0] = slice::from_raw_parts(ptr.add(4), mystr2_len as usize) else {
                    return Err(DecodeError::NoNullTerm);
                };
                let Ok(mystr2) = str::from_utf8(mystr2) else {
                    return Err(DecodeError::NonUtf8);
                };
                ptr = ptr.add((4 + mystr2_pad_len) as usize);
                let myuint = *ptr.cast::<u32>();
                Ok(TestMix4 { myint, mystr, mystr2, myuint, })
            }
        }

    }

    /// request, opcode `10`
    pub mod test_fd {
        use super::*;
        pub const OPCODE: u16 = 10;
        pub const IS_DESTRUCTOR: bool = false;
        pub const SIZE: u16 = 4;

        pub struct TestFd {
            pub myint: i32,
        }

        impl EncodePayload for TestFd {
            const OPCODE: u16 = 10;

            fn encoded_size(&self) -> u16 {
                4
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
                ptr.cast::<i32>().write(self.myint);
            }
        }

        impl<'a> DecodePayload<'a> for TestFd {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                ptr = ptr.add(8);
                let myint = *ptr.cast::<i32>();
                Ok(TestFd { myint, })
            }
        }

    }

    /// request, opcode `11`
    pub mod test_only_fd {
        use super::*;
        pub const OPCODE: u16 = 11;
        pub const IS_DESTRUCTOR: bool = false;
        pub const SIZE: u16 = 0;

        pub struct TestOnlyFd {
        }

        impl EncodePayload for TestOnlyFd {
            const OPCODE: u16 = 11;

            fn encoded_size(&self) -> u16 {
                0
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            }
        }

        impl<'a> DecodePayload<'a> for TestOnlyFd {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                ptr = ptr.add(8);
                Ok(TestOnlyFd { })
            }
        }

    }

    /// request, opcode `12`
    pub mod test_enum {
        use super::*;
        pub const OPCODE: u16 = 12;
        pub const IS_DESTRUCTOR: bool = false;
        pub const SIZE: u16 = 4;

        pub struct TestEnum {
            pub myenum: u32,
        }

        impl EncodePayload for TestEnum {
            const OPCODE: u16 = 12;

            fn encoded_size(&self) -> u16 {
                4
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
                ptr.cast::<u32>().write(self.myenum);
            }
        }

        impl<'a> DecodePayload<'a> for TestEnum {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                ptr = ptr.add(8);
                let myenum = *ptr.cast::<u32>();
                Ok(TestEnum { myenum, })
            }
        }

    }

    /// request, opcode `13`, type "destructor"
    pub mod destroy {
        use super::*;
        pub const OPCODE: u16 = 13;
        pub const IS_DESTRUCTOR: bool = true;
        pub const SIZE: u16 = 0;

        pub struct Destroy {
        }

        impl EncodePayload for Destroy {
            const OPCODE: u16 = 13;

            fn encoded_size(&self) -> u16 {
                0
            }

            unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            }
        }

        impl<'a> DecodePayload<'a> for Destroy {
            unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
                ptr = ptr.add(8);
                Ok(Destroy { })
            }
        }

    }

    /// bitfield: false
    pub enum TestEnum {
        Zero = 0,
        One = 1,
        Two = 2,
    }

    /// bitfield: true
    pub enum TestBitfield {
        FlagA = 1,
        FlagB = 2,
        FlagC = 4,
    }
}
