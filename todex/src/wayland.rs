//! wayland
//!
//! # Copyright
//! Copyright © 2008-2011 Kristian Høgsberg
//! Copyright © 2010-2011 Intel Corporation
//! Copyright © 2012-2013 Collabora, Ltd.
//!
//! Permission is hereby granted, free of charge, to any person
//! obtaining a copy of this software and associated documentation files
//! (the "Software"), to deal in the Software without restriction,
//! including without limitation the rights to use, copy, modify, merge,
//! publish, distribute, sublicense, and/or sell copies of the Software,
//! and to permit persons to whom the Software is furnished to do so,
//! subject to the following conditions:
//!
//! The above copyright notice and this permission notice (including the
//! next paragraph) shall be included in all copies or substantial
//! portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
//! EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
//! MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
//! NONINFRINGEMENT.  IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
//! BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
//! ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
//! CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.
#![allow(unsafe_op_in_unsafe_fn)]
use std::slice;

use crate::error::DecodeError;
use crate::message::{DecodePayload, EncodePayload};

const fn roundup4(value: u16) -> u16 {
    (value + 3) & (u16::MAX << 2)
}

pub struct WlDisplay;
pub mod wl_display {
    use super::*;
    pub const VERSION: u32 = 1;
    pub const NAME_LEN: u16 = 11;
    pub static NEW_ID: [u8; 20] = *b"\x0b\x00\x00\x00wl_display\0\0\x01\x00\x00\x00";

    /// request, opcode `0`
    #[derive(Debug)]
    pub struct Sync {
        pub callback: u32,
    }

    impl EncodePayload for Sync {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.callback);
        }
    }

    impl<'a> DecodePayload<'a> for Sync {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let callback = *ptr.cast::<u32>();
            Ok(Sync { callback, })
        }
    }


    /// request, opcode `1`
    #[derive(Debug)]
    pub struct GetRegistry {
        pub registry: u32,
    }

    impl EncodePayload for GetRegistry {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.registry);
        }
    }

    impl<'a> DecodePayload<'a> for GetRegistry {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let registry = *ptr.cast::<u32>();
            Ok(GetRegistry { registry, })
        }
    }


    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Error<'a> {
        pub object_id: u32,
        pub code: u32,
        pub message: &'a str,
    }

    impl<'a> EncodePayload for Error<'a> {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            12 + self.message.len() as u16 + 1
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.object_id);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.code);
            ptr = ptr.add(4);
            let len = self.message.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.message.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
        }
    }

    impl<'a> DecodePayload<'a> for Error<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 12 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 12;
            let message_len = *ptr.add(8).cast::<u32>();
            let message_pad_len = roundup4(message_len as u16);
            if rem < message_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= message_pad_len;
            let object_id = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let code = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let [message @ .., 0] = slice::from_raw_parts(ptr.add(4), message_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(message) = str::from_utf8(message) else {
                return Err(DecodeError::NonUtf8);
            };
            Ok(Error { object_id, code, message, })
        }
    }


    /// bitfield: false
    pub enum ErrorEnum {
        InvalidObject = 0,
        InvalidMethod = 1,
        NoMemory = 2,
        Implementation = 3,
    }

    /// event, opcode `1`
    #[derive(Debug)]
    pub struct DeleteId {
        pub id: u32,
    }

    impl EncodePayload for DeleteId {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
        }
    }

    impl<'a> DecodePayload<'a> for DeleteId {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            Ok(DeleteId { id, })
        }
    }

}


pub struct WlRegistry;
pub mod wl_registry {
    use super::*;
    pub const VERSION: u32 = 1;
    pub const NAME_LEN: u16 = 12;
    pub static NEW_ID: [u8; 20] = *b"\x0c\x00\x00\x00wl_registry\0\x01\x00\x00\x00";

    /// request, opcode `0`
    #[derive(Debug)]
    pub struct Bind<'a> {
        pub name: u32,
        pub id_name: &'a str,
        pub id_version: u32,
        pub id: u32,
    }

    impl<'a> EncodePayload for Bind<'a> {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            16 + self.id_name.len() as u16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.name);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.id_name.len() as u32);
            ptr.add(4).copy_from_nonoverlapping(self.id_name.as_ptr(), self.id_name.len());
            ptr.add(4 + self.id_name.len()).write(0);
            let id_pad_len = roundup4(self.id_name.len() as u16 + 1);
            ptr = ptr.add((4 + id_pad_len) as usize);
            ptr.cast::<u32>().write(self.id_version);
            ptr.add(4).cast::<u32>().write(self.id);
        }
    }

    impl<'a> DecodePayload<'a> for Bind<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 8 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 8;
            let id_len = *ptr.add(4).cast::<u32>();
            let id_pad_len = roundup4(id_len as u16);
            if rem < id_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= id_pad_len;
            let name = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let [id_name @ .., 0] = slice::from_raw_parts(ptr.add(4), id_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(id_name) = str::from_utf8(id_name) else {
                return Err(DecodeError::NonUtf8);
            };
            let id_version = *ptr.add((4 + id_pad_len) as usize).cast::<u32>();
            let id = *ptr.add((8 + id_pad_len) as usize).cast::<u32>();
            Ok(Bind { name, id_name, id_version, id, })
        }
    }


    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Global<'a> {
        pub name: u32,
        pub interface: &'a str,
        pub version: u32,
    }

    impl<'a> EncodePayload for Global<'a> {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            12 + self.interface.len() as u16 + 1
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.name);
            ptr = ptr.add(4);
            let len = self.interface.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.interface.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
            ptr = ptr.add((4 + roundup4(len + 1)) as usize);
            ptr.cast::<u32>().write(self.version);
        }
    }

    impl<'a> DecodePayload<'a> for Global<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 8 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 8;
            let interface_len = *ptr.add(4).cast::<u32>();
            let interface_pad_len = roundup4(interface_len as u16);
            if rem < interface_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= interface_pad_len;
            let name = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let [interface @ .., 0] = slice::from_raw_parts(ptr.add(4), interface_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(interface) = str::from_utf8(interface) else {
                return Err(DecodeError::NonUtf8);
            };
            ptr = ptr.add((4 + interface_pad_len) as usize);
            let version = *ptr.cast::<u32>();
            Ok(Global { name, interface, version, })
        }
    }


    /// event, opcode `1`
    #[derive(Debug)]
    pub struct GlobalRemove {
        pub name: u32,
    }

    impl EncodePayload for GlobalRemove {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.name);
        }
    }

    impl<'a> DecodePayload<'a> for GlobalRemove {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let name = *ptr.cast::<u32>();
            Ok(GlobalRemove { name, })
        }
    }

}


pub struct WlCallback;
pub mod wl_callback {
    use super::*;
    pub const VERSION: u32 = 1;
    pub const NAME_LEN: u16 = 12;
    pub static NEW_ID: [u8; 20] = *b"\x0c\x00\x00\x00wl_callback\0\x01\x00\x00\x00";

    /// event, opcode `0`, type "destructor"
    #[derive(Debug)]
    pub struct Done {
        pub callback_data: u32,
    }

    impl EncodePayload for Done {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.callback_data);
        }
    }

    impl<'a> DecodePayload<'a> for Done {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let callback_data = *ptr.cast::<u32>();
            Ok(Done { callback_data, })
        }
    }

}


pub struct WlCompositor;
pub mod wl_compositor {
    use super::*;
    pub const VERSION: u32 = 7;
    pub const NAME_LEN: u16 = 14;
    pub static NEW_ID: [u8; 24] = *b"\x0e\x00\x00\x00wl_compositor\0\0\0\x07\x00\x00\x00";

    /// request, opcode `0`
    #[derive(Debug)]
    pub struct CreateSurface {
        pub id: u32,
    }

    impl EncodePayload for CreateSurface {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
        }
    }

    impl<'a> DecodePayload<'a> for CreateSurface {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            Ok(CreateSurface { id, })
        }
    }


    /// request, opcode `1`
    #[derive(Debug)]
    pub struct CreateRegion {
        pub id: u32,
    }

    impl EncodePayload for CreateRegion {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
        }
    }

    impl<'a> DecodePayload<'a> for CreateRegion {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            Ok(CreateRegion { id, })
        }
    }


    /// request, opcode `2`, type "destructor"
    #[derive(Debug)]
    pub struct Release {
    }

    impl EncodePayload for Release {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Release {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Release { })
        }
    }

}


pub struct WlShmPool;
pub mod wl_shm_pool {
    use super::*;
    pub const VERSION: u32 = 2;
    pub const NAME_LEN: u16 = 12;
    pub static NEW_ID: [u8; 20] = *b"\x0c\x00\x00\x00wl_shm_pool\0\x02\x00\x00\x00";

    /// request, opcode `0`
    #[derive(Debug)]
    pub struct CreateBuffer {
        pub id: u32,
        pub offset: i32,
        pub width: i32,
        pub height: i32,
        pub stride: i32,
        pub format: u32,
    }

    impl EncodePayload for CreateBuffer {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            24
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.offset);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.width);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.height);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.stride);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.format);
        }
    }

    impl<'a> DecodePayload<'a> for CreateBuffer {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let offset = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let width = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let height = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let stride = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let format = *ptr.cast::<u32>();
            Ok(CreateBuffer { id, offset, width, height, stride, format, })
        }
    }


    /// request, opcode `1`, type "destructor"
    #[derive(Debug)]
    pub struct Destroy {
    }

    impl EncodePayload for Destroy {
        const OPCODE: u16 = 1;

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


    /// request, opcode `2`
    #[derive(Debug)]
    pub struct Resize {
        pub size: i32,
    }

    impl EncodePayload for Resize {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.size);
        }
    }

    impl<'a> DecodePayload<'a> for Resize {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let size = *ptr.cast::<i32>();
            Ok(Resize { size, })
        }
    }

}


