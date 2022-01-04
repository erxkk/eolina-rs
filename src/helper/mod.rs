mod ascii;
pub use ascii::AsciiExt;

pub mod nom;

mod range;
pub use range::EolinaIndex;
pub use range::EolinaRange;
pub use range::IndexError;
pub use range::RangeError;
