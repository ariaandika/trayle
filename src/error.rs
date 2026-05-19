use std::io;

// ===== Error Util =====

pub struct UnknownId;
pub struct UnknownKey;

macro_rules! impl_subject {
    ($t:ty, $n:literal) => {
        impl Subject for $t {
            const NAME: &'static str = $n;
        }
    };
}
impl_subject!(crate::epoll::Epoll, "EPOLL");
impl_subject!(crate::sigfd::Sigfd, "SIGFD");
impl_subject!(crate::listener::Listener, "LISTENER");
impl_subject!(crate::clients::ClientMut<'_>, "CLIENT");
impl_subject!(crate::clients::Clients, "CLIENTS");

pub trait Subject {
    const NAME: &'static str;

    fn err<T, R: Into<Repr>>(&self, err: R, m: &'static str) -> Result<T, HandleError> {
        Err(HandleError::new(Self::NAME, m, err.into()))
    }
}

pub trait HandleErrorExt<T> {
    fn cx<S: Subject>(self, m: &'static str) -> Result<T, HandleError>;
}

impl<S: Subject> Subject for &S {
    const NAME: &'static str = S::NAME;
}

impl<T, E: Into<Repr>> HandleErrorExt<T> for Result<T, E> {
    fn cx<S: Subject>(self, m: &'static str) -> Result<T, HandleError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(HandleError::new(S::NAME, m, err.into()))
        }
    }
}

impl<T> HandleErrorExt<T> for Option<T> {
    fn cx<S: Subject>(self, m: &'static str) -> Result<T, HandleError> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(HandleError::new(S::NAME, m, Repr::None)),
        }
    }
}

// ===== Handle Error =====

pub struct HandleError {
    subject: &'static str,
    message: &'static str,
    repr: Repr,
}

pub enum Repr {
    Errno,
    UnknownId,
    UnknownKey,
    MsgError(crate::conn::MsgError),
    None,
}

macro_rules! impl_into_repr {
    ($t:ty, $r:ident) => {
        impl From<$t> for Repr {
            fn from(_: $t) -> Self { Self::$r }
        }
    };
}
impl_into_repr!(crate::errno::Errno, Errno);
impl_into_repr!(UnknownId, UnknownId);
impl_into_repr!(UnknownKey, UnknownKey);

impl From<crate::conn::MsgError> for Repr {
    fn from(value: crate::conn::MsgError) -> Self {
        Self::MsgError(value)
    }
}

impl HandleError {
    fn new(subject: &'static str, message: &'static str, repr: Repr) -> Self {
        Self { subject, message, repr }
    }
}

impl std::fmt::Display for HandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] failed to {}", self.subject, self.message)?;
        match &self.repr {
            Repr::Errno => write!(f, ": {}", std::io::Error::last_os_error()),
            Repr::UnknownId => write!(f, ": unrecognized ID"),
            Repr::UnknownKey => write!(f, ": unrecognized key"),
            Repr::MsgError(err) => err.fmt(f),
            Repr::None => Ok(()),
        }
    }
}

// ===== Error =====

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// General error.
pub struct Error {
    inner: Box<ErrorRepr>,
}

#[derive(Debug)]
enum ErrorRepr {
    Errno,
    Bind(crate::listener::BindError),
    Io(io::Error),
}

impl From<crate::errno::Errno> for Error {
    fn from(_: crate::errno::Errno) -> Self {
        Self {
            inner: Box::new(ErrorRepr::Errno),
        }
    }
}

impl From<crate::listener::BindError> for Error {
    fn from(value: crate::listener::BindError) -> Self {
        Self {
            inner: Box::new(ErrorRepr::Bind(value)),
        }
    }
}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Self {
            inner: Box::new(ErrorRepr::Io(v)),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ErrorRepr as E;
        match self.inner.as_ref() {
            E::Errno => write!(f, "fatal error: {}", io::Error::last_os_error()),
            E::Bind(err) => write!(f, "{err}"),
            E::Io(err) => write!(f, "fatal error: {err}"),
        }
    }
}

// ===== Terminate =====

/// Custom exit behavior.
///
/// Because `fn main()` is a freak and uses `Debug` and prefixed with `Error: ` for `Result` return.
pub struct Terminate {
    result: Result<()>
}

impl From<Result<()>> for Terminate {
    fn from(result: Result<()>) -> Self {
        Self { result }
    }
}

impl std::process::Termination for Terminate {
    fn report(self) -> std::process::ExitCode {
        match self.result {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("{err}");
                std::process::ExitCode::FAILURE
            }
        }
    }
}