pub struct WlShm;
pub mod wl_shm {
    use super::*;
    pub const VERSION: u32 = 2;
    pub const NAME_LEN: u16 = 7;
    pub static NEW_ID: [u8; 16] = *b"\x07\x00\x00\x00wl_shm\0\0\x02\x00\x00\x00";

    /// bitfield: false
    pub enum ErrorEnum {
        InvalidFormat = 0,
        InvalidStride = 1,
        InvalidFd = 2,
    }

    /// bitfield: false
    pub enum FormatEnum {
        Argb8888 = 0,
        Xrgb8888 = 1,
        C8 = 0x20203843,
        Rgb332 = 0x38424752,
        Bgr233 = 0x38524742,
        Xrgb4444 = 0x32315258,
        Xbgr4444 = 0x32314258,
        Rgbx4444 = 0x32315852,
        Bgrx4444 = 0x32315842,
        Argb4444 = 0x32315241,
        Abgr4444 = 0x32314241,
        Rgba4444 = 0x32314152,
        Bgra4444 = 0x32314142,
        Xrgb1555 = 0x35315258,
        Xbgr1555 = 0x35314258,
        Rgbx5551 = 0x35315852,
        Bgrx5551 = 0x35315842,
        Argb1555 = 0x35315241,
        Abgr1555 = 0x35314241,
        Rgba5551 = 0x35314152,
        Bgra5551 = 0x35314142,
        Rgb565 = 0x36314752,
        Bgr565 = 0x36314742,
        Rgb888 = 0x34324752,
        Bgr888 = 0x34324742,
        Xbgr8888 = 0x34324258,
        Rgbx8888 = 0x34325852,
        Bgrx8888 = 0x34325842,
        Abgr8888 = 0x34324241,
        Rgba8888 = 0x34324152,
        Bgra8888 = 0x34324142,
        Xrgb2101010 = 0x30335258,
        Xbgr2101010 = 0x30334258,
        Rgbx1010102 = 0x30335852,
        Bgrx1010102 = 0x30335842,
        Argb2101010 = 0x30335241,
        Abgr2101010 = 0x30334241,
        Rgba1010102 = 0x30334152,
        Bgra1010102 = 0x30334142,
        Yuyv = 0x56595559,
        Yvyu = 0x55595659,
        Uyvy = 0x59565955,
        Vyuy = 0x59555956,
        Ayuv = 0x56555941,
        Nv12 = 0x3231564e,
        Nv21 = 0x3132564e,
        Nv16 = 0x3631564e,
        Nv61 = 0x3136564e,
        Yuv410 = 0x39565559,
        Yvu410 = 0x39555659,
        Yuv411 = 0x31315559,
        Yvu411 = 0x31315659,
        Yuv420 = 0x32315559,
        Yvu420 = 0x32315659,
        Yuv422 = 0x36315559,
        Yvu422 = 0x36315659,
        Yuv444 = 0x34325559,
        Yvu444 = 0x34325659,
        R8 = 0x20203852,
        R16 = 0x20363152,
        Rg88 = 0x38384752,
        Gr88 = 0x38385247,
        Rg1616 = 0x32334752,
        Gr1616 = 0x32335247,
        Xrgb16161616f = 0x48345258,
        Xbgr16161616f = 0x48344258,
        Argb16161616f = 0x48345241,
        Abgr16161616f = 0x48344241,
        Xyuv8888 = 0x56555958,
        Vuy888 = 0x34325556,
        Vuy101010 = 0x30335556,
        Y210 = 0x30313259,
        Y212 = 0x32313259,
        Y216 = 0x36313259,
        Y410 = 0x30313459,
        Y412 = 0x32313459,
        Y416 = 0x36313459,
        Xvyu2101010 = 0x30335658,
        Xvyu1216161616 = 0x36335658,
        Xvyu16161616 = 0x38345658,
        Y0l0 = 0x304c3059,
        X0l0 = 0x304c3058,
        Y0l2 = 0x324c3059,
        X0l2 = 0x324c3058,
        Yuv4208bit = 0x38305559,
        Yuv42010bit = 0x30315559,
        Xrgb8888A8 = 0x38415258,
        Xbgr8888A8 = 0x38414258,
        Rgbx8888A8 = 0x38415852,
        Bgrx8888A8 = 0x38415842,
        Rgb888A8 = 0x38413852,
        Bgr888A8 = 0x38413842,
        Rgb565A8 = 0x38413552,
        Bgr565A8 = 0x38413542,
        Nv24 = 0x3432564e,
        Nv42 = 0x3234564e,
        P210 = 0x30313250,
        P010 = 0x30313050,
        P012 = 0x32313050,
        P016 = 0x36313050,
        Axbxgxrx106106106106 = 0x30314241,
        Nv15 = 0x3531564e,
        Q410 = 0x30313451,
        Q401 = 0x31303451,
        Xrgb16161616 = 0x38345258,
        Xbgr16161616 = 0x38344258,
        Argb16161616 = 0x38345241,
        Abgr16161616 = 0x38344241,
        C1 = 0x20203143,
        C2 = 0x20203243,
        C4 = 0x20203443,
        D1 = 0x20203144,
        D2 = 0x20203244,
        D4 = 0x20203444,
        D8 = 0x20203844,
        R1 = 0x20203152,
        R2 = 0x20203252,
        R4 = 0x20203452,
        R10 = 0x20303152,
        R12 = 0x20323152,
        Avuy8888 = 0x59555641,
        Xvuy8888 = 0x59555658,
        P030 = 0x30333050,
        Rgb161616 = 0x38344752,
        Bgr161616 = 0x38344742,
        R16f = 0x48202052,
        Gr1616f = 0x48205247,
        Bgr161616f = 0x48524742,
        R32f = 0x46202052,
        Gr3232f = 0x46205247,
        Bgr323232f = 0x46524742,
        Abgr32323232f = 0x46384241,
        Nv20 = 0x3032564e,
        Nv30 = 0x3033564e,
        S010 = 0x30313053,
        S210 = 0x30313253,
        S410 = 0x30313453,
        S012 = 0x32313053,
        S212 = 0x32313253,
        S412 = 0x32313453,
        S016 = 0x36313053,
        S216 = 0x36313253,
        S416 = 0x36313453,
    }

    /// request, opcode `0`
    #[derive(Debug)]
    pub struct CreatePool {
        pub id: u32,
        pub size: i32,
    }

    impl EncodePayload for CreatePool {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.size);
        }
    }

    impl<'a> DecodePayload<'a> for CreatePool {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let size = *ptr.cast::<i32>();
            Ok(CreatePool { id, size, })
        }
    }


    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Format {
        pub format: u32,
    }

    impl EncodePayload for Format {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.format);
        }
    }

    impl<'a> DecodePayload<'a> for Format {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let format = *ptr.cast::<u32>();
            Ok(Format { format, })
        }
    }


    /// request, opcode `1`, type "destructor"
    #[derive(Debug)]
    pub struct Release {
    }

    impl EncodePayload for Release {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Release {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Release { })
        }
    }

}


pub struct WlBuffer;
pub mod wl_buffer {
    use super::*;
    pub const VERSION: u32 = 1;
    pub const NAME_LEN: u16 = 10;
    pub static NEW_ID: [u8; 20] = *b"\x0a\x00\x00\x00wl_buffer\0\0\0\x01\x00\x00\x00";

    /// request, opcode `0`, type "destructor"
    #[derive(Debug)]
    pub struct Destroy {
    }

    impl EncodePayload for Destroy {
        const OPCODE: u16 = 0;

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


    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Release {
    }

    impl EncodePayload for Release {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Release {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Release { })
        }
    }

}


pub struct WlDataOffer;
pub mod wl_data_offer {
    use super::*;
    pub const VERSION: u32 = 4;
    pub const NAME_LEN: u16 = 14;
    pub static NEW_ID: [u8; 24] = *b"\x0e\x00\x00\x00wl_data_offer\0\0\0\x04\x00\x00\x00";

    /// bitfield: false
    pub enum ErrorEnum {
        InvalidFinish = 0,
        InvalidActionMask = 1,
        InvalidAction = 2,
        InvalidOffer = 3,
    }

