///
/// An error that can occur during parsing.
///
#[derive(thiserror::Error, Debug)]
pub enum Error {
    ///
    /// A given program was empty.
    ///
    #[error("empty program")]
    Empty,

    ///
    /// Unknown token was encountered.
    ///
    #[error("unknown token at `{0}`")]
    Unknown(String),
}
