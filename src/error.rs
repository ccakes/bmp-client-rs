#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Other(String),
    Unknown(Box<dyn std::error::Error>),
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {}", error),
            Self::Other(error) => write!(f, "Error: {}", error),

            Self::Unknown(error) => write!(f, "Unknown error: {}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<Box<dyn std::error::Error + Sync + Send>> for Error {
    fn from(error: Box<dyn std::error::Error + Sync + Send>) -> Self {
        Self::Unknown(error)
    }
}

// use failure::{Backtrace, Context, Fail};

// use std::fmt;
// // use std::io;

// #[derive(Debug)]
// pub struct BmpError {
//     inner: Context<BmpErrorKind>,
// }

// #[derive(Clone, Eq, PartialEq, Debug, Fail)]
// pub enum BmpErrorKind {
//     #[fail(display = "Peer disconnected")]
//     PeerDisconnected,

//     #[fail(display = "Some other error")]
//     Other(failure::Error),
// }

// impl Fail for BmpError {
//     fn cause(&self) -> Option<&Fail> {
//         self.inner.cause()
//     }

//     fn backtrace(&self) -> Option<&Backtrace> {
//         self.inner.backtrace()
//     }
// }

// impl fmt::Display for BmpError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         fmt::Display::fmt(&self.inner, f)
//     }
// }

// impl BmpError {
//     pub fn kind(&self) -> BmpErrorKind {
//         *self.inner.get_context()
//     }
// }

// impl From<BmpErrorKind> for BmpError {
//     fn from(kind: BmpErrorKind) -> BmpError {
//         BmpError { inner: Context::new(kind) }
//     }
// }

// impl From<Context<BmpErrorKind>> for BmpError {
//     fn from(inner: Context<BmpErrorKind>) -> BmpError {
//         BmpError { inner: inner }
//     }
// }