    /// request, opcode `0`
    #[derive(Debug)]
    pub struct Accept<'a> {
        pub serial: u32,
        pub mime_type: Option<&'a str>,
    }

    impl<'a> EncodePayload for Accept<'a> {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            8 + self.mime_type.map(|s|s.len() as u16 + 1).unwrap_or(0)
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            match self.mime_type {
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

    impl<'a> DecodePayload<'a> for Accept<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 8 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 8;
            let mime_type_len = *ptr.add(4).cast::<u32>();
            let mime_type_pad_len = roundup4(mime_type_len as u16);
            if rem < mime_type_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= mime_type_pad_len;
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let mime_type = if mime_type_len != 0 {
                let [mime_type @ .., 0] = slice::from_raw_parts(ptr.add(4), mime_type_len as usize) else {
                    return Err(DecodeError::NoNullTerm);
                };
                let Ok(mime_type) = str::from_utf8(mime_type) else {
                    return Err(DecodeError::NonUtf8);
                };
                Some(mime_type)
            } else {
                None
            };
            Ok(Accept { serial, mime_type, })
        }
    }


    /// request, opcode `1`
    #[derive(Debug)]
    pub struct Receive<'a> {
        pub mime_type: &'a str,
    }

    impl<'a> EncodePayload for Receive<'a> {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            4 + self.mime_type.len() as u16 + 1
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            let len = self.mime_type.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.mime_type.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
        }
    }

    impl<'a> DecodePayload<'a> for Receive<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 4 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 4;
            let mime_type_len = *ptr.add(0).cast::<u32>();
            let mime_type_pad_len = roundup4(mime_type_len as u16);
            if rem < mime_type_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= mime_type_pad_len;
            let [mime_type @ .., 0] = slice::from_raw_parts(ptr.add(4), mime_type_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(mime_type) = str::from_utf8(mime_type) else {
                return Err(DecodeError::NonUtf8);
            };
            Ok(Receive { mime_type, })
        }
    }


    /// request, opcode `2`, type "destructor"
    #[derive(Debug)]
    pub struct Destroy {
    }

    impl EncodePayload for Destroy {
        const OPCODE: u16 = 2;

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


    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Offer<'a> {
        pub mime_type: &'a str,
    }

    impl<'a> EncodePayload for Offer<'a> {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4 + self.mime_type.len() as u16 + 1
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            let len = self.mime_type.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.mime_type.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
        }
    }

    impl<'a> DecodePayload<'a> for Offer<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 4 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 4;
            let mime_type_len = *ptr.add(0).cast::<u32>();
            let mime_type_pad_len = roundup4(mime_type_len as u16);
            if rem < mime_type_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= mime_type_pad_len;
            let [mime_type @ .., 0] = slice::from_raw_parts(ptr.add(4), mime_type_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(mime_type) = str::from_utf8(mime_type) else {
                return Err(DecodeError::NonUtf8);
            };
            Ok(Offer { mime_type, })
        }
    }


    /// request, opcode `3`
    #[derive(Debug)]
    pub struct Finish {
    }

    impl EncodePayload for Finish {
        const OPCODE: u16 = 3;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Finish {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Finish { })
        }
    }


    /// request, opcode `4`
    #[derive(Debug)]
    pub struct SetActions {
        pub dnd_actions: u32,
        pub preferred_action: u32,
    }

    impl EncodePayload for SetActions {
        const OPCODE: u16 = 4;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.dnd_actions);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.preferred_action);
        }
    }

    impl<'a> DecodePayload<'a> for SetActions {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let dnd_actions = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let preferred_action = *ptr.cast::<u32>();
            Ok(SetActions { dnd_actions, preferred_action, })
        }
    }


    /// event, opcode `1`
    #[derive(Debug)]
    pub struct SourceActions {
        pub source_actions: u32,
    }

    impl EncodePayload for SourceActions {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.source_actions);
        }
    }

    impl<'a> DecodePayload<'a> for SourceActions {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let source_actions = *ptr.cast::<u32>();
            Ok(SourceActions { source_actions, })
        }
    }


    /// event, opcode `2`
    #[derive(Debug)]
    pub struct Action {
        pub dnd_action: u32,
    }

    impl EncodePayload for Action {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.dnd_action);
        }
    }

    impl<'a> DecodePayload<'a> for Action {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let dnd_action = *ptr.cast::<u32>();
            Ok(Action { dnd_action, })
        }
    }

}


pub struct WlDataSource;
pub mod wl_data_source {
    use super::*;
    pub const VERSION: u32 = 4;
    pub const NAME_LEN: u16 = 15;
    pub static NEW_ID: [u8; 24] = *b"\x0f\x00\x00\x00wl_data_source\0\0\x04\x00\x00\x00";

    /// bitfield: false
    pub enum ErrorEnum {
        InvalidActionMask = 0,
        InvalidSource = 1,
    }

    /// request, opcode `0`
    #[derive(Debug)]
    pub struct Offer<'a> {
        pub mime_type: &'a str,
    }

    impl<'a> EncodePayload for Offer<'a> {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4 + self.mime_type.len() as u16 + 1
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            let len = self.mime_type.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.mime_type.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
        }
    }

    impl<'a> DecodePayload<'a> for Offer<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 4 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 4;
            let mime_type_len = *ptr.add(0).cast::<u32>();
            let mime_type_pad_len = roundup4(mime_type_len as u16);
            if rem < mime_type_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= mime_type_pad_len;
            let [mime_type @ .., 0] = slice::from_raw_parts(ptr.add(4), mime_type_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(mime_type) = str::from_utf8(mime_type) else {
                return Err(DecodeError::NonUtf8);
            };
            Ok(Offer { mime_type, })
        }
    }


    /// request, opcode `1`, type "destructor"
    #[derive(Debug)]
    pub struct Destroy {
    }

    impl EncodePayload for Destroy {
        const OPCODE: u16 = 1;

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


    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Target<'a> {
        pub mime_type: Option<&'a str>,
    }

    impl<'a> EncodePayload for Target<'a> {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4 + self.mime_type.map(|s|s.len() as u16 + 1).unwrap_or(0)
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            match self.mime_type {
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

    impl<'a> DecodePayload<'a> for Target<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 4 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 4;
            let mime_type_len = *ptr.add(0).cast::<u32>();
            let mime_type_pad_len = roundup4(mime_type_len as u16);
            if rem < mime_type_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= mime_type_pad_len;
            let mime_type = if mime_type_len != 0 {
                let [mime_type @ .., 0] = slice::from_raw_parts(ptr.add(4), mime_type_len as usize) else {
                    return Err(DecodeError::NoNullTerm);
                };
                let Ok(mime_type) = str::from_utf8(mime_type) else {
                    return Err(DecodeError::NonUtf8);
                };
                Some(mime_type)
            } else {
                None
            };
            Ok(Target { mime_type, })
        }
    }


    /// event, opcode `1`
    #[derive(Debug)]
    pub struct Send<'a> {
        pub mime_type: &'a str,
    }

    impl<'a> EncodePayload for Send<'a> {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            4 + self.mime_type.len() as u16 + 1
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            let len = self.mime_type.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.mime_type.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
        }
    }

    impl<'a> DecodePayload<'a> for Send<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 4 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 4;
            let mime_type_len = *ptr.add(0).cast::<u32>();
            let mime_type_pad_len = roundup4(mime_type_len as u16);
            if rem < mime_type_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= mime_type_pad_len;
            let [mime_type @ .., 0] = slice::from_raw_parts(ptr.add(4), mime_type_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(mime_type) = str::from_utf8(mime_type) else {
                return Err(DecodeError::NonUtf8);
            };
            Ok(Send { mime_type, })
        }
    }


    /// event, opcode `2`
    #[derive(Debug)]
    pub struct Cancelled {
    }

    impl EncodePayload for Cancelled {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Cancelled {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Cancelled { })
        }
    }


    /// request, opcode `2`
    #[derive(Debug)]
    pub struct SetActions {
        pub dnd_actions: u32,
    }

    impl EncodePayload for SetActions {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.dnd_actions);
        }
    }

    impl<'a> DecodePayload<'a> for SetActions {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let dnd_actions = *ptr.cast::<u32>();
            Ok(SetActions { dnd_actions, })
        }
    }


    /// event, opcode `3`
    #[derive(Debug)]
    pub struct DndDropPerformed {
    }

    impl EncodePayload for DndDropPerformed {
        const OPCODE: u16 = 3;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for DndDropPerformed {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(DndDropPerformed { })
        }
    }


    /// event, opcode `4`
    #[derive(Debug)]
    pub struct DndFinished {
    }

    impl EncodePayload for DndFinished {
        const OPCODE: u16 = 4;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for DndFinished {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(DndFinished { })
        }
    }


    /// event, opcode `5`
    #[derive(Debug)]
    pub struct Action {
        pub dnd_action: u32,
    }

    impl EncodePayload for Action {
        const OPCODE: u16 = 5;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.dnd_action);
        }
    }

    impl<'a> DecodePayload<'a> for Action {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let dnd_action = *ptr.cast::<u32>();
            Ok(Action { dnd_action, })
        }
    }

}


pub struct WlDataDevice;
pub mod wl_data_device {
    use super::*;
    pub const VERSION: u32 = 4;
    pub const NAME_LEN: u16 = 15;
    pub static NEW_ID: [u8; 24] = *b"\x0f\x00\x00\x00wl_data_device\0\0\x04\x00\x00\x00";

    /// bitfield: false
    pub enum ErrorEnum {
        Role = 0,
        UsedSource = 1,
    }

    /// request, opcode `0`
    #[derive(Debug)]
    pub struct StartDrag {
        pub source: u32,
        pub origin: u32,
        pub icon: u32,
        pub serial: u32,
    }

