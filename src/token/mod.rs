mod transform;
pub use transform::Map;
pub use transform::Predicate;
pub use transform::Transform;

mod inline;
pub use inline::Inline;

mod program;
pub use program::Program;

mod repeat;
pub use repeat::Kind as RepeatKind;
pub use repeat::Repeat;

use crate::helper::{EolinaIndex, EolinaRange};
use std::fmt::{self, Display, Formatter};

///
/// A function token.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'p> {
    ///
    /// A repeatable single token.
    ///
    Repeat(Repeat),

    ///
    /// A token block `{x}` where `x` is a valid program or a literal `"..."`/`(...)`.
    ///
    Inline(Inline<'p>),

    ///
    /// A transform token `x[y]` where `x` is a transform, `y` is an argument.
    ///
    Transform(Transform<'p>),

    ///
    /// The index token `|x|` where `x` is a [`isize`].
    ///
    Index(EolinaIndex),

    ///
    /// The slice token `|x.x|` where `x` are empty or [`isize`].
    ///
    Slice(EolinaRange),
}

impl<'p> Display for Token<'p> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repeat(repeat) => repeat.fmt(f),
            Self::Inline(inline) => inline.fmt(f),
            Self::Transform(transform) => transform.fmt(f),
            Self::Index(idx) => write!(f, "|{}|", idx),
            Self::Slice(range) => range.fmt(f),
        }
    }
}
