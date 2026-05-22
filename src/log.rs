#![allow(static_mut_refs)]

// ===== Logger =====

#[derive(Clone, Copy)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Level {
    pub const fn to_bytes(self) -> [u8; LEVEL_PAD] {
        match self {
            Level::Error => *b"\x1b[91mERROR\x1b[39m",
            Level::Warn => *b"\x1b[93mWARN \x1b[39m",
            Level::Info => *b"\x1b[92mINFO \x1b[39m",
            Level::Debug => *b"\x1b[94mDEBUG\x1b[39m",
            Level::Trace => *b"\x1b[90mTRACE\x1b[39m",
        }
    }
}

type LogBuffer = Vec<u8>;

static mut BUFFER: LogBuffer = LogBuffer::new();

fn buffer<'a>() -> &'a mut Vec<u8> {
    unsafe { &mut BUFFER }
}

const LEVEL_PAD: usize = 15;
const NAME_PAD: usize = 6;
const HEADER_LEN: usize = LEVEL_PAD + NAME_PAD + 3/*spaces*/;

#[doc(hidden)]
pub const fn build_prefix(level: Level, name: [u8; NAME_PAD]) -> [u8; HEADER_LEN] {
    let mut bytes = [b' '; HEADER_LEN];
    let ptr = bytes.as_mut_ptr();
    unsafe {
        ptr.copy_from_nonoverlapping(level.to_bytes().as_ptr(), LEVEL_PAD);
        ptr.add(LEVEL_PAD + 1).copy_from_nonoverlapping(name.as_ptr(), NAME_PAD);
    }
    bytes
}

#[doc(hidden)] // gaslighted the autocomplete with `log::info`
pub fn init() -> LogGuard {
    *buffer() = LogBuffer::with_capacity(1024);
    LogGuard
}

#[doc(hidden)]
pub fn log_me(prefix: [u8; HEADER_LEN], args: std::fmt::Arguments) {
    use std::io::Write;
    let b = buffer();
    b.reserve(prefix.len());
    let _ = b.write_all(&prefix);
    let _ = b.write_fmt(args);
    let _ = b.write(b"\n");
}

pub fn flush() {
    let b = buffer();
    if b.is_empty() {
        return;
    }
    let mut stdout = std::io::stdout().lock();
    let _ = std::io::Write::write_all(&mut stdout, b);
    let _ = std::io::Write::flush(&mut stdout);
    b.clear();
}

#[doc(hidden)]
pub struct LogGuard;

impl Drop for LogGuard {
    fn drop(&mut self) {
        flush();
        unsafe { std::ptr::drop_in_place(buffer()) }
    }
}

// ===== macros =====

macro_rules! names {
    (client) => {*b"client"};
    (epoll ) => {*b"epoll "};
    (sigfd ) => {*b"sigfd "};
    (listener) => {*b"listen"};
    ($($tt:tt)*) => {compile_error!(stringify!(stringify!($($tt)*)))};
}

macro_rules! log {
    ($l:ident, $s:ident, $($tt:tt)*) => {
        crate::log::log_me(
            const { crate::log::build_prefix(crate::log::Level::$l, crate::log::names!($s)) },
            format_args!($($tt)*)
        )
    };
}

macro_rules! error { ($($tt:tt)*) => { crate::log::log!(Error, $($tt)*) }; }
macro_rules! _warn { ($($tt:tt)*) => { crate::log::log!(Warn, $($tt)*) }; }
macro_rules! info  { ($($tt:tt)*) => { crate::log::log!(Info, $($tt)*) }; }
macro_rules! debug { ($($tt:tt)*) => { crate::log::log!(Debug, $($tt)*) }; }
macro_rules! trace { ($($tt:tt)*) => { crate::log::log!(Trace, $($tt)*) }; }

pub(crate) use {names, log, error, _warn as warn, info, debug, trace};
