use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    Truncated,
    Unsupported,
    Malformed,
    Checksum,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Truncated => write!(f, "truncated input buffer"),
            Error::Unsupported => write!(f, "unsupported input parameter"),
            Error::Malformed => write!(f, "malformed input parameter"),
            Error::Checksum => write!(f, "bad checksum"),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
