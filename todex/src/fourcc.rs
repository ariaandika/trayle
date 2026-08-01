#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Format(u32);

const fn fc(a: char, b: char, c: char, d: char) -> Format {
    Format(a as u32 | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24))
}

impl Format {
    pub const XRGB8888: Format = fc('X', 'R', '2', '4');
    pub const ARGB8888: Format = fc('A', 'R', '2', '4');
}

impl From<Format> for u32 {
    #[inline]
    fn from(value: Format) -> Self {
        value.0
    }
}

impl std::fmt::Debug for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0.to_ne_bytes();
        match str::from_utf8(&bytes) {
            Ok(ok) => f.write_str(ok),
            Err(_) => f.debug_tuple("Format").field(&self.0).finish(),
        }
    }
}
