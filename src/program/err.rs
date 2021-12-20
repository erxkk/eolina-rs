use super::Kind;
use std::fmt::{Display, Formatter, Result as FmtResult};

///
/// An error that can occur during program execution.
///
#[derive(thiserror::Error, Debug)]
pub enum Error {
    ///
    /// A relative index resulted in an invalid target index range.
    ///
    #[error("index error")]
    Index(#[from] crate::helper::IndexError),

    ///
    /// A relative range resulted in an invalid target slice range.
    ///
    #[error("range error")]
    Range(#[from] crate::helper::RangeError),

    ///
    /// A function argument was not of an expected type.
    ///
    #[error("arg mismatch")]
    ArgMismatch(#[from] ArgMismatchError),

    ///
    /// Two values were not of the same type.
    ///
    #[error("type mismatch: <{0}> != <{1}>")]
    Mismatch(Kind, Kind),

    ///
    /// The queue was shorter than expected.
    ///
    #[error("expected at least {0} value(s) on the queue, found {1}")]
    QueueTooShort(usize, usize),
}

#[derive(thiserror::Error, Debug)]
pub struct ArgMismatchError {
    expected: &'static [Kind],
    given: Kind,
}

impl ArgMismatchError {
    pub(super) fn new(expected: &'static [Kind], given: Kind) -> Self {
        Self { expected, given }
    }
}

impl Display for ArgMismatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.expected.len() == 1 {
            write!(f, "expected '{}', given '{}'", self.expected[0], self.given)
        } else {
            write!(f, "expected any of")?;

            let mut iter = self.expected.iter();
            write!(f, " '{}'", iter.next().expect("no expected args given"))?;

            for expected in iter {
                write!(f, ", '{}'", expected)?;
            }

            write!(f, ", found '{}'", self.given)?;

            Ok(())
        }
    }
}
