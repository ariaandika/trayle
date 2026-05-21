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
    fn display(self) -> &'static str {
        match self {
            Level::Error => "\x1b[91mERR\x1b[39m",
            Level::Warn => "\x1b[93mWRN\x1b[39m",
            Level::Info => "\x1b[92mINF\x1b[39m",
            Level::Debug => "\x1b[94mDBG\x1b[39m",
            Level::Trace => "\x1b[90mTRC\x1b[39m",
        }
    }
}

thread_local! {
    static BUFFER: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub fn init() {
    BUFFER.with_borrow_mut(|b|{
        *b = Vec::with_capacity(1024);
    });
}

pub fn log_me(level: Level, name: &str, args: std::fmt::Arguments) {
    use std::io::Write;
    BUFFER.with_borrow_mut(|b|{
        b.reserve(3 + name.len());
        let _ = writeln!(b, "{}:{name} {args}", level.display());
    })
}

pub fn flush() {
    BUFFER.with_borrow_mut(flush_in)
}

fn flush_in(b: &mut Vec<u8>) {
    if b.is_empty() {
        return;
    }
    let mut stdout = std::io::stdout().lock();
    let _ = std::io::Write::write_all(&mut stdout, b);
    let _ = std::io::Write::flush(&mut stdout);
    b.clear();
}

// ===== macros =====

macro_rules! log {
    ($l:ident, type $t:ty, $($tt:tt)*) => {
        crate::log::log_me(
            crate::log::Level::$l,
            <$t as crate::log::Subject>::NAME,
            format_args!($($tt)*)
        )
    };
    ($l:ident, $s:expr, $($tt:tt)*) => {
        crate::log::log_me(
            crate::log::Level::$l,
            {use crate::log::Subject;$s.name()},
            format_args!($($tt)*)
        )
    };
}

macro_rules! error { ($($tt:tt)*) => { crate::log::log!(Error, $($tt)*) }; }
macro_rules! _warn { ($($tt:tt)*) => { crate::log::log!(Warn, $($tt)*) }; }
macro_rules! info  { ($($tt:tt)*) => { crate::log::log!(Info, $($tt)*) }; }
macro_rules! debug { ($($tt:tt)*) => { crate::log::log!(Debug, $($tt)*) }; }
macro_rules! trace { ($($tt:tt)*) => { crate::log::log!(Trace, $($tt)*) }; }

pub(crate) use {log, error, _warn as warn, info, debug, trace};

// ===== traits =====

pub trait Subject {
    const NAME: &str;

    fn name(&self) -> &str {
        Self::NAME
    }
}

macro_rules! impl_subject {
    ($t:ty, $n:literal) => {
        impl Subject for $t {
            const NAME: &'static str = $n;
        }
    };
}

impl_subject!(crate::epoll::Epoll, "EPOLL");
impl_subject!(crate::sigfd::Sigfd, "SIGFD");
impl_subject!(crate::clients::ClientMut<'_>, "CLIEN");
impl_subject!(crate::clients::Client, "CLIEN");
// impl_subject!(crate::clients::Clients, "CLIENTS");

impl<S: Subject> Subject for &mut S {
    const NAME: &'static str = S::NAME;
}

// ===== Guards =====

// pub fn flush_guard() -> FlushGuard {
//     FlushGuard { _p: () }
// }
//
// pub struct FlushGuard {
//     _p: ()
// }
//
// impl Drop for FlushGuard {
//     fn drop(&mut self) {
//         flush();
//     }
// }
