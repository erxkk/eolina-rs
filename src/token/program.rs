use std::fmt::{self, Display, Formatter};

// TODO: the tokens alone imply lazy evaluation, add a generator to abstract it and allow eager?

///
/// A sub program to execute and inline.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Program<'p> {
    ///
    /// The sublice of tokens of the original program that correspond to this sub program.
    ///
    tokens: &'p str,
}

impl<'p> Program<'p> {
    pub fn new(tokens: &'p str) -> Self {
        Self { tokens }
    }
}

impl<'p> Display for Program<'p> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}}}", self.tokens)
    }
}
