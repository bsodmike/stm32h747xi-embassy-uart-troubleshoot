use alloc::boxed::Box;
use core::{error::Error as StdError, fmt, fmt::Debug};

pub type BoxError = Box<dyn StdError + Send + Sync>;

#[derive(Debug)]
pub struct BoardError {
    inner: Box<ErrorKind>,
}

#[allow(dead_code)]
#[derive(Debug)]
/// Error kind record struct holding both [`Kind`] and an [`Option`]< [`BoxError`]>
struct ErrorKind {
    kind: Kind,
    cause: Option<BoxError>,
}

/// Error kind enum
#[derive(Debug)]
pub enum Kind {
    /// Default crate error.
    InternalError,
}

impl BoardError {
    /// Create an instance of [`BoardError`] specifying a [`Kind`] but without a cause.
    fn with_kind(kind: Kind) -> Self {
        Self {
            inner: Box::new(ErrorKind { kind, cause: None }),
        }
    }

    /// Create a new instance of [`Error`] with cause of type [`BoxError`]
    pub fn with<C: Into<BoxError>>(mut self, cause: C) -> Self {
        self.inner.cause = Some(cause.into());
        self
    }

    /// Create a new instance of [`Error`] of type [`Kind::InternalError`] with cause of type [`BoxError`]
    pub fn new<E: Into<BoxError>>(cause: E) -> Self {
        Self::default().with(cause)
    }
}

impl core::default::Default for BoardError {
    /// Create a new instance of [`Error`] of type [`Kind::InternalError`]
    fn default() -> Self {
        Self::with_kind(Kind::InternalError)
    }
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(ref cause) = self.inner.cause {
            write!(f, "BoardError: {}", cause)
        } else {
            f.write_str("BoardError: Unknown error")
        }
    }
}

impl StdError for BoardError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.inner
            .cause
            .as_ref()
            .map(|cause| &**cause as &(dyn StdError + 'static))
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn StdError> {
        self.source()
    }

    fn provide<'a>(&'a self, _request: &mut core::error::Request<'a>) {}
}

pub(crate) struct WriteErr {}

impl From<core::fmt::Error> for WriteErr {
    fn from(_val: core::fmt::Error) -> Self {
        Self {}
    }
}

impl From<WriteErr> for BoxError {
    fn from(_value: WriteErr) -> Self {
        Box::<BoardError>::default()
    }
}
