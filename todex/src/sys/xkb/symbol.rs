use std::num::NonZeroU32;

/// XKB keysym.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct KeySym(NonZeroU32);

macro_rules! def {
    ($(#define $name:ident $val:literal $alias:ident)*) => {
        impl KeySym {$(
            #[doc = stringify!($name)]
            pub const $alias: Self = Self(NonZeroU32::new($val).unwrap());
        )*}
    };
}

def! {
#define XKB_KEY_Escape                        0xff1b  ESCAPE
}
