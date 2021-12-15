use crate::exec;

///
/// Represents an error that can occur during cli execution.
///
#[derive(Debug)]
pub struct Error {
    repr: Kind,
}

impl From<exec::Error> for Error {
    fn from(inner: exec::Error) -> Self {
        Self {
            repr: Kind::Exec(inner),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(inner: std::io::Error) -> Self {
        Self {
            repr: Kind::Io(inner),
        }
    }
}

impl From<clap::Error> for Error {
    fn from(inner: clap::Error) -> Self {
        Self {
            repr: Kind::Clap(inner),
        }
    }
}

///
/// Represents the kind of an error during cli execution.
///
#[derive(Debug)]
pub enum Kind {
    ///
    /// An error during executor execution.
    ///
    Exec(exec::Error),

    ///
    /// An error during IO.
    ///
    Io(std::io::Error),

    ///
    /// An error during arg parsing.
    ///
    Clap(clap::Error),
}
