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
