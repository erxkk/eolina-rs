use super::{next_token, Error, Token};
use std::rc::Rc;

///
/// A token iterator, attempts yielding tokens on every call to [`Iterator::next`].
/// Once a yield returned [`Some(Err(_))`], subsequent yields only return [`None`].
///
/// Use [`Iterator::collect::<Result<C, _>>()`] to collect into a `C`.
///
#[derive(Debug)]
pub struct TokenIter {
    input: Rc<String>,
    index: usize,
    error: bool,
}

impl TokenIter {
    ///
    /// Creates a new [`TokenIter`] for the given `input` string.
    ///
    pub fn new(input: Rc<String>) -> Self {
        Self {
            input,
            index: 0,
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

impl Iterator for TokenIter {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error || self.input[self.index..].is_empty() {
            None
        } else {
            match next_token(&self.input[self.index..]) {
                Ok((rest, token)) => {
                    self.index = if rest.is_empty() {
                        self.input.len()
                    } else {
                        self.input.find(rest).unwrap()
                    };

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
        let stream = TokenIter::new("<>/|.|".to_owned().into());
        let tokens = stream
            .collect::<Result<Vec<_>, _>>()
            .expect("the given tokens are valid");

        assert_eq!(
            tokens,
            vec![
                Token::In,
                Token::Out,
                Token::Split,
                Token::Slice(None, None)
            ]
        );
    }

    #[test]
    fn token_iter_error() {
        let stream = TokenIter::new("<>/|.".to_owned().into());
        stream
            .collect::<Result<Vec<_>, _>>()
            .expect_err("`|.` is invalid");

        let mut stream = TokenIter::new("|.".to_owned().into());
        stream.next();

        assert!(stream.error);
    }
}
