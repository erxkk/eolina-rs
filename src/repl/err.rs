///
/// Represents an error that can occur during repl execution.
///
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    ///
    /// An error when an unknown command is attempted to be loaded.
    ///
    #[error("unknown command: `{0}`")]
    UnknownCommand(String),

    ///
    /// An error when a command is missing a parameter.
    ///
    #[error("missing param {0} at pos {1}")]
    MissingCommandParameter(&'static str, usize),

    ///
    /// An error when an unknown program is attempted to be loaded.
    ///
    #[error("unknown program: `{0}`")]
    UnknownProgramm(String),

    ///
    /// An error during executor execution.
    ///
    #[error("exec error: {0}")]
    Exec(#[from] crate::exec::Error),

    ///
    /// An error occured during IO.
    ///
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