    impl EncodePayload for StartDrag {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.source);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.origin);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.icon);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.serial);
        }
    }

    impl<'a> DecodePayload<'a> for StartDrag {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let source = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let origin = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let icon = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let serial = *ptr.cast::<u32>();
            Ok(StartDrag { source, origin, icon, serial, })
        }
    }


    /// request, opcode `1`
    #[derive(Debug)]
    pub struct SetSelection {
        pub source: u32,
        pub serial: u32,
    }

    impl EncodePayload for SetSelection {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.source);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.serial);
        }
    }

    impl<'a> DecodePayload<'a> for SetSelection {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let source = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let serial = *ptr.cast::<u32>();
            Ok(SetSelection { source, serial, })
        }
    }


    /// event, opcode `0`
    #[derive(Debug)]
    pub struct DataOffer {
        pub id: u32,
    }

    impl EncodePayload for DataOffer {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
        }
    }

    impl<'a> DecodePayload<'a> for DataOffer {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            Ok(DataOffer { id, })
        }
    }


    /// event, opcode `1`
    #[derive(Debug)]
    pub struct Enter {
        pub serial: u32,
        pub surface: u32,
        pub x: f32,
        pub y: f32,
        pub id: u32,
    }

    impl EncodePayload for Enter {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            20
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.surface);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.x * 256.0).round() as i32);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.y * 256.0).round() as i32);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.id);
        }
    }

    impl<'a> DecodePayload<'a> for Enter {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let surface = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let x = *ptr.cast::<i32>() as f32 / 256.0;
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>() as f32 / 256.0;
            ptr = ptr.add(4);
            let id = *ptr.cast::<u32>();
            Ok(Enter { serial, surface, x, y, id, })
        }
    }


    /// event, opcode `2`
    #[derive(Debug)]
    pub struct Leave {
    }

    impl EncodePayload for Leave {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Leave {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Leave { })
        }
    }


    /// event, opcode `3`
    #[derive(Debug)]
    pub struct Motion {
        pub time: u32,
        pub x: f32,
        pub y: f32,
    }

    impl EncodePayload for Motion {
        const OPCODE: u16 = 3;

        fn encoded_size(&self) -> u16 {
            12
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.time);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.x * 256.0).round() as i32);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.y * 256.0).round() as i32);
        }
    }

    impl<'a> DecodePayload<'a> for Motion {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let time = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let x = *ptr.cast::<i32>() as f32 / 256.0;
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>() as f32 / 256.0;
            Ok(Motion { time, x, y, })
        }
    }


    /// event, opcode `4`
    #[derive(Debug)]
    pub struct Drop {
    }

    impl EncodePayload for Drop {
        const OPCODE: u16 = 4;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Drop {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Drop { })
        }
    }


    /// event, opcode `5`
    #[derive(Debug)]
    pub struct Selection {
        pub id: u32,
    }

    impl EncodePayload for Selection {
        const OPCODE: u16 = 5;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
        }
    }

    impl<'a> DecodePayload<'a> for Selection {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            Ok(Selection { id, })
        }
    }


    /// request, opcode `2`, type "destructor"
    #[derive(Debug)]
    pub struct Release {
    }

    impl EncodePayload for Release {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Release {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Release { })
        }
    }

}


pub struct WlDataDeviceManager;
pub mod wl_data_device_manager {
    use super::*;
    pub const VERSION: u32 = 4;
    pub const NAME_LEN: u16 = 23;
    pub static NEW_ID: [u8; 32] = *b"\x17\x00\x00\x00wl_data_device_manager\0\0\x04\x00\x00\x00";

    /// request, opcode `0`
    #[derive(Debug)]
    pub struct CreateDataSource {
        pub id: u32,
    }

    impl EncodePayload for CreateDataSource {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
        }
    }

    impl<'a> DecodePayload<'a> for CreateDataSource {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            Ok(CreateDataSource { id, })
        }
    }


    /// request, opcode `1`
    #[derive(Debug)]
    pub struct GetDataDevice {
        pub id: u32,
        pub seat: u32,
    }

    impl EncodePayload for GetDataDevice {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.seat);
        }
    }

    impl<'a> DecodePayload<'a> for GetDataDevice {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let seat = *ptr.cast::<u32>();
            Ok(GetDataDevice { id, seat, })
        }
    }


    /// since: 3
    /// bitfield: true
    pub enum DndActionEnum {
        None = 0,
        Copy = 1,
        Move = 2,
        Ask = 4,
    }

    /// request, opcode `2`, type "destructor"
    #[derive(Debug)]
    pub struct Release {
    }

    impl EncodePayload for Release {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Release {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Release { })
        }
    }

}


pub struct WlShell;
pub mod wl_shell {
    use super::*;
    pub const VERSION: u32 = 1;
    pub const NAME_LEN: u16 = 9;
    pub static NEW_ID: [u8; 20] = *b"\x09\x00\x00\x00wl_shell\0\0\0\0\x01\x00\x00\x00";

    /// bitfield: false
    pub enum ErrorEnum {
        Role = 0,
    }

    /// request, opcode `0`
    #[derive(Debug)]
    pub struct GetShellSurface {
        pub id: u32,
        pub surface: u32,
    }

    impl EncodePayload for GetShellSurface {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.surface);
        }
    }

    impl<'a> DecodePayload<'a> for GetShellSurface {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let surface = *ptr.cast::<u32>();
            Ok(GetShellSurface { id, surface, })
        }
    }

}


pub struct WlShellSurface;
pub mod wl_shell_surface {
    use super::*;
    pub const VERSION: u32 = 1;
    pub const NAME_LEN: u16 = 17;
    pub static NEW_ID: [u8; 28] = *b"\x11\x00\x00\x00wl_shell_surface\0\0\0\0\x01\x00\x00\x00";

    /// request, opcode `0`
    #[derive(Debug)]
    pub struct Pong {
        pub serial: u32,
    }

    impl EncodePayload for Pong {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
        }
    }

    impl<'a> DecodePayload<'a> for Pong {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let serial = *ptr.cast::<u32>();
            Ok(Pong { serial, })
        }
    }


    /// request, opcode `1`
    #[derive(Debug)]
    pub struct Move {
        pub seat: u32,
        pub serial: u32,
    }

    impl EncodePayload for Move {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.seat);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.serial);
        }
    }

    impl<'a> DecodePayload<'a> for Move {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let seat = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let serial = *ptr.cast::<u32>();
            Ok(Move { seat, serial, })
        }
    }


    /// bitfield: true
    pub enum ResizeEnum {
        None = 0,
        Top = 1,
        Bottom = 2,
        Left = 4,
        TopLeft = 5,
        BottomLeft = 6,
        Right = 8,
        TopRight = 9,
        BottomRight = 10,
    }

    /// request, opcode `2`
    #[derive(Debug)]
    pub struct Resize {
        pub seat: u32,
        pub serial: u32,
        pub edges: u32,
    }

    impl EncodePayload for Resize {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            12
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.seat);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.edges);
        }
    }

    impl<'a> DecodePayload<'a> for Resize {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let seat = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let edges = *ptr.cast::<u32>();
            Ok(Resize { seat, serial, edges, })
        }
    }


    /// request, opcode `3`
    #[derive(Debug)]
    pub struct SetToplevel {
    }

    impl EncodePayload for SetToplevel {
        const OPCODE: u16 = 3;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for SetToplevel {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(SetToplevel { })
        }
    }


    /// bitfield: true
    pub enum TransientEnum {
        Inactive = 0x1,
    }

    /// request, opcode `4`
    #[derive(Debug)]
    pub struct SetTransient {
        pub parent: u32,
        pub x: i32,
        pub y: i32,
        pub flags: u32,
    }

    impl EncodePayload for SetTransient {
        const OPCODE: u16 = 4;

        fn encoded_size(&self) -> u16 {
            16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.parent);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.x);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.y);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.flags);
        }
    }

    impl<'a> DecodePayload<'a> for SetTransient {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let parent = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let x = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let flags = *ptr.cast::<u32>();
            Ok(SetTransient { parent, x, y, flags, })
        }
    }


    /// bitfield: false
    pub enum FullscreenMethodEnum {
        Default = 0,
        Scale = 1,
        Driver = 2,
        Fill = 3,
    }

    /// request, opcode `5`
    #[derive(Debug)]
    pub struct SetFullscreen {
        pub method: u32,
        pub framerate: u32,
        pub output: u32,
    }

    impl EncodePayload for SetFullscreen {
        const OPCODE: u16 = 5;

        fn encoded_size(&self) -> u16 {
            12
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.method);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.framerate);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.output);
        }
    }

    impl<'a> DecodePayload<'a> for SetFullscreen {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let method = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let framerate = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let output = *ptr.cast::<u32>();
            Ok(SetFullscreen { method, framerate, output, })
        }
    }


    /// request, opcode `6`
    #[derive(Debug)]
    pub struct SetPopup {
        pub seat: u32,
        pub serial: u32,
        pub parent: u32,
        pub x: i32,
        pub y: i32,
        pub flags: u32,
    }

    impl EncodePayload for SetPopup {
        const OPCODE: u16 = 6;

        fn encoded_size(&self) -> u16 {
            24
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.seat);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.parent);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.x);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.y);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.flags);
        }
    }

    impl<'a> DecodePayload<'a> for SetPopup {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let seat = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let parent = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let x = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let flags = *ptr.cast::<u32>();
            Ok(SetPopup { seat, serial, parent, x, y, flags, })
        }
    }


    /// request, opcode `7`
    #[derive(Debug)]
    pub struct SetMaximized {
        pub output: u32,
    }

    impl EncodePayload for SetMaximized {
        const OPCODE: u16 = 7;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.output);
        }
    }

    impl<'a> DecodePayload<'a> for SetMaximized {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let output = *ptr.cast::<u32>();
            Ok(SetMaximized { output, })
        }
    }


    /// request, opcode `8`
    #[derive(Debug)]
    pub struct SetTitle<'a> {
        pub title: &'a str,
    }

    impl<'a> EncodePayload for SetTitle<'a> {
        const OPCODE: u16 = 8;

        fn encoded_size(&self) -> u16 {
            4 + self.title.len() as u16 + 1
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            let len = self.title.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.title.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
        }
    }

    impl<'a> DecodePayload<'a> for SetTitle<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 4 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 4;
            let title_len = *ptr.add(0).cast::<u32>();
            let title_pad_len = roundup4(title_len as u16);
            if rem < title_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= title_pad_len;
            let [title @ .., 0] = slice::from_raw_parts(ptr.add(4), title_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(title) = str::from_utf8(title) else {
                return Err(DecodeError::NonUtf8);
            };
            Ok(SetTitle { title, })
        }
    }


    /// request, opcode `9`
    #[derive(Debug)]
    pub struct SetClass<'a> {
        pub class_: &'a str,
    }

    impl<'a> EncodePayload for SetClass<'a> {
        const OPCODE: u16 = 9;

        fn encoded_size(&self) -> u16 {
            4 + self.class_.len() as u16 + 1
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            let len = self.class_.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.class_.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
        }
    }

    impl<'a> DecodePayload<'a> for SetClass<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 4 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 4;
            let class__len = *ptr.add(0).cast::<u32>();
            let class__pad_len = roundup4(class__len as u16);
            if rem < class__pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= class__pad_len;
            let [class_ @ .., 0] = slice::from_raw_parts(ptr.add(4), class__len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(class_) = str::from_utf8(class_) else {
                return Err(DecodeError::NonUtf8);
            };
            Ok(SetClass { class_, })
        }
    }


    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Ping {
        pub serial: u32,
    }

    impl EncodePayload for Ping {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
        }
    }

    impl<'a> DecodePayload<'a> for Ping {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let serial = *ptr.cast::<u32>();
            Ok(Ping { serial, })
        }
    }


    /// event, opcode `1`
    #[derive(Debug)]
    pub struct Configure {
        pub edges: u32,
        pub width: i32,
        pub height: i32,
    }

    impl EncodePayload for Configure {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            12
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.edges);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.width);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.height);
        }
    }

    impl<'a> DecodePayload<'a> for Configure {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let edges = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let width = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let height = *ptr.cast::<i32>();
            Ok(Configure { edges, width, height, })
        }
    }


    /// event, opcode `2`
    #[derive(Debug)]
    pub struct PopupDone {
    }

    impl EncodePayload for PopupDone {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for PopupDone {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(PopupDone { })
        }
    }

}


