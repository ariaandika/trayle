#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    pub(crate) const fn to_fmt(self) -> &'static str {
        match self {
            Level::Trace => "\x1b[90mTRACE\x1b[39m",
            Level::Debug => "\x1b[94mDEBUG\x1b[39m",
            Level::Info => "\x1b[92mINFO \x1b[39m",
            Level::Warn => "\x1b[93mWARN \x1b[39m",
            Level::Error => "\x1b[91mERROR\x1b[39m",
        }
    }
}
