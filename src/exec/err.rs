use super::ValueKind;
use crate::parse;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io,
};

///
/// Represents an error during interpretation of an eolina programm.
///
#[derive(Debug)]
pub struct Error {
    repr: ErrorKind,
}

impl Error {
    ///
    /// Creates a new [`Error`] for a parse error with the given inner [`ParseError`].
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
    /// Creates a new [`Error`] for a concat type mismatch with the given the `expected`
    /// and `actual` types.
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
    fn from(inner: io::Error) -> Self {
        Self::io(inner)
    }
}

impl From<parse::Error> for Error {
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
    /// An error occured during IO (`<` failed reading / `>` failed writing).
    ///
    Io(io::Error),
    ///
    /// The top of the stack did not conain the right type of value for the current function.
    ///
    ArgMismatch(&'static [ValueKind], ValueKind),
    ///
    /// The tow last values on the stack were not of the same type.
    ///
    Mismatch(ValueKind, ValueKind),
    ///
    /// The stack was empty, but there was still a function left that requires a non-IO input.
    ///
    QueueEmpty,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Parse(inner) => write!(f, "parse error: ({})", inner),
            Self::Io(inner) => write!(f, "io error: ({})", inner),
            Self::ArgMismatch(expected, actual) => {
                if expected.len() == 1 {
                    write!(
                        f,
                        "arg mismatch: expected `{}`, found `{}`",
                        expected[0], actual
                    )
                } else {
                    write!(
                        f,
                        "arg mismatch: expected any of `{:?}`, found `{}`",
                        expected, actual
                    )
                }
            }
            Self::Mismatch(left, right) => {
                write!(f, "type mismatch: `{}` != `{}`", left, right)
            }
            Self::QueueEmpty => f.write_str("No value found on queue"),
        }
    }
}