pub struct WlSurface;
pub mod wl_surface {
    use super::*;
    pub const VERSION: u32 = 7;
    pub const NAME_LEN: u16 = 11;
    pub static NEW_ID: [u8; 20] = *b"\x0b\x00\x00\x00wl_surface\0\0\x07\x00\x00\x00";

    /// bitfield: false
    pub enum ErrorEnum {
        InvalidScale = 0,
        InvalidTransform = 1,
        InvalidSize = 2,
        InvalidOffset = 3,
        DefunctRoleObject = 4,
        NoBuffer = 5,
    }

    /// request, opcode `0`, type "destructor"
    #[derive(Debug)]
    pub struct Destroy {
    }

    impl EncodePayload for Destroy {
        const OPCODE: u16 = 0;

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


    /// request, opcode `1`
    #[derive(Debug)]
    pub struct Attach {
        pub buffer: u32,
        pub x: i32,
        pub y: i32,
    }

    impl EncodePayload for Attach {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            12
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.buffer);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.x);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.y);
        }
    }

    impl<'a> DecodePayload<'a> for Attach {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let buffer = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let x = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>();
            Ok(Attach { buffer, x, y, })
        }
    }


    /// request, opcode `2`
    #[derive(Debug)]
    pub struct Damage {
        pub x: i32,
        pub y: i32,
        pub width: i32,
        pub height: i32,
    }

    impl EncodePayload for Damage {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.x);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.y);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.width);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.height);
        }
    }

    impl<'a> DecodePayload<'a> for Damage {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let x = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let width = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let height = *ptr.cast::<i32>();
            Ok(Damage { x, y, width, height, })
        }
    }


    /// request, opcode `3`
    #[derive(Debug)]
    pub struct Frame {
        pub callback: u32,
    }

    impl EncodePayload for Frame {
        const OPCODE: u16 = 3;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.callback);
        }
    }

    impl<'a> DecodePayload<'a> for Frame {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let callback = *ptr.cast::<u32>();
            Ok(Frame { callback, })
        }
    }


    /// request, opcode `4`
    #[derive(Debug)]
    pub struct SetOpaqueRegion {
        pub region: u32,
    }

    impl EncodePayload for SetOpaqueRegion {
        const OPCODE: u16 = 4;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.region);
        }
    }

    impl<'a> DecodePayload<'a> for SetOpaqueRegion {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let region = *ptr.cast::<u32>();
            Ok(SetOpaqueRegion { region, })
        }
    }


    /// request, opcode `5`
    #[derive(Debug)]
    pub struct SetInputRegion {
        pub region: u32,
    }

    impl EncodePayload for SetInputRegion {
        const OPCODE: u16 = 5;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.region);
        }
    }

    impl<'a> DecodePayload<'a> for SetInputRegion {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let region = *ptr.cast::<u32>();
            Ok(SetInputRegion { region, })
        }
    }


    /// request, opcode `6`
    #[derive(Debug)]
    pub struct Commit {
    }

    impl EncodePayload for Commit {
        const OPCODE: u16 = 6;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Commit {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Commit { })
        }
    }


    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Enter {
        pub output: u32,
    }

    impl EncodePayload for Enter {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.output);
        }
    }

    impl<'a> DecodePayload<'a> for Enter {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let output = *ptr.cast::<u32>();
            Ok(Enter { output, })
        }
    }


    /// event, opcode `1`
    #[derive(Debug)]
    pub struct Leave {
        pub output: u32,
    }

    impl EncodePayload for Leave {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.output);
        }
    }

    impl<'a> DecodePayload<'a> for Leave {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let output = *ptr.cast::<u32>();
            Ok(Leave { output, })
        }
    }


    /// request, opcode `7`
    #[derive(Debug)]
    pub struct SetBufferTransform {
        pub transform: i32,
    }

    impl EncodePayload for SetBufferTransform {
        const OPCODE: u16 = 7;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.transform);
        }
    }

    impl<'a> DecodePayload<'a> for SetBufferTransform {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let transform = *ptr.cast::<i32>();
            Ok(SetBufferTransform { transform, })
        }
    }


    /// request, opcode `8`
    #[derive(Debug)]
    pub struct SetBufferScale {
        pub scale: i32,
    }

    impl EncodePayload for SetBufferScale {
        const OPCODE: u16 = 8;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.scale);
        }
    }

    impl<'a> DecodePayload<'a> for SetBufferScale {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let scale = *ptr.cast::<i32>();
            Ok(SetBufferScale { scale, })
        }
    }


    /// request, opcode `9`
    #[derive(Debug)]
    pub struct DamageBuffer {
        pub x: i32,
        pub y: i32,
        pub width: i32,
        pub height: i32,
    }

    impl EncodePayload for DamageBuffer {
        const OPCODE: u16 = 9;

        fn encoded_size(&self) -> u16 {
            16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.x);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.y);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.width);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.height);
        }
    }

    impl<'a> DecodePayload<'a> for DamageBuffer {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let x = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let width = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let height = *ptr.cast::<i32>();
            Ok(DamageBuffer { x, y, width, height, })
        }
    }


    /// request, opcode `10`
    #[derive(Debug)]
    pub struct Offset {
        pub x: i32,
        pub y: i32,
    }

    impl EncodePayload for Offset {
        const OPCODE: u16 = 10;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.x);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.y);
        }
    }

    impl<'a> DecodePayload<'a> for Offset {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let x = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>();
            Ok(Offset { x, y, })
        }
    }


    /// event, opcode `2`
    #[derive(Debug)]
    pub struct PreferredBufferScale {
        pub factor: i32,
    }

    impl EncodePayload for PreferredBufferScale {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.factor);
        }
    }

    impl<'a> DecodePayload<'a> for PreferredBufferScale {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let factor = *ptr.cast::<i32>();
            Ok(PreferredBufferScale { factor, })
        }
    }


    /// event, opcode `3`
    #[derive(Debug)]
    pub struct PreferredBufferTransform {
        pub transform: u32,
    }

    impl EncodePayload for PreferredBufferTransform {
        const OPCODE: u16 = 3;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.transform);
        }
    }

    impl<'a> DecodePayload<'a> for PreferredBufferTransform {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let transform = *ptr.cast::<u32>();
            Ok(PreferredBufferTransform { transform, })
        }
    }


    /// request, opcode `11`
    #[derive(Debug)]
    pub struct GetRelease {
        pub callback: u32,
    }

    impl EncodePayload for GetRelease {
        const OPCODE: u16 = 11;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.callback);
        }
    }

    impl<'a> DecodePayload<'a> for GetRelease {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let callback = *ptr.cast::<u32>();
            Ok(GetRelease { callback, })
        }
    }

}


pub struct WlSeat;
pub mod wl_seat {
    use super::*;
    pub const VERSION: u32 = 10;
    pub const NAME_LEN: u16 = 8;
    pub static NEW_ID: [u8; 16] = *b"\x08\x00\x00\x00wl_seat\0\x0a\x00\x00\x00";

    /// bitfield: true
    pub enum CapabilityEnum {
        Pointer = 1,
        Keyboard = 2,
        Touch = 4,
    }

    /// bitfield: false
    pub enum ErrorEnum {
        MissingCapability = 0,
    }

    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Capabilities {
        pub capabilities: u32,
    }

    impl EncodePayload for Capabilities {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.capabilities);
        }
    }

    impl<'a> DecodePayload<'a> for Capabilities {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let capabilities = *ptr.cast::<u32>();
            Ok(Capabilities { capabilities, })
        }
    }


    /// request, opcode `0`
    #[derive(Debug)]
    pub struct GetPointer {
        pub id: u32,
    }

    impl EncodePayload for GetPointer {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
        }
    }

    impl<'a> DecodePayload<'a> for GetPointer {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            Ok(GetPointer { id, })
        }
    }


    /// request, opcode `1`
    #[derive(Debug)]
    pub struct GetKeyboard {
        pub id: u32,
    }

    impl EncodePayload for GetKeyboard {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
        }
    }

    impl<'a> DecodePayload<'a> for GetKeyboard {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            Ok(GetKeyboard { id, })
        }
    }


    /// request, opcode `2`
    #[derive(Debug)]
    pub struct GetTouch {
        pub id: u32,
    }

    impl EncodePayload for GetTouch {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
        }
    }

    impl<'a> DecodePayload<'a> for GetTouch {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            Ok(GetTouch { id, })
        }
    }


    /// event, opcode `1`
    #[derive(Debug)]
    pub struct Name<'a> {
        pub name: &'a str,
    }

    impl<'a> EncodePayload for Name<'a> {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            4 + self.name.len() as u16 + 1
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            let len = self.name.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.name.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
        }
    }

    impl<'a> DecodePayload<'a> for Name<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 4 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 4;
            let name_len = *ptr.add(0).cast::<u32>();
            let name_pad_len = roundup4(name_len as u16);
            if rem < name_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= name_pad_len;
            let [name @ .., 0] = slice::from_raw_parts(ptr.add(4), name_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(name) = str::from_utf8(name) else {
                return Err(DecodeError::NonUtf8);
            };
            Ok(Name { name, })
        }
    }


    /// request, opcode `3`, type "destructor"
    #[derive(Debug)]
    pub struct Release {
    }

    impl EncodePayload for Release {
        const OPCODE: u16 = 3;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Release {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Release { })
        }
    }

}


