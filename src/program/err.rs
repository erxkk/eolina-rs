use super::Kind;
use crate::helper::{EolinaRange, EolinaRangeBound};
use std::{
    cmp::Ordering::Greater,
    fmt::{Display, Formatter, Result as FmtResult},
};

///
/// An error that can occur during program execution.
///
#[derive(thiserror::Error, Debug)]
pub enum Error {
    ///
    /// An error occured during parsing.
    ///
    Parse(#[from] crate::parse::Error),

    ///
    /// A slice had incompatible indecies (start > end).
    ///
    SliceIncompatible(EolinaRange, EolinaRange, usize),

    ///
    /// The slice range bounds are out of the target range bounds.
    ///
    SliceOutOfRange(EolinaRange, EolinaRange, usize),

    ///
    /// A function argument was not of an expected type.
    ///
    ArgMismatch(&'static [Kind], Kind),

    ///
    /// Two values were not of the same type.
    ///
    Mismatch(Kind, Kind),

    ///
    /// The queue was empty.
    ///
    QueueEmpty,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Parse(_) => write!(f, "parse error"),
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

                match (start > 0, end > 0) {
                    (true, true) => write!(f, "start & end < 0 ({}, {})", start, end)?,
                    (true, _) => write!(f, "start < 0 ({})", start)?,
                    (_, true) => write!(f, "end < 0 ({})", start)?,
                    _ => {}
                };

                match (start.cmp(&length), end.cmp(&length)) {
                    (Greater, Greater) => {
                        write!(f, "start & end > len ({}, {} > {})", start, end, length)?
                    }
                    (Greater, _) => write!(f, "start > len ({} > {})", end, length)?,
                    (_, Greater) => write!(f, "end > len ({} > {})", end, length)?,
                    _ => {}
                };

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
                    write!(f, " `{}`", iter.next().expect("no expected args given"))?;

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
