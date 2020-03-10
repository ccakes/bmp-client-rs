use std::error::Error as StdError;

#[derive(Debug, Eq, PartialEq)]
pub enum ErrorKind {
    PeerDisconnected,
    Io,
    Other,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub description: String,
    pub cause: Option<Box<dyn StdError>>,
}

impl Error {
    pub(crate) fn disconnected() -> Self {
        Self {
            kind: ErrorKind::PeerDisconnected,
            description: String::from("Peer disconnected"),
            cause: None
        }
    }
}

impl From<Box<dyn StdError>> for Error {
    fn from(err: Box<dyn StdError>) -> Self {
        Self {
            kind: ErrorKind::Other,
            description: err.to_string(),
            cause: Some(err)
        }
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