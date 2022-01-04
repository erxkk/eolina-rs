use crate::token::Inline;
use std::fmt::{self, Display, Formatter, Write};

///
/// A predicate token, a token used as `x` in a filter `f[x]` or as `p[x]` itself.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Predicate<'p> {
    ///
    /// The vowel predicate `v`.
    ///
    Vowel,

    ///
    /// The vowel predicate `c`.
    ///
    Conso,

    ///
    /// The lowercase predicate `_`.
    ///
    Lower,

    ///
    /// The uppercase predicate `^`.
    ///
    Upper,

    ///
    /// The literal filter `x` where x is a literal.
    ///
    Contains(Inline<'p>),
}

impl<'p> Display for Predicate<'p> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vowel => f.write_char('v'),
            Self::Conso => f.write_char('c'),
            Self::Lower => f.write_char('_'),
            Self::Upper => f.write_char('^'),
            Self::Contains(literal) => write!(f, "p[{}]", literal),
        }
    }
}
