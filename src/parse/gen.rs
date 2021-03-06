use super::{next_token, Token};
use std::{
    cmp::Ordering,
    ops::{Generator, GeneratorState},
    pin::Pin,
};

///
/// A **reusable** token generator.
///
pub trait Gen<'t>: Generator<Yield = (Token<'t>, usize), Return = color_eyre::Result<()>> {
    ///
    /// Returns this [`Gen`]'s token slice.
    ///
    fn tokens(&self) -> &'t str;

    /// Resets this [`Gen`]'s state to begin yielding at the start again.
    fn reset(&mut self);
}

///
/// A lazily evaluated token [`Gen`], will yield until the generator is complete or
/// encounters an invalid [`Token`].
///
#[derive(Debug)]
pub struct LazyGen<'t> {
    ///
    /// The program this token is yielding tokens from.
    ///
    program: &'t str,

    ///
    /// The remainting slice of tokens to yield.
    ///
    remaining: &'t str,

    ///
    /// Whether or not this generator completed, this is set to true after remaining is "".
    ///
    completed: bool,
}

impl<'t> LazyGen<'t> {
    ///
    /// Creates a new [`LazyGen`] for the given `input` string.
    ///
    pub fn new(input: &'t str) -> Self {
        Self {
            program: input,
            remaining: input,
            completed: false,
        }
    }
}

impl<'t> Generator for LazyGen<'t> {
    type Yield = (Token<'t>, usize);
    type Return = color_eyre::Result<()>;

    fn resume(self: Pin<&mut Self>, _arg: ()) -> GeneratorState<Self::Yield, Self::Return> {
        let this = Pin::into_inner(self);

        if this.completed {
            panic!("resumed genarator after completion wihtout reset");
        } else if this.remaining.is_empty() {
            this.completed = true;
            GeneratorState::Complete(Ok(()))
        } else {
            match next_token(this.remaining) {
                Ok((rest, token, size)) => {
                    this.remaining = rest.trim_matches(|ch: char| ch.is_ascii_whitespace());
                    GeneratorState::Yielded((token, size))
                }
                Err(err) => GeneratorState::Complete(Err(err)),
            }
        }
    }
}

impl<'t> Gen<'t> for LazyGen<'t> {
    fn tokens(&self) -> &'t str {
        self.program
    }

    fn reset(&mut self) {
        self.remaining = self.program;
    }
}

///
/// An eagerly collected token [`Gen`], if this generator is created succesfully, it will always
/// complete with [`Ok`].
///
#[derive(Debug)]
pub struct EagerGen<'t> {
    program: &'t str,
    yield_at: usize,
    tokens: Vec<(Token<'t>, usize)>,
}

impl<'a> EagerGen<'a> {
    ///
    /// Attempts creating a new [`EagerGen`] for the given `input` string.
    ///
    pub fn new(input: &'a str) -> color_eyre::Result<Self> {
        let mut lazy = LazyGen::new(input);
        let mut tokens = vec![];

        loop {
            match Pin::new(&mut lazy).resume(()) {
                GeneratorState::Yielded(yielded) => tokens.push(yielded),
                GeneratorState::Complete(res) => {
                    res?;
                    break;
                }
            }
        }

        Ok(Self {
            program: input,
            yield_at: 0,
            tokens,
        })
    }
}

