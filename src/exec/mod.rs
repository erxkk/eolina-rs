//!
//! Contains functionailty and data structures related to execution
//! of a program.
//!

mod err;
pub use self::err::Error;

mod func;

mod inner;
pub use inner::Executor;

mod value;
pub use value::Value;
pub use value::ValueKind;
