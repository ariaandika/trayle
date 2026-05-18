use std::io;

pub type Result<T, E = Error> = std::result::Result<T, E>;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub trait ErrorExt<T> {
    fn cx<S: ToString>(self, context: S) -> Result<T, Error>;
}

impl<T, E: Into<BoxError>> ErrorExt<T> for Result<T, E> {
    fn cx<S: ToString>(self, context: S) -> Result<T, Error> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(Error::context(context.to_string(), err.into())),
        }
    }
}

/// General error.
pub struct Error {
    inner: Box<ErrorRepr>,
}

#[derive(Debug)]
enum ErrorRepr {
    Context(String, BoxError),
    Errno,
    Io(io::Error),
}

impl Error {
    fn context(cx: String, err: BoxError) -> Self {
        Self {
            inner: Box::new(ErrorRepr::Context(cx, err)),
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
            E::Context(msg, err) => write!(f, "{msg}: {err}"),
            E::Errno => write!(f, "fatal error: {}", io::Error::last_os_error()),
            E::Io(err) => write!(f, "fatal error: {err}"),
        }
    }
}

impl From<crate::errno::Errno> for Error {
    fn from(_: crate::errno::Errno) -> Self {
        Self {
            inner: Box::new(ErrorRepr::Errno),
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

/// Custom exit behavior.
///
/// Because `fn main()` is a freak and uses `Debug` with new line suffix for `Result` return.
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
        let Err(err) = self.result else {
            return std::process::ExitCode::SUCCESS;
        };
        eprintln!("{err}");
        std::process::ExitCode::FAILURE
    }
}
