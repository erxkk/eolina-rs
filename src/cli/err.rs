///
/// Represents an error that can occur during cli execution.
///
#[derive(thiserror::Error, Debug)]
pub enum Error {
    ///
    /// An error during eolina exec.
    ///
    #[error("execution error: {0}")]
    Exec(#[from] crate::exec::Error),

    ///
    /// An error during IO.
    ///
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    ///
    /// An error during arg parsing.
    ///
    #[error("argument parsing error: {0}")]
    Clap(#[from] clap::Error),
}
