//!
//! Contains functionailty and data structures related to execution
//! of a program.
//!

mod err;
pub use self::err::Error;

mod func;

mod inner;
pub use inner::Context;

mod value;
pub use value::Kind;
pub use value::Value;
