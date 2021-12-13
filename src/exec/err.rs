use super::ValueKind;
use crate::{
    helper::{EolinaRange, EolinaRangeBound},
    parse,
};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io,
};

///
/// Represents an error taht can occur during execution of a program.
///
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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
    /// Creates a new [`Error`] for a slice error, with the given invalid range and target length.
    ///
    pub fn slice_oor(
        given: impl Into<EolinaRange>,
        translated: impl Into<EolinaRange>,
        length: usize,
    ) -> Self {
        Self {
            repr: ErrorKind::SliceOutOfRange(given.into(), translated.into(), length),
        }
    }

    ///
    /// Creates a new [`Error`] for a slice error, with the given invalid range and target length.
    ///
    pub fn slice_incompat(
        given: impl Into<EolinaRange>,
        translated: impl Into<EolinaRange>,
        length: usize,
    ) -> Self {
        Self {
            repr: ErrorKind::SliceIncompatible(given.into(), translated.into(), length),
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
#[non_exhaustive]
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
    /// An error when have incompatible indecies (start > end).
    ///
    SliceIncompatible(EolinaRange, EolinaRange, usize),

    ///
    /// An error when the the slice range is out of target range bound.
    ///
    SliceOutOfRange(EolinaRange, EolinaRange, usize),

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

#[cfg(test)]
impl PartialEq for ErrorKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ErrorKind::Parse(this), ErrorKind::Parse(other)) if this == other => true,
            (
                ErrorKind::SliceIncompatible(this_g, this_e, this_l),
                ErrorKind::SliceIncompatible(other_g, other_e, other_l),
            ) if this_l == other_l && this_e == other_e && this_g == other_g => true,
            (
                ErrorKind::SliceOutOfRange(this_g, this_e, this_l),
                ErrorKind::SliceOutOfRange(other_g, other_e, other_l),
            ) if this_l == other_l && this_e == other_e && this_g == other_g => true,
            (ErrorKind::ArgMismatch(this_e, this_g), ErrorKind::ArgMismatch(other_e, other_g))
                if this_g == other_g && this_e == other_e =>
            {
                true
            }
            (ErrorKind::Mismatch(this_e, this_g), ErrorKind::Mismatch(other_e, other_g))
                if this_g == other_g && this_e == other_e =>
            {
                true
            }
            (ErrorKind::QueueEmpty, ErrorKind::QueueEmpty) => true,
            (ErrorKind::Io(_), ErrorKind::Io(_)) => panic!("don't compare io errors"),
            _ => false,
        }
    }
}

#[cfg(test)]
impl Eq for ErrorKind {}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Parse(inner) => write!(f, "parse error: {}", inner),
            Self::Io(inner) => write!(f, "io error: {}", inner),
            Self::SliceIncompatible(given, translated, length) => {
                write!(
                    f,
                    "slice error: start > end ({} > {}) (`{}` => `{}`)",
                    translated.start.unwrap_or(EolinaRangeBound::Start(0)),
                    translated.end.unwrap_or(EolinaRangeBound::Start(*length)),
                    given,
                    translated
                )
            }
            Self::SliceOutOfRange(given, translated, length) => {
                let start: isize = translated
                    .start
                    .unwrap_or(EolinaRangeBound::Start(0))
                    .into();
                let end: isize = translated
                    .end
                    .unwrap_or(EolinaRangeBound::Start(*length))
                    .into();
                let length = *length as isize;

                f.write_str("slice error: ")?;

                // TODO: this can probably be improved
                if start < 0 && end < 0 {
                    write!(f, "start & end < 0 ({}, {})", start, end)
                } else if start < 0 {
                    write!(f, "start < 0 ({})", start)
                } else if end < 0 {
                    write!(f, "end < 0 ({})", start)
                } else if start > length && end > length {
                    write!(f, "start & end > len ({}, {} > {})", start, end, length)
                } else if end > length {
                    write!(f, "end > len ({} > {})", end, length)
                } else if start > length {
                    write!(f, "start > len ({} > {})", end, length)
                } else {
                    unreachable!(
                        "invalid branch:\ngiven: {}\ntranslated: {}\nlen: {}",
                        given, translated, length
                    )
                }?;

                write!(f, " (rel {} => abs {})", given, translated)?;
                Ok(())
            }
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
