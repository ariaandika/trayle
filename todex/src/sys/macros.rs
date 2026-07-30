macro_rules! simple_ffi {
    ($impl:ident Debug for $me:ident) => {
        $impl std::fmt::Debug for $me {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($me)).finish_non_exhaustive()
            }
        }
    };
    ($impl:ident Clone for $me:ident::$fn:ident) => {
        $impl Clone for $me {
            #[inline]
            fn clone(&self) -> Self {
                Self(unsafe { $fn(self.0) })
            }
        }
    };
    ($impl:ident Drop for $me:ident::$fn:ident) => {
        $impl Drop for $me {
            #[inline]
            fn drop(&mut self) {
                unsafe { $fn(self.0) };
            }
        }
    };
}
pub(crate) use simple_ffi;
