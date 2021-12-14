use super::{next_token, Error, Token};

///
/// A token iterator, attempts yielding tokens on every call to [`Iterator::next`].
/// Once a yield returned [`Some(Err(_))`], subsequent yields only return [`None`].
///
/// Use [`Iterator::collect::<Result<C, _>>()`] to collect into a `C`.
///
#[derive(Debug)]
pub struct TokenIter<'a> {
    input: &'a str,
    slice: &'a str,
    error: bool,
}

impl<'a> TokenIter<'a> {
    ///
    /// Creates a new [`TokenIter`] for the given `input` string.
    ///
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            slice: input,
            error: false,
        }
    }

    ///
    /// Returns whether this iterator has previously yielded an [`Err`].
    ///
    pub fn error(&self) -> bool {
        self.error
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error || self.slice.is_empty() {
            None
        } else {
            match next_token(self.slice) {
                Ok((rest, token)) => {
                    self.slice = rest;
                    self.error = false;
                    Some(Ok(token))
                }
                Err(err) => {
                    self.error = true;
                    Some(Err(err))
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn token_iter() {
        let stream = TokenIter::new("<>//|.|");
        let tokens = stream
            .collect::<Result<Vec<_>, _>>()
            .expect("the given tokens are valid");

        assert_eq!(
            tokens,
            vec![
                Token::In,
                Token::Out,
                Token::Split(None),
                Token::Slice((..).into())
            ]
        );
    }

    #[test]
    fn token_iter_error() {
        let stream = TokenIter::new("<>/|.");
        stream
            .collect::<Result<Vec<_>, _>>()
            .expect_err("`|.` is invalid");

        let mut stream = TokenIter::new("|.");
        stream.next();

        assert!(stream.error);
    }
}
