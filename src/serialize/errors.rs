use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;
use std::str::Utf8Error;

pub type Result<T> = ::std::result::Result<T, Error>;

type Cause = Box<StdError + Send + Sync>;

pub struct Error {
    inner: Box<ErrorImpl>,
}

struct ErrorImpl {
    kind: ErrorKind,
    cause: Option<Cause>,
}

#[derive(Debug)]
pub enum ErrorKind {
    SerializeError,
    UnserializeError,
    NotEnoughBytes,
    TooMuchBytes,
    Io,
    StringError,
}

impl Error {
    pub fn new(kind: ErrorKind, cause: Option<Cause>) -> Error {
        Error {
            inner: Box::new(ErrorImpl { kind, cause }),
        }
    }

    pub fn new_serialize() -> Error {
        Error::new(ErrorKind::SerializeError, None)
    }

    pub fn new_unserialize() -> Error {
        Error::new(ErrorKind::UnserializeError, None)
    }

    pub fn new_not_enough_bytes() -> Error {
        Error::new(ErrorKind::NotEnoughBytes, None)
    }

    pub fn new_too_much_bytes() -> Error {
        Error::new(ErrorKind::TooMuchBytes, None)
    }

    pub fn new_io_error(err: IoError) -> Error {
        Error::new(ErrorKind::Io, Some(err.into()))
    }

    pub fn new_string_error(err: Utf8Error) -> Error {
        Error::new(ErrorKind::StringError, Some(err.into()))
    }

    pub fn into_cause(self) -> Option<Box<StdError + Sync + Send>> {
        self.inner.cause
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Error");
        f.field("kind", &self.inner.kind);
        if let Some(ref cause) = self.inner.cause {
            f.field("cause", cause);
        }
        f.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref cause) = self.inner.cause {
            write!(f, "{}: {}", self.description(), cause)
        } else {
            f.write_str(self.description())
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self.inner.kind {
            ErrorKind::SerializeError => "Impossible to serialize.",
            ErrorKind::UnserializeError => "Impossible to unserialize.",
            ErrorKind::NotEnoughBytes => "Provided not enough bytes to serialize this type.",
            ErrorKind::TooMuchBytes => "Provided too much bytes to serialize this type.",
            ErrorKind::Io => "I/O Error",
            ErrorKind::StringError => "String Error",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        self.inner.cause.as_ref().map(|cause| &**cause as &StdError)
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::new_io_error(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Error {
        Error::new_string_error(err)
    }
}