pub struct WlPointer;
pub mod wl_pointer {
    use super::*;
    pub const VERSION: u32 = 10;
    pub const NAME_LEN: u16 = 11;
    pub static NEW_ID: [u8; 20] = *b"\x0b\x00\x00\x00wl_pointer\0\0\x0a\x00\x00\x00";

    /// bitfield: false
    pub enum ErrorEnum {
        Role = 0,
    }

    /// request, opcode `0`
    #[derive(Debug)]
    pub struct SetCursor {
        pub serial: u32,
        pub surface: u32,
        pub hotspot_x: i32,
        pub hotspot_y: i32,
    }

    impl EncodePayload for SetCursor {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.surface);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.hotspot_x);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.hotspot_y);
        }
    }

    impl<'a> DecodePayload<'a> for SetCursor {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let surface = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let hotspot_x = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let hotspot_y = *ptr.cast::<i32>();
            Ok(SetCursor { serial, surface, hotspot_x, hotspot_y, })
        }
    }


    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Enter {
        pub serial: u32,
        pub surface: u32,
        pub surface_x: f32,
        pub surface_y: f32,
    }

    impl EncodePayload for Enter {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.surface);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.surface_x * 256.0).round() as i32);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.surface_y * 256.0).round() as i32);
        }
    }

    impl<'a> DecodePayload<'a> for Enter {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let surface = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let surface_x = *ptr.cast::<i32>() as f32 / 256.0;
            ptr = ptr.add(4);
            let surface_y = *ptr.cast::<i32>() as f32 / 256.0;
            Ok(Enter { serial, surface, surface_x, surface_y, })
        }
    }


    /// event, opcode `1`
    #[derive(Debug)]
    pub struct Leave {
        pub serial: u32,
        pub surface: u32,
    }

    impl EncodePayload for Leave {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.surface);
        }
    }

    impl<'a> DecodePayload<'a> for Leave {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let surface = *ptr.cast::<u32>();
            Ok(Leave { serial, surface, })
        }
    }


    /// event, opcode `2`
    #[derive(Debug)]
    pub struct Motion {
        pub time: u32,
        pub surface_x: f32,
        pub surface_y: f32,
    }

    impl EncodePayload for Motion {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            12
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.time);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.surface_x * 256.0).round() as i32);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.surface_y * 256.0).round() as i32);
        }
    }

    impl<'a> DecodePayload<'a> for Motion {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let time = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let surface_x = *ptr.cast::<i32>() as f32 / 256.0;
            ptr = ptr.add(4);
            let surface_y = *ptr.cast::<i32>() as f32 / 256.0;
            Ok(Motion { time, surface_x, surface_y, })
        }
    }


    /// bitfield: false
    pub enum ButtonStateEnum {
        Released = 0,
        Pressed = 1,
    }

    /// event, opcode `3`
    #[derive(Debug)]
    pub struct Button {
        pub serial: u32,
        pub time: u32,
        pub button: u32,
        pub state: u32,
    }

    impl EncodePayload for Button {
        const OPCODE: u16 = 3;

        fn encoded_size(&self) -> u16 {
            16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.time);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.button);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.state);
        }
    }

    impl<'a> DecodePayload<'a> for Button {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let time = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let button = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let state = *ptr.cast::<u32>();
            Ok(Button { serial, time, button, state, })
        }
    }


    /// bitfield: false
    pub enum AxisEnum {
        VerticalScroll = 0,
        HorizontalScroll = 1,
    }

    /// event, opcode `4`
    #[derive(Debug)]
    pub struct Axis {
        pub time: u32,
        pub axis: u32,
        pub value: f32,
    }

    impl EncodePayload for Axis {
        const OPCODE: u16 = 4;

        fn encoded_size(&self) -> u16 {
            12
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.time);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.axis);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.value * 256.0).round() as i32);
        }
    }

    impl<'a> DecodePayload<'a> for Axis {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let time = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let axis = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let value = *ptr.cast::<i32>() as f32 / 256.0;
            Ok(Axis { time, axis, value, })
        }
    }


    /// request, opcode `1`, type "destructor"
    #[derive(Debug)]
    pub struct Release {
    }

    impl EncodePayload for Release {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Release {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Release { })
        }
    }


    /// event, opcode `5`
    #[derive(Debug)]
    pub struct Frame {
    }

    impl EncodePayload for Frame {
        const OPCODE: u16 = 5;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Frame {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Frame { })
        }
    }


    /// bitfield: false
    pub enum AxisSourceEnum {
        Wheel = 0,
        Finger = 1,
        Continuous = 2,
        /// since: 6
        WheelTilt = 3,
    }

    /// event, opcode `6`
    #[derive(Debug)]
    pub struct AxisSource {
        pub axis_source: u32,
    }

    impl EncodePayload for AxisSource {
        const OPCODE: u16 = 6;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.axis_source);
        }
    }

    impl<'a> DecodePayload<'a> for AxisSource {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let axis_source = *ptr.cast::<u32>();
            Ok(AxisSource { axis_source, })
        }
    }


    /// event, opcode `7`
    #[derive(Debug)]
    pub struct AxisStop {
        pub time: u32,
        pub axis: u32,
    }

    impl EncodePayload for AxisStop {
        const OPCODE: u16 = 7;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.time);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.axis);
        }
    }

    impl<'a> DecodePayload<'a> for AxisStop {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let time = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let axis = *ptr.cast::<u32>();
            Ok(AxisStop { time, axis, })
        }
    }


    /// event, opcode `8`
    #[derive(Debug)]
    pub struct AxisDiscrete {
        pub axis: u32,
        pub discrete: i32,
    }

    impl EncodePayload for AxisDiscrete {
        const OPCODE: u16 = 8;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.axis);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.discrete);
        }
    }

    impl<'a> DecodePayload<'a> for AxisDiscrete {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let axis = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let discrete = *ptr.cast::<i32>();
            Ok(AxisDiscrete { axis, discrete, })
        }
    }


    /// event, opcode `9`
    #[derive(Debug)]
    pub struct AxisValue120 {
        pub axis: u32,
        pub value120: i32,
    }

    impl EncodePayload for AxisValue120 {
        const OPCODE: u16 = 9;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.axis);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.value120);
        }
    }

    impl<'a> DecodePayload<'a> for AxisValue120 {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let axis = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let value120 = *ptr.cast::<i32>();
            Ok(AxisValue120 { axis, value120, })
        }
    }


    /// bitfield: false
    pub enum AxisRelativeDirectionEnum {
        Identical = 0,
        Inverted = 1,
    }

    /// event, opcode `10`
    #[derive(Debug)]
    pub struct AxisRelativeDirection {
        pub axis: u32,
        pub direction: u32,
    }

    impl EncodePayload for AxisRelativeDirection {
        const OPCODE: u16 = 10;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.axis);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.direction);
        }
    }

    impl<'a> DecodePayload<'a> for AxisRelativeDirection {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let axis = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let direction = *ptr.cast::<u32>();
            Ok(AxisRelativeDirection { axis, direction, })
        }
    }

}


pub struct WlKeyboard;
pub mod wl_keyboard {
    use super::*;
    pub const VERSION: u32 = 10;
    pub const NAME_LEN: u16 = 12;
    pub static NEW_ID: [u8; 20] = *b"\x0c\x00\x00\x00wl_keyboard\0\x0a\x00\x00\x00";

    /// bitfield: false
    pub enum KeymapFormatEnum {
        NoKeymap = 0,
        XkbV1 = 1,
    }

    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Keymap {
        pub format: u32,
        pub size: u32,
    }

