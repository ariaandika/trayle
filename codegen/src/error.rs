use std::cell::RefCell;

use crate::str::Str;

thread_local! {
    static SPANS: RefCell<Vec<Span>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone)]
pub struct Span {
    value: Str,
}

pub struct SpanGuard {
    _p: ()
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        SPANS.with_borrow_mut(|spans|{
            spans.pop();
        });
    }
}

pub fn span<T: Into<Str>>(value: T) -> SpanGuard {
    let value = value.into();
    SPANS.with_borrow_mut(move |spans|{
        spans.push(Span { value });
    });
    SpanGuard {
        _p: ()
    }
}

// ===== Error =====

macro_rules! err {
    ($($tt:tt)*) => {
        Err(crate::error::Error::new(format!($($tt)*)))
    };
}

pub(crate) use err;

pub struct Error {
    repr: Box<ErrorRepr>,
}

impl Error {
    pub fn new(value: impl Into<Str>) -> Self {
        SPANS.with_borrow_mut(|spans|{
            Self {
                repr: Box::new(ErrorRepr {
                    value: value.into(),
                    spans: spans.clone().into_boxed_slice(),
                }),
            }
        })
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.repr.spans.is_empty() {
            write!(f, "in")?;
            for (i, span) in self.repr.spans.iter().enumerate() {
                if i != 0 {
                    write!(f, ",")?;
                }
                write!(f, " {}", span.value)?;
            }
            write!(f, ": ")?;
        }
        write!(f, "{}", self.repr.value)
    }
}

struct ErrorRepr {
    value: Str,
    spans: Box<[Span]>,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self {
            repr: Box::new(ErrorRepr {
                value: value.to_string().into(),
                spans: Box::new([]),
            }),
        }
    }
}

// ===== trait =====

pub trait ErrorExt<T> {
    fn cx(self, msg: impl Into<Str>) -> Result<T, Error>;
}

impl<T> ErrorExt<T> for Option<T> {
    fn cx(self, msg: impl Into<Str>) -> Result<T, Error> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Error::new(msg)),
        }
    }
}

impl<T, E> ErrorExt<T> for Result<T, E> {
    fn cx(self, msg: impl Into<Str>) -> Result<T, Error> {
        match self {
            Ok(ok) => Ok(ok),
            Err(_) => Err(Error::new(msg)),
        }
    }
}

