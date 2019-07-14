use std::error::Error;

pub type DecodeResult<T> = Result<T, Decode Error>;

#[derive(Debug)]
pub enum DecodeError {
    InvalidHeader,
}

impl Error for DecodeError {
    fn description(&self) -> &str {
        match self {
            DecodeError::InvalidHeader => "Invalid header bytes passed",
        }
    }

    fn cause(&self) -> Option<&dyn StdError> { None }
    fn source(&self) -> Option<&(dyn StdError + 'static)> { None }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}