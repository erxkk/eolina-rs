///
/// Represents an error during parsing.
///
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Error {
    ///
    /// The given program was empty.
    ///
    #[error("empty program")]
    Empty,

    ///
    /// Not known token was found at the start of the program.
    ///
    #[error("unknown token `{0}`")]
    Unknown(String),
}
