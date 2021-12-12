//!
//! Contains utility to parse eolina tokens lazily.
//!

mod iter;
pub use iter::TokenIter;

mod err;
pub use err::Error;

mod token;
pub use token::next_token;
pub use token::Filter as FilterToken;
pub use token::Token;
