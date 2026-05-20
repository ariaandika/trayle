pub type BoxError = Box<dyn std::error::Error>;

pub type Result<T, E = Error> = std::result::Result<T, E>;

// ===== Error =====

/// General error.
pub struct Error {
    inner: BoxError,
}

impl<E: Into<BoxError>> From<E> for Error {
    fn from(value: E) -> Self {
        Self {
            inner: value.into(),
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
        self.inner.fmt(f)
    }
}

// ===== Terminate =====

/// Custom exit behavior.
///
/// Because `fn main()` is a freak and uses `Debug` and prefixed with `Error: ` for `Result` return.
pub struct Terminate {
    result: Result<()>,
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
