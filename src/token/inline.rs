use crate::token::Program;
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

///
/// A token that is a literal or can be inlined as a literal.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inline<'p> {
    ///
    /// A sub program to inline.
    ///
    Program(Program<'p>),

    ///
    /// A string literal `"..."`.
    ///
    StringLiteral(Cow<'p, str>),

    ///
    /// An array literal `("...", ...)`.
    ///
    ArrayLiteral(Vec<Cow<'p, str>>),
}

impl<'p> Display for Inline<'p> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Program(program) => program.fmt(f),
            Self::StringLiteral(literal) => literal.fmt(f),
            Self::ArrayLiteral(literal) => {
                let mut tuple = f.debug_tuple("");

                for _ in literal {
                    tuple.field(literal);
                }

                tuple.finish()
            }
        }
    }
}