    impl EncodePayload for Keymap {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.format);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.size);
        }
    }

    impl<'a> DecodePayload<'a> for Keymap {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let format = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let size = *ptr.cast::<u32>();
            Ok(Keymap { format, size, })
        }
    }


    /// event, opcode `1`
    #[derive(Debug)]
    pub struct Enter<'a> {
        pub serial: u32,
        pub surface: u32,
        pub keys: &'a [u8],
    }

    impl<'a> EncodePayload for Enter<'a> {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            12 + self.keys.len() as u16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.surface);
            ptr = ptr.add(4);
            let len = self.keys.len() as u16;
            ptr.cast::<u32>().write(len as u32);
            ptr.add(4).copy_from_nonoverlapping(self.keys.as_ptr(), len as usize);
        }
    }

    impl<'a> DecodePayload<'a> for Enter<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 12 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 12;
            let keys_len = *ptr.add(8).cast::<u32>();
            let keys_pad_len = roundup4(keys_len as u16);
            if rem < keys_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= keys_pad_len;
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let surface = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let keys = slice::from_raw_parts(ptr.add(4), keys_len as usize);
            Ok(Enter { serial, surface, keys, })
        }
    }


    /// event, opcode `2`
    #[derive(Debug)]
    pub struct Leave {
        pub serial: u32,
        pub surface: u32,
    }

    impl EncodePayload for Leave {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.surface);
        }
    }

    impl<'a> DecodePayload<'a> for Leave {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let surface = *ptr.cast::<u32>();
            Ok(Leave { serial, surface, })
        }
    }


    /// bitfield: false
    pub enum KeyStateEnum {
        Released = 0,
        Pressed = 1,
        /// since: 10
        Repeated = 2,
    }

    /// event, opcode `3`
    #[derive(Debug)]
    pub struct Key {
        pub serial: u32,
        pub time: u32,
        pub key: u32,
        pub state: u32,
    }

    impl EncodePayload for Key {
        const OPCODE: u16 = 3;

        fn encoded_size(&self) -> u16 {
            16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.time);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.key);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.state);
        }
    }

    impl<'a> DecodePayload<'a> for Key {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let time = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let key = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let state = *ptr.cast::<u32>();
            Ok(Key { serial, time, key, state, })
        }
    }


    /// event, opcode `4`
    #[derive(Debug)]
    pub struct Modifiers {
        pub serial: u32,
        pub mods_depressed: u32,
        pub mods_latched: u32,
        pub mods_locked: u32,
        pub group: u32,
    }

    impl EncodePayload for Modifiers {
        const OPCODE: u16 = 4;

        fn encoded_size(&self) -> u16 {
            20
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.mods_depressed);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.mods_latched);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.mods_locked);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.group);
        }
    }

    impl<'a> DecodePayload<'a> for Modifiers {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let mods_depressed = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let mods_latched = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let mods_locked = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let group = *ptr.cast::<u32>();
            Ok(Modifiers { serial, mods_depressed, mods_latched, mods_locked, group, })
        }
    }


    /// request, opcode `0`, type "destructor"
    #[derive(Debug)]
    pub struct Release {
    }

    impl EncodePayload for Release {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Release {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Release { })
        }
    }


    /// event, opcode `5`
    #[derive(Debug)]
    pub struct RepeatInfo {
        pub rate: i32,
        pub delay: i32,
    }

    impl EncodePayload for RepeatInfo {
        const OPCODE: u16 = 5;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.rate);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.delay);
        }
    }

    impl<'a> DecodePayload<'a> for RepeatInfo {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let rate = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let delay = *ptr.cast::<i32>();
            Ok(RepeatInfo { rate, delay, })
        }
    }

}


pub struct WlTouch;
pub mod wl_touch {
    use super::*;
    pub const VERSION: u32 = 10;
    pub const NAME_LEN: u16 = 9;
    pub static NEW_ID: [u8; 20] = *b"\x09\x00\x00\x00wl_touch\0\0\0\0\x0a\x00\x00\x00";

    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Down {
        pub serial: u32,
        pub time: u32,
        pub surface: u32,
        pub id: i32,
        pub x: f32,
        pub y: f32,
    }

    impl EncodePayload for Down {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            24
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.time);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.surface);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.id);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.x * 256.0).round() as i32);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.y * 256.0).round() as i32);
        }
    }

    impl<'a> DecodePayload<'a> for Down {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let time = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let surface = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let id = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let x = *ptr.cast::<i32>() as f32 / 256.0;
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>() as f32 / 256.0;
            Ok(Down { serial, time, surface, id, x, y, })
        }
    }


    /// event, opcode `1`
    #[derive(Debug)]
    pub struct Up {
        pub serial: u32,
        pub time: u32,
        pub id: i32,
    }

    impl EncodePayload for Up {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            12
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.serial);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.time);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.id);
        }
    }

    impl<'a> DecodePayload<'a> for Up {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let serial = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let time = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let id = *ptr.cast::<i32>();
            Ok(Up { serial, time, id, })
        }
    }


    /// event, opcode `2`
    #[derive(Debug)]
    pub struct Motion {
        pub time: u32,
        pub id: i32,
        pub x: f32,
        pub y: f32,
    }

    impl EncodePayload for Motion {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.time);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.id);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.x * 256.0).round() as i32);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.y * 256.0).round() as i32);
        }
    }

    impl<'a> DecodePayload<'a> for Motion {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let time = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let id = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let x = *ptr.cast::<i32>() as f32 / 256.0;
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>() as f32 / 256.0;
            Ok(Motion { time, id, x, y, })
        }
    }


    /// event, opcode `3`
    #[derive(Debug)]
    pub struct Frame {
    }

    impl EncodePayload for Frame {
        const OPCODE: u16 = 3;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Frame {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Frame { })
        }
    }


    /// event, opcode `4`
    #[derive(Debug)]
    pub struct Cancel {
    }

    impl EncodePayload for Cancel {
        const OPCODE: u16 = 4;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Cancel {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Cancel { })
        }
    }


    /// request, opcode `0`, type "destructor"
    #[derive(Debug)]
    pub struct Release {
    }

    impl EncodePayload for Release {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Release {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Release { })
        }
    }


    /// event, opcode `5`
    #[derive(Debug)]
    pub struct Shape {
        pub id: i32,
        pub major: f32,
        pub minor: f32,
    }

    impl EncodePayload for Shape {
        const OPCODE: u16 = 5;

        fn encoded_size(&self) -> u16 {
            12
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.id);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.major * 256.0).round() as i32);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.minor * 256.0).round() as i32);
        }
    }

    impl<'a> DecodePayload<'a> for Shape {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let major = *ptr.cast::<i32>() as f32 / 256.0;
            ptr = ptr.add(4);
            let minor = *ptr.cast::<i32>() as f32 / 256.0;
            Ok(Shape { id, major, minor, })
        }
    }


    /// event, opcode `6`
    #[derive(Debug)]
    pub struct Orientation {
        pub id: i32,
        pub orientation: f32,
    }

    impl EncodePayload for Orientation {
        const OPCODE: u16 = 6;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.id);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write((self.orientation * 256.0).round() as i32);
        }
    }

    impl<'a> DecodePayload<'a> for Orientation {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let orientation = *ptr.cast::<i32>() as f32 / 256.0;
            Ok(Orientation { id, orientation, })
        }
    }

}


pub struct WlOutput;
pub mod wl_output {
    use super::*;
    pub const VERSION: u32 = 4;
    pub const NAME_LEN: u16 = 10;
    pub static NEW_ID: [u8; 20] = *b"\x0a\x00\x00\x00wl_output\0\0\0\x04\x00\x00\x00";

    /// bitfield: false
    pub enum SubpixelEnum {
        Unknown = 0,
        None = 1,
        HorizontalRgb = 2,
        HorizontalBgr = 3,
        VerticalRgb = 4,
        VerticalBgr = 5,
    }

    /// bitfield: false
    pub enum TransformEnum {
        Normal = 0,
        _90 = 1,
        _180 = 2,
        _270 = 3,
        Flipped = 4,
        Flipped90 = 5,
        Flipped180 = 6,
        Flipped270 = 7,
    }

    /// event, opcode `0`
    #[derive(Debug)]
    pub struct Geometry<'a> {
        pub x: i32,
        pub y: i32,
        pub physical_width: i32,
        pub physical_height: i32,
        pub subpixel: i32,
        pub make: &'a str,
        pub model: &'a str,
        pub transform: i32,
    }

    impl<'a> EncodePayload for Geometry<'a> {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            32 + self.make.len() as u16 + 1 + self.model.len() as u16 + 1
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.x);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.y);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.physical_width);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.physical_height);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.subpixel);
            ptr = ptr.add(4);
            let len = self.make.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.make.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
            ptr = ptr.add((4 + roundup4(len + 1)) as usize);
            let len = self.model.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.model.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
            ptr = ptr.add((4 + roundup4(len + 1)) as usize);
            ptr.cast::<i32>().write(self.transform);
        }
    }

    impl<'a> DecodePayload<'a> for Geometry<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 24 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 24;
            let make_len = *ptr.add(20).cast::<u32>();
            let make_pad_len = roundup4(make_len as u16);
            if rem < make_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= make_pad_len;
            if rem < 4 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 4;
            let model_len = *ptr.add(0).cast::<u32>();
            let model_pad_len = roundup4(model_len as u16);
            if rem < model_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= model_pad_len;
            let x = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let physical_width = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let physical_height = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let subpixel = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let [make @ .., 0] = slice::from_raw_parts(ptr.add(4), make_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(make) = str::from_utf8(make) else {
                return Err(DecodeError::NonUtf8);
            };
            ptr = ptr.add((4 + make_pad_len) as usize);
            let [model @ .., 0] = slice::from_raw_parts(ptr.add(4), model_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(model) = str::from_utf8(model) else {
                return Err(DecodeError::NonUtf8);
            };
            ptr = ptr.add((4 + model_pad_len) as usize);
            let transform = *ptr.cast::<i32>();
            Ok(Geometry { x, y, physical_width, physical_height, subpixel, make, model, transform, })
        }
    }


    /// bitfield: true
    pub enum ModeEnum {
        Current = 0x1,
        Preferred = 0x2,
    }

    /// event, opcode `1`
    #[derive(Debug)]
    pub struct Mode {
        pub flags: u32,
        pub width: i32,
        pub height: i32,
        pub refresh: i32,
    }

    impl EncodePayload for Mode {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.flags);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.width);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.height);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.refresh);
        }
    }

    impl<'a> DecodePayload<'a> for Mode {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let flags = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let width = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let height = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let refresh = *ptr.cast::<i32>();
            Ok(Mode { flags, width, height, refresh, })
        }
    }


    /// event, opcode `2`
    #[derive(Debug)]
    pub struct Done {
    }

    impl EncodePayload for Done {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Done {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Done { })
        }
    }


    /// event, opcode `3`
    #[derive(Debug)]
    pub struct Scale {
        pub factor: i32,
    }

    impl EncodePayload for Scale {
        const OPCODE: u16 = 3;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.factor);
        }
    }

    impl<'a> DecodePayload<'a> for Scale {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let factor = *ptr.cast::<i32>();
            Ok(Scale { factor, })
        }
    }


    /// request, opcode `0`, type "destructor"
    #[derive(Debug)]
    pub struct Release {
    }

    impl EncodePayload for Release {
        const OPCODE: u16 = 0;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for Release {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(Release { })
        }
    }


    /// event, opcode `4`
    #[derive(Debug)]
    pub struct Name<'a> {
        pub name: &'a str,
    }

    impl<'a> EncodePayload for Name<'a> {
        const OPCODE: u16 = 4;

        fn encoded_size(&self) -> u16 {
            4 + self.name.len() as u16 + 1
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            let len = self.name.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.name.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
        }
    }

    impl<'a> DecodePayload<'a> for Name<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 4 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 4;
            let name_len = *ptr.add(0).cast::<u32>();
            let name_pad_len = roundup4(name_len as u16);
            if rem < name_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= name_pad_len;
            let [name @ .., 0] = slice::from_raw_parts(ptr.add(4), name_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(name) = str::from_utf8(name) else {
                return Err(DecodeError::NonUtf8);
            };
            Ok(Name { name, })
        }
    }


    /// event, opcode `5`
    #[derive(Debug)]
    pub struct Description<'a> {
        pub description: &'a str,
    }

    impl<'a> EncodePayload for Description<'a> {
        const OPCODE: u16 = 5;

        fn encoded_size(&self) -> u16 {
            4 + self.description.len() as u16 + 1
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            let len = self.description.len() as u16;
            ptr.cast::<u32>().write((len + 1) as u32);
            ptr.add(4).copy_from_nonoverlapping(self.description.as_ptr(), len as usize);
            ptr.add((4 + len) as usize).write(0);
        }
    }

    impl<'a> DecodePayload<'a> for Description<'a> {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            let mut rem = *ptr.add(6).cast::<u16>();
            ptr = ptr.add(8);
            if rem < 4 {
                return Err(DecodeError::Insufficient);
            }
            rem -= 4;
            let description_len = *ptr.add(0).cast::<u32>();
            let description_pad_len = roundup4(description_len as u16);
            if rem < description_pad_len {
                return Err(DecodeError::Insufficient);
            }
            rem -= description_pad_len;
            let [description @ .., 0] = slice::from_raw_parts(ptr.add(4), description_len as usize) else {
                return Err(DecodeError::NoNullTerm);
            };
            let Ok(description) = str::from_utf8(description) else {
                return Err(DecodeError::NonUtf8);
            };
            Ok(Description { description, })
        }
    }

}


