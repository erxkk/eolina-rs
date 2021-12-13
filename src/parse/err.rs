use std::fmt::{self, Display, Formatter};

///
/// Represents an error during tokenization.
///
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Error {
    slice: String,
    kind: ErrorKind,
}

impl Error {
    ///
    /// Creates a new [`Error`] for an unknown token at th estart of the given slice.
    ///
    pub fn unknown(slice: String) -> Self {
        Self {
            slice,
            kind: ErrorKind::Unknown,
        }
    }

    ///
    /// Creates a new [`Error`] for an empty slice.
    ///
    pub fn empty() -> Self {
        Self {
            slice: "".to_owned(),
            kind: ErrorKind::Empty,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::Empty => f.write_str("the slice was empty."),
            ErrorKind::Unknown => write!(f, "unknown token encountered at: `{}`", self.slice),
        }
    }
}

///
/// Represents an error during tokenization.
///
#[non_exhaustive]
#[derive(Debug)]
enum ErrorKind {
    ///
    /// The given program was empty.
    ///
    Empty,

    ///
    /// Not known token was found at the start of the program.
    ///
    Unknown,
}

#[cfg(test)]
impl PartialEq for ErrorKind {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (ErrorKind::Empty, ErrorKind::Empty) | (ErrorKind::Unknown, ErrorKind::Unknown)
        )
    }
}

#[cfg(test)]
impl Eq for ErrorKind {}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty program"),
            Self::Unknown => f.write_str("unknown token"),
        }
    }
}
