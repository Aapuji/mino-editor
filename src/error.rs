use std::io;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(io::ErrorKind)
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            Self::Io(err) => match err {
                io::ErrorKind::NotFound         => "File not found",
                io::ErrorKind::PermissionDenied => "Permission denied",
                io::ErrorKind::AlreadyExists    => "File already exists",
                _                               => &format!("{}", err)
            } 
        };

        write!(f, " \x1b[31m\u{26A0}Error!\x1b[31m {}", err_msg)
    }
}

pub type Result<T> = std::result::Result<T, Error>;