pub struct WlRegion;
pub mod wl_region {
    use super::*;
    pub const VERSION: u32 = 7;
    pub const NAME_LEN: u16 = 10;
    pub static NEW_ID: [u8; 20] = *b"\x0a\x00\x00\x00wl_region\0\0\0\x07\x00\x00\x00";

    /// request, opcode `0`, type "destructor"
    #[derive(Debug)]
    pub struct Destroy {
    }

    impl EncodePayload for Destroy {
        const OPCODE: u16 = 0;

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


    /// request, opcode `1`
    #[derive(Debug)]
    pub struct Add {
        pub x: i32,
        pub y: i32,
        pub width: i32,
        pub height: i32,
    }

    impl EncodePayload for Add {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.x);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.y);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.width);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.height);
        }
    }

    impl<'a> DecodePayload<'a> for Add {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let x = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let width = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let height = *ptr.cast::<i32>();
            Ok(Add { x, y, width, height, })
        }
    }


    /// request, opcode `2`
    #[derive(Debug)]
    pub struct Subtract {
        pub x: i32,
        pub y: i32,
        pub width: i32,
        pub height: i32,
    }

    impl EncodePayload for Subtract {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            16
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.x);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.y);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.width);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.height);
        }
    }

    impl<'a> DecodePayload<'a> for Subtract {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let x = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let width = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let height = *ptr.cast::<i32>();
            Ok(Subtract { x, y, width, height, })
        }
    }

}


pub struct WlSubcompositor;
pub mod wl_subcompositor {
    use super::*;
    pub const VERSION: u32 = 1;
    pub const NAME_LEN: u16 = 17;
    pub static NEW_ID: [u8; 28] = *b"\x11\x00\x00\x00wl_subcompositor\0\0\0\0\x01\x00\x00\x00";

    /// request, opcode `0`, type "destructor"
    #[derive(Debug)]
    pub struct Destroy {
    }

    impl EncodePayload for Destroy {
        const OPCODE: u16 = 0;

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


    /// bitfield: false
    pub enum ErrorEnum {
        BadSurface = 0,
        BadParent = 1,
    }

    /// request, opcode `1`
    #[derive(Debug)]
    pub struct GetSubsurface {
        pub id: u32,
        pub surface: u32,
        pub parent: u32,
    }

    impl EncodePayload for GetSubsurface {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            12
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.id);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.surface);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.parent);
        }
    }

    impl<'a> DecodePayload<'a> for GetSubsurface {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let id = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let surface = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let parent = *ptr.cast::<u32>();
            Ok(GetSubsurface { id, surface, parent, })
        }
    }

}


pub struct WlSubsurface;
pub mod wl_subsurface {
    use super::*;
    pub const VERSION: u32 = 1;
    pub const NAME_LEN: u16 = 14;
    pub static NEW_ID: [u8; 24] = *b"\x0e\x00\x00\x00wl_subsurface\0\0\0\x01\x00\x00\x00";

    /// request, opcode `0`, type "destructor"
    #[derive(Debug)]
    pub struct Destroy {
    }

    impl EncodePayload for Destroy {
        const OPCODE: u16 = 0;

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


    /// bitfield: false
    pub enum ErrorEnum {
        BadSurface = 0,
    }

    /// request, opcode `1`
    #[derive(Debug)]
    pub struct SetPosition {
        pub x: i32,
        pub y: i32,
    }

    impl EncodePayload for SetPosition {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<i32>().write(self.x);
            ptr = ptr.add(4);
            ptr.cast::<i32>().write(self.y);
        }
    }

    impl<'a> DecodePayload<'a> for SetPosition {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let x = *ptr.cast::<i32>();
            ptr = ptr.add(4);
            let y = *ptr.cast::<i32>();
            Ok(SetPosition { x, y, })
        }
    }


    /// request, opcode `2`
    #[derive(Debug)]
    pub struct PlaceAbove {
        pub sibling: u32,
    }

    impl EncodePayload for PlaceAbove {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.sibling);
        }
    }

    impl<'a> DecodePayload<'a> for PlaceAbove {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let sibling = *ptr.cast::<u32>();
            Ok(PlaceAbove { sibling, })
        }
    }


    /// request, opcode `3`
    #[derive(Debug)]
    pub struct PlaceBelow {
        pub sibling: u32,
    }

    impl EncodePayload for PlaceBelow {
        const OPCODE: u16 = 3;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.sibling);
        }
    }

    impl<'a> DecodePayload<'a> for PlaceBelow {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let sibling = *ptr.cast::<u32>();
            Ok(PlaceBelow { sibling, })
        }
    }


    /// request, opcode `4`
    #[derive(Debug)]
    pub struct SetSync {
    }

    impl EncodePayload for SetSync {
        const OPCODE: u16 = 4;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for SetSync {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(SetSync { })
        }
    }


    /// request, opcode `5`
    #[derive(Debug)]
    pub struct SetDesync {
    }

    impl EncodePayload for SetDesync {
        const OPCODE: u16 = 5;

        fn encoded_size(&self) -> u16 {
            0
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
        }
    }

    impl<'a> DecodePayload<'a> for SetDesync {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            Ok(SetDesync { })
        }
    }

}


pub struct WlFixes;
pub mod wl_fixes {
    use super::*;
    pub const VERSION: u32 = 2;
    pub const NAME_LEN: u16 = 9;
    pub static NEW_ID: [u8; 20] = *b"\x09\x00\x00\x00wl_fixes\0\0\0\0\x02\x00\x00\x00";

    /// bitfield: false
    pub enum ErrorEnum {
        InvalidAckRemove = 0,
    }

    /// request, opcode `0`, type "destructor"
    #[derive(Debug)]
    pub struct Destroy {
    }

    impl EncodePayload for Destroy {
        const OPCODE: u16 = 0;

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


    /// request, opcode `1`
    #[derive(Debug)]
    pub struct DestroyRegistry {
        pub registry: u32,
    }

    impl EncodePayload for DestroyRegistry {
        const OPCODE: u16 = 1;

        fn encoded_size(&self) -> u16 {
            4
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.registry);
        }
    }

    impl<'a> DecodePayload<'a> for DestroyRegistry {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let registry = *ptr.cast::<u32>();
            Ok(DestroyRegistry { registry, })
        }
    }


    /// request, opcode `2`
    #[derive(Debug)]
    pub struct AckGlobalRemove {
        pub registry: u32,
        pub name: u32,
    }

    impl EncodePayload for AckGlobalRemove {
        const OPCODE: u16 = 2;

        fn encoded_size(&self) -> u16 {
            8
        }

        unsafe fn encode_raw(&self, mut ptr: *mut u8) {
            ptr.cast::<u32>().write(self.registry);
            ptr = ptr.add(4);
            ptr.cast::<u32>().write(self.name);
        }
    }

    impl<'a> DecodePayload<'a> for AckGlobalRemove {
        unsafe fn decode_raw(mut ptr: *const u8) -> Result<Self, DecodeError> {
            ptr = ptr.add(8);
            let registry = *ptr.cast::<u32>();
            ptr = ptr.add(4);
            let name = *ptr.cast::<u32>();
            Ok(AckGlobalRemove { registry, name, })
        }
    }

}
