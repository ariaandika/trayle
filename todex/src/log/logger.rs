use std::io::Write;

use crate::log::Level;
use crate::log::buffer::{self, LogBuffer};

static mut LEVEL: Level = if cfg!(debug_assertions) {
    Level::Debug
} else {
    Level::Info
};

pub fn get_level() -> Level {
    unsafe { LEVEL }
}

pub fn set_level(level: Level) {
    unsafe { LEVEL = level };
}

#[must_use = "returns a guard for the global logger lifetime"]
pub fn init() -> LogGuard {
    *buffer::get_mut() = LogBuffer::with_capacity(512);
    LogGuard
}

pub fn log_me(level: Level, target: impl std::fmt::Display, args: std::fmt::Arguments) {
    let b = buffer::get_mut();
    let _ = writeln!(b, "{} {target}{args}", level.to_fmt(),);
}

#[allow(unused, reason = "debugging")]
pub fn lossy(bytes: &[u8]) {
    let b = buffer::get_mut();
    b.extend_from_slice(Level::Debug.to_fmt().as_bytes());
    b.push(b' ');
    for byte in bytes {
        if byte.is_ascii_alphabetic() || *byte == b'_' {
            b.push(*byte)
        } else {
            let _ = write!(b, "\\x{byte:0>2X}");
        }
    }
    b.push(b'\n');
}

pub fn flush() {
    let b = buffer::get_mut();
    if b.is_empty() {
        return;
    }
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(b);
    let _ = stdout.flush();
    b.clear();
}

pub struct LogGuard;

impl Drop for LogGuard {
    fn drop(&mut self) {
        flush();
        unsafe { std::ptr::drop_in_place(buffer::get_mut()) }
    }
}

// ===== macros =====

#[macro_export]
macro_rules! log {
    ($l:expr, target: $t:expr, $($tt:tt)*) => {{
        if $l >= $crate::log::logger::get_level() {
            $crate::log::logger::log_me($l, format_args!("{} ", $t), format_args!($($tt)*));
        }
    }};
    ($l:expr, $($tt:tt)*) => {{
        if $l >= $crate::log::logger::get_level() {
            $crate::log::logger::log_me($l, "", format_args!($($tt)*));
        }
    }};
}

#[macro_export] macro_rules! error { ($($tt:tt)*) => { $crate::log::log!($crate::log::Level::Error, $($tt)*) } }
#[macro_export] macro_rules! _warn { ($($tt:tt)*) => { $crate::log::log!($crate::log::Level::Warn, $($tt)*) }; }
#[macro_export] macro_rules! info { ($($tt:tt)*) => { $crate::log::log!($crate::log::Level::Info, $($tt)*) }; }
#[macro_export] macro_rules! debug { ($($tt:tt)*) => { $crate::log::log!($crate::log::Level::Debug, $($tt)*) }; }
#[macro_export] macro_rules! trace { ($($tt:tt)*) => { $crate::log::log!($crate::log::Level::Trace, $($tt)*) }; }

pub use {log, error, _warn as warn, info, debug, trace};
