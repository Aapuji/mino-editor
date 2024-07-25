use std::io;
use std::fmt;

use crate::screen::Screen;

#[derive(Debug)]
pub enum Error {
    Io(io::ErrorKind)
}

impl Report for Error {
    type Output = Self;

    fn report(self, screen: &mut Screen) -> Self::Output {
        screen.set_status_msg(format!("\x1b[31mError:\x1b[m {}", self));

        self
    }

    fn noscreen_report(self) {
        eprintln!("{self}");
    }
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

        write!(f, " \x1b[31mError:\x1b[31m {}", err_msg)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl<T> Report for Result<T> {
    type Output = Self;

    fn report(self, screen: &mut Screen) -> Self::Output {
        if let Err(err) = self {
            Err(err.report(screen))
        } else {
            self
        }
    }

    fn noscreen_report(self) {
        if let Err(err) = self {
            eprintln!("\x1bc{err}");
        }
    }
}

/// Trait for reporting errors to the user.
/// 
/// I made it a trait so I could implement it for Result<T, Error>, so I don't need 100k `if let Err(err) = ..` statements.
pub trait Report {
    type Output;

    /// Reports error through the status msg area.
    fn report(self, screen: &mut Screen) -> Self::Output;

    /// Reports error by clearing the screen and printing.
    fn noscreen_report(self);
}