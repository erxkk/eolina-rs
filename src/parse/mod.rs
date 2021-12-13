//!
//! Contains utility to parse tokens.
//!

mod iter;
pub use iter::TokenIter;

mod err;
pub use err::Error;

mod token;
pub use token::next_token;
pub use token::Check as CheckToken;
pub use token::Map as MapToken;
pub use token::Token;
