use crate::exec;
use std::{
    fmt::{self, Display, Formatter},
    io,
};

///
/// Represents an error that can occur during repl execution.
///
#[derive(Debug)]
pub struct Error<'a> {
    repr: ErrorKind<'a>,
}

impl<'a> Error<'a> {
    ///
    /// Creates a new [`Error`] for a missing command parameter.
    ///
    pub fn missing_param(param: &'static str, pos: usize) -> Self {
        Self {
            repr: ErrorKind::MissingCommandParameter(param, pos),
        }
    }

    ///
    /// Creates a new [`Error`] for an unknown command.
    ///
    pub fn unknown_command(command: &'a str) -> Self {
        Self {
            repr: ErrorKind::UnknownCommand(command),
        }
    }

    ///
    /// Creates a new [`Error`] for an unknown program.
    ///
    pub fn unknown_program(program: &'a str) -> Self {
        Self {
            repr: ErrorKind::UnknownProgramm(program),
        }
    }

    ///
    /// Creates a new [`Error`] for the given inner [`exec::Error`].
    ///
    pub fn exec(inner: exec::Error) -> Self {
        Self {
            repr: ErrorKind::Executor(inner),
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
}

impl<'a> From<exec::Error> for Error<'a> {
    ///
    /// Creates an [`Error`] from an [`exec::Error`].
    ///
    fn from(inner: exec::Error) -> Self {
        Self::exec(inner)
    }
}

impl<'a> From<io::Error> for Error<'a> {
    ///
    /// Creates an [`Error`] from an [`io::Error`].
    ///
    fn from(inner: io::Error) -> Self {
        Self::io(inner)
    }
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.repr.fmt(f)
    }
}

///
/// Represents the kind of an error during repl execution.
///
#[non_exhaustive]
#[derive(Debug)]
enum ErrorKind<'a> {
    ///
    /// An error when an unknown command is attempted to be loaded.
    ///
    UnknownCommand(&'a str),

    ///
    /// An error when a command is missing a parameter.
    ///
    MissingCommandParameter(&'static str, usize),

    ///
    /// An error when an unknown program is attempted to be loaded.
    ///
    UnknownProgramm(&'a str),

    ///
    /// An error during executor execution.
    ///
    Executor(exec::Error),

    ///
    /// An error occured during IO.
    ///
    Io(io::Error),
}

impl<'a> Display for ErrorKind<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::UnknownCommand(cmd) => write!(f, "unknown command: `{}`", cmd),
            ErrorKind::MissingCommandParameter(param, pos) => {
                write!(f, "missing param {} at pos {}", param, pos)
            }
            ErrorKind::UnknownProgramm(program) => write!(f, "unknown program: `{}`", program),
            ErrorKind::Executor(inner) => write!(f, "exec error: {}", inner),
            ErrorKind::Io(inner) => write!(f, "io error: {}", inner),
        }
    }
}
