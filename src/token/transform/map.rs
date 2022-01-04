use std::fmt::{self, Display, Formatter, Write};

///
/// A map token, used in as `x` in a map `m[x]`.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Map {
    ///
    /// The lowercase map `_`.
    ///
    Lower,

    ///
    /// The uppercase map `^`.
    ///
    Upper,

    ///
    /// The swap case map `%`.
    ///
    Swap,
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lower => f.write_char('_'),
            Self::Upper => f.write_char('^'),
            Self::Swap => f.write_char('%'),
        }
    }
}
