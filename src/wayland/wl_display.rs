use crate::wayland::{Write, id::Id};

macro_rules! roundup4 {
    ($e:expr) => {
        ($e + 3) & (u16::MAX << 2)
    };
}

#[derive(Debug)]
pub struct WlDisplay {

}

impl Default for WlDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl WlDisplay {
    pub fn new() -> Self {
        Self { }
    }

    /// Send `wl_display::error` event.
    pub fn error(&self, object_id: Id, code: u32, message: &str, mut writer: impl Write) {
        let msg_len = message.len() as u16;
        let len = 20 + roundup4!(msg_len + 1);
        // SAFETY: initialization in `error_inner`
        let ptr = unsafe { writer.spare(len as usize) };
        self.error_inner(object_id, code, message.as_ptr(), msg_len, len, ptr);
    }

    fn error_inner(&self, object_id: Id, code: u32, msg: *const u8, msg_len: u16, len: u16, ptr: *mut u8) {
        // object_id 1, opcode 0, len placeholder
        const HEADER: u64 = 1;
        unsafe {
            ptr.cast::<u64>().write(HEADER);
            ptr.add(6).cast::<u16>().write(len);
            ptr.add(8).cast::<Id>().write(object_id);
            ptr.add(12).cast::<u32>().write(code);
            ptr.add(16).cast::<u32>().write((msg_len + 1) as u32);
            ptr.add(20).copy_from_nonoverlapping(msg, msg_len as usize);
            ptr.add((20 + msg_len) as usize).write(0);
        }
    }
}

