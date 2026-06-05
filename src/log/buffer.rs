pub type LogBuffer = Vec<u8>;

static mut BUFFER: LogBuffer = LogBuffer::new();

/// Get the global logger buffer.
pub fn get_mut<'a>() -> &'a mut LogBuffer {
    unsafe { &mut BUFFER }
}
