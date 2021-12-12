use super::ValueKind;
use crate::parse;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io,
};

///
/// Represents an error taht can occur during execution of a program.
///
#[derive(Debug)]
pub struct Error {
    repr: ErrorKind,
}

impl Error {
    ///
    /// Creates a new [`Error`] for a parse error with the given inner [`parse::Error`].
    ///
    pub fn parse(inner: parse::Error) -> Self {
        Self {
            repr: ErrorKind::Parse(inner),
        }
    }

    ///
    /// Creates a new [`Error`] for an io error with the given inner [`io::Error`].
    ///
    pub fn io(inner: io::Error) -> Self {
        Self {
            repr: ErrorKind::Io(inner),
        }
    }

    ///
    /// Creates a new [`Error`] for a arg type mismatch with the given the `expected`
    /// and `actual` types.
    ///
    pub fn arg_mismatch(expected: &'static [ValueKind], actual: ValueKind) -> Self {
        Self {
            repr: ErrorKind::ArgMismatch(expected, actual),
        }
    }

    ///
    /// Creates a new [`Error`] for a concat type mismatch with the given the `left`
    /// and `right` types.
    ///
    pub fn mismatch(left: ValueKind, right: ValueKind) -> Self {
        Self {
            repr: ErrorKind::Mismatch(left, right),
        }
    }

    ///
    /// Creates a new [`Error`] for an empty queue.
    ///
    pub fn empty() -> Self {
        Self {
            repr: ErrorKind::QueueEmpty,
        }
    }
}

impl From<io::Error> for Error {
    ///
    /// Creates an [`Error`] from an [`io::Error`].
    ///
    fn from(inner: io::Error) -> Self {
        Self::io(inner)
    }
}

impl From<parse::Error> for Error {
    ///
    /// Creates an [`Error`] from a [`parse::Error`].
    ///
    fn from(inner: parse::Error) -> Self {
        Self::parse(inner)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.repr.fmt(f)
    }
}

///
/// Represents the kind of an error during execution.
///
#[derive(Debug)]
enum ErrorKind {
    ///
    /// An error occured during parsing.
    ///
    Parse(parse::Error),
    ///
    /// An error occured during IO.
    ///
    Io(io::Error),
    ///
    /// A function argument was not of an expected type.
    ///
    ArgMismatch(&'static [ValueKind], ValueKind),
    ///
    /// Two values were not of the same type.
    ///
    Mismatch(ValueKind, ValueKind),
    ///
    /// The queue was empty.
    ///
    QueueEmpty,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Parse(inner) => write!(f, "parse error: {}", inner),
            Self::Io(inner) => write!(f, "io error: {}", inner),
            Self::ArgMismatch(expected, actual) => {
                if expected.len() == 1 {
                    write!(
                        f,
                        "arg mismatch: expected `{}`, found `{}`",
                        expected[0], actual
                    )
                } else {
                    write!(f, "arg mismatch: expected any of")?;

                    let mut iter = expected.iter();
                    write!(f, " `{}`", iter.next().unwrap())?;

                    for expected in iter {
                        write!(f, ", `{}`", expected)?;
                    }

                    write!(f, ", found `{}`", actual)?;

                    Ok(())
                }
            }
            Self::Mismatch(left, right) => {
                write!(f, "type mismatch: `{}` != `{}`", left, right)
            }
            Self::QueueEmpty => f.write_str("No value found on queue"),
        }
    }
}
