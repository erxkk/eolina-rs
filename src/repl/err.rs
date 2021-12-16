///
/// An error that can occur during repl execution.
///
#[derive(thiserror::Error, Debug)]
pub enum Error {
    ///
    /// Attempted to use an unknown command.
    ///
    #[error("unknown command: `{0}`")]
    UnknownCommand(String),

    ///
    /// An error occured during program execution.
    ///
    #[error("exec error")]
    Program(#[from] crate::program::Error),

    ///
    /// An error occured during IO.
    ///
    #[error("io error")]
    Io(#[from] std::io::Error),
}
