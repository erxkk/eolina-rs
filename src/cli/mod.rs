mod err;
pub use err::Error;

mod inner;
pub use inner::Eolina;

pub type Result = std::result::Result<(), Error>;
