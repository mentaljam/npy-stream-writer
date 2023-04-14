use std::{error, fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    #[non_exhaustive]
    HeaderLen(<i32 as TryFrom<usize>>::Error, crate::VersionBytes),
    #[non_exhaustive]
    Write(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HeaderLen(_, ver) => {
                write!(f, "header length too big for version={}.{}", ver[0], ver[1])
            }
            Self::Write(_) => write!(f, "failed to write npy data"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(match self {
            Self::HeaderLen(err, ..) => err,
            Self::Write(err) => err,
        })
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Write(value)
    }
}
