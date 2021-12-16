///
/// An error that can occur during cli execution.
///
#[derive(thiserror::Error, Debug)]
pub enum Error {
    ///
    /// An error occured during a program execution.
    ///
    #[error("execution error")]
    Program(#[from] crate::program::Error),

    ///
    /// An error occured during repl execution.
    ///
    #[error("repl error")]
    Repl(#[from] crate::repl::Error),

    ///
    /// An error occured during IO.
    ///
    #[error("io error")]
    Io(#[from] std::io::Error),

    ///
    /// An error occured during arg parsing.
    ///
    #[error("argument parsing error")]
    Clap(#[from] clap::Error),

    ///
    /// A logic error caused by a user.
    ///
    #[error("error")]
    User(String),
}