impl<'t> Generator for EagerGen<'t> {
    type Yield = (Token<'t>, usize);
    type Return = color_eyre::Result<()>;

    fn resume(self: Pin<&mut Self>, _arg: ()) -> GeneratorState<Self::Yield, Self::Return> {
        let this = Pin::into_inner(self);

        match this.yield_at.cmp(&this.tokens.len()) {
            Ordering::Greater => {
                panic!("resumed genarator after completion wihtout reset");
            }
            Ordering::Equal => GeneratorState::Complete(Ok(())),
            _ => {
                let yielded = this.tokens[this.yield_at];
                this.yield_at += 1;
                GeneratorState::Yielded(yielded)
            }
        }
    }
}

impl<'t> Gen<'t> for EagerGen<'t> {
    fn tokens(&self) -> &'t str {
        self.program
    }

    fn reset(&mut self) {
        self.yield_at = 0;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lazy() {
        let mut gen = LazyGen::new("<//|.|>");

        let mut yields = vec![];
        for _ in 0..4 {
            yields.push(match Pin::new(&mut gen).resume(()) {
                GeneratorState::Yielded(yielded) => yielded,
                _ => panic!("less than 4 tokens yielded"),
            });
        }

        match Pin::new(&mut gen).resume(()) {
            GeneratorState::Complete(res) => res.expect("program is valid"),
            _ => panic!("more than 4 tokens yielded"),
        };

        assert_eq!(yields[0], (Token::In, 1));
        assert_eq!(yields[1], (Token::Split(None), 2));
        assert_eq!(yields[2], (Token::Slice((..).into()), 3));
        assert_eq!(yields[3], (Token::Out, 1));
    }

    #[test]
    fn eager() {
        let mut gen = EagerGen::new("<//|.|>").expect("the program is valid");

        let mut yields = vec![];
        for _ in 0..4 {
            yields.push(match Pin::new(&mut gen).resume(()) {
                GeneratorState::Yielded(yielded) => yielded,
                _ => panic!("less than 4 tokens yielded"),
            });
        }

        match Pin::new(&mut gen).resume(()) {
            GeneratorState::Complete(res) => res.expect("program is valid"),
            _ => panic!("more than 4 tokens yielded"),
        };

        assert_eq!(yields[0], (Token::In, 1));
        assert_eq!(yields[1], (Token::Split(None), 2));
        assert_eq!(yields[2], (Token::Slice((..).into()), 3));
        assert_eq!(yields[3], (Token::Out, 1));
    }

    #[test]
    fn lazy_error() {
        let mut gen = LazyGen::new("<//|.");

        let mut yields = vec![];
        for _ in 0..2 {
            yields.push(match Pin::new(&mut gen).resume(()) {
                GeneratorState::Yielded(yielded) => yielded,
                _ => panic!("less than 2 valid tokens yielded"),
            });
        }

        match Pin::new(&mut gen).resume(()) {
            GeneratorState::Complete(res) => res.expect_err("program is invalid"),
            _ => panic!("more than 2 valid tokens yielded"),
        };

        assert_eq!(yields[0], (Token::In, 1));
        assert_eq!(yields[1], (Token::Split(None), 2));
    }

    #[test]
    fn eager_error() {
        EagerGen::new("<//|.").expect_err("program is invalid");
    }

    #[test]
    fn lazy_reset() {
        let mut gen = LazyGen::new("<//|.|>");

        let mut yields = vec![];
        for at in 0..7 {
            yields.push(match Pin::new(&mut gen).resume(()) {
                GeneratorState::Yielded(yielded) => yielded,
                _ => panic!("less than 4 tokens yielded"),
            });

            if at == 2 {
                gen.reset();
            }
        }

        match Pin::new(&mut gen).resume(()) {
            GeneratorState::Complete(res) => res.expect("program is valid"),
            _ => panic!("more than 4 tokens yielded"),
        };

        assert_eq!(yields[0], (Token::In, 1));
        assert_eq!(yields[1], (Token::Split(None), 2));
        assert_eq!(yields[2], (Token::Slice((..).into()), 3));
        assert_eq!(yields[3], (Token::In, 1));
        assert_eq!(yields[4], (Token::Split(None), 2));
        assert_eq!(yields[5], (Token::Slice((..).into()), 3));
        assert_eq!(yields[6], (Token::Out, 1));
    }

    #[test]
    fn eager_reset() {
        let mut gen = EagerGen::new("<//|.|>").expect("the program is valid");

        let mut yields = vec![];
        for at in 0..7 {
            yields.push(match Pin::new(&mut gen).resume(()) {
                GeneratorState::Yielded(yielded) => yielded,
                _ => panic!("less than 4 tokens yielded"),
            });

            if at == 2 {
                gen.reset();
            }
        }

        match Pin::new(&mut gen).resume(()) {
            GeneratorState::Complete(res) => res.expect("program is valid"),
            _ => panic!("more than 4 tokens yielded"),
        };

        assert_eq!(yields[0], (Token::In, 1));
        assert_eq!(yields[1], (Token::Split(None), 2));
        assert_eq!(yields[2], (Token::Slice((..).into()), 3));
        assert_eq!(yields[3], (Token::In, 1));
        assert_eq!(yields[4], (Token::Split(None), 2));
        assert_eq!(yields[5], (Token::Slice((..).into()), 3));
        assert_eq!(yields[6], (Token::Out, 1));
    }
}
