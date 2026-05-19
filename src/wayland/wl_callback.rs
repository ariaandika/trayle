use crate::wayland::{PtrWrite, Write};

/// Send `wl_callback::done` event.
pub fn done(object_id: u32, callback_data: u32, mut writer: impl Write) {
    // opcode `0`, len `12`
    const HEADER_SUFFIX: *const u8 = [0, 0, 12, 0].as_ptr();
    unsafe {
        let ptr = writer.spare(12);
        ptr.put(object_id);
        ptr.add(4).copy_from_nonoverlapping(HEADER_SUFFIX, 4);
        ptr.add(8).put(callback_data);
    }
}
