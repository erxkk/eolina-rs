use std::fmt::{self, Display, Formatter, Write};

///
/// A single toke n that can be repeated with an optional number.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Repeat {
    kind: Kind,
    count: usize,
}

impl Repeat {
    ///
    /// Creates a new [`Repeat`] from the given [`Kind`] and `count`.
    ///
    pub fn new(kind: Kind, count: usize) -> Self {
        Self { kind, count }
    }

    ///
    /// Returns the [`Kind`] of this token.
    ///
    pub fn kind(&self) -> Kind {
        self.kind
    }

    ///
    /// Returns how often this token should be repeated.
    ///
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Display for Repeat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)?;
        self.count.fmt(f)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    ///
    /// The input token `<x`.
    ///
    In,

    ///
    /// The output token `>x`.
    ///
    Out,

    ///
    /// The concatenation token `~x`.
    ///
    Concat,

    ///
    /// The copy token `*x`.
    ///
    Duplicate,

    ///
    /// The rotate token `@x`.
    ///
    Rotate,
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Kind::In => f.write_char('<'),
            Kind::Out => f.write_char('>'),
            Kind::Concat => f.write_char('~'),
            Kind::Duplicate => f.write_char('.'),
            Kind::Rotate => f.write_char('@'),
        }
    }
}
