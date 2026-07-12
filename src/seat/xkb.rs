use std::ffi::{CStr, c_char};
use std::ptr::NonNull;
use todex::alloc;
use todex::sys::errno::Errno;

// https://github.com/xkbcommon/libxkbcommon/blob/master/include/xkbcommon/xkbcommon.h

#[expect(non_camel_case_types)]
mod xkb {
    use std::ffi::c_char;

    pub type context_flags = u32;
    pub type keymap_format = u32;
    pub type keymap_compile_flags = u32;

    pub const CONTEXT_NO_FLAGS: u32 = 0;
    pub const KEYMAP_COMPILE_NO_FLAGS: u32 = 0;

    pub const KEYMAP_FORMAT_TEXT_V1: u32 = 1;
    pub const KEYMAP_FORMAT_TEXT_V2: u32 = 2;

    pub enum context {}

    /// [https://xkbcommon.org/doc/current/group__keymap.html#gab0f75d6cc5773e5dd404e2c3f61366a3].
    pub enum keymap {}

    /// Names to compile a keymap with, also known as [RMLVO].
    ///
    /// The names are the common configuration values by which a user picks a keymap.
    ///
    /// If the entire struct is NULL, then each field is taken to be NULL. You should prefer passing
    /// NULL instead of choosing your own defaults.
    ///
    /// [RMLVO]: https://xkbcommon.org/doc/current/xkb-intro.html#RMLVO-intro
    #[repr(C)]
    pub struct rule_names {
        pub rules: *const c_char,
        pub model: *const c_char,
        pub layout: *const c_char,
        pub variant: *const c_char,
        pub options: *const c_char,
    }
}

#[link(name = "xkbcommon")]
unsafe extern "C" {
    fn xkb_context_new(flags: xkb::context_flags) -> *mut xkb::context;
    fn xkb_context_unref(context: *mut xkb::context);

    fn xkb_keymap_new_from_names2(
        context: *mut xkb::context,
        names: *const xkb::rule_names,
        format: xkb::keymap_format,
        flags: xkb::keymap_compile_flags,
    ) -> *mut xkb::keymap;
    fn xkb_keymap_unref(keymap: *mut xkb::keymap);

    /// The returned string is dynamically allocated and should be freed by the caller.
    fn xkb_keymap_get_as_string(
        keymap: *mut xkb::keymap,
        format: xkb::keymap_format,
    ) -> *const c_char;
}

// ===== Xkb =====

pub struct Xkb {
    cx: NonNull<xkb::context>,
    keymap: NonNull<xkb::keymap>,
    keymap_string: NonNull<i8>,
}

impl Drop for Xkb {
    fn drop(&mut self) {
        unsafe {
            xkb_keymap_unref(self.keymap.as_ptr());
            xkb_context_unref(self.cx.as_ptr());
            alloc::deallocate(self.keymap_string);
        }
    }
}

impl Xkb {
    pub fn new() -> Self {
        let cx = unsafe { xkb_context_new(xkb::CONTEXT_NO_FLAGS) };
        let Some(cx) = NonNull::new(cx) else {
            panic!("cannot create xkb: {}", Errno::get())
        };

        let keymap = unsafe {
            xkb_keymap_new_from_names2(
                cx.as_ptr(),
                0 as _,
                xkb::KEYMAP_FORMAT_TEXT_V2,
                xkb::KEYMAP_COMPILE_NO_FLAGS,
            )
        };
        let Some(keymap) = NonNull::new(keymap) else {
            panic!("cannot create keymap: {}", Errno::get())
        };

        let keymap_string = unsafe {
            let string = xkb_keymap_get_as_string(keymap.as_ptr(), xkb::KEYMAP_FORMAT_TEXT_V1);
            match NonNull::new(string.cast_mut()) {
                Some(ok) => ok,
                None => panic!("cannot get keymap string"),
            }
        };

        Self {
            cx,
            keymap,
            keymap_string,
        }
    }

    pub const fn keymap_str(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.keymap_string.as_ptr()) }
    }
}
