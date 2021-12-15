use super::{next_token, Token};

///
/// A token iterator, attempts yielding tokens on every call to [`Iterator::next`].
/// Once a yield returned [`Some(Err(_))`], subsequent yields only return [`None`].
///
/// Use [`Iterator::collect::<Result<C, _>>()`] to collect into a `C`.
///
#[derive(Debug)]
pub struct Iter<'a> {
    slice: &'a str,
    error: bool,
}

impl<'a> Iter<'a> {
    ///
    /// Creates a new [`TokenIter`] for the given `input` string.
    ///
    pub fn new(input: &'a str) -> Self {
        Self {
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

    ///
    /// Returns whether this iterator has previously yielded an [`Err`].
    ///
    pub fn slice(&self) -> &'a str {
        self.slice
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = color_eyre::Result<(Token, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error || self.slice.is_empty() {
            None
        } else {
            match next_token(self.slice) {
                Ok((rest, token, pos)) => {
                    self.slice = rest.trim_matches(|ch: char| ch.is_ascii_whitespace());
                    self.error = false;
                    Some(Ok((token, pos)))
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
        let stream = Iter::new("<>//|.|");
        let tokens = stream
            .collect::<Result<Vec<_>, _>>()
            .expect("the given tokens are valid");

        assert_eq!(
            tokens,
            vec![
                (Token::In, 1),
                (Token::Out, 1),
                (Token::Split(None), 2),
                (Token::Slice((..).into()), 3)
            ]
        );
    }

    #[test]
    fn token_iter_error() {
        let stream = Iter::new("<>/|.");
        stream
            .collect::<Result<Vec<_>, _>>()
            .expect_err("`|.` is invalid");

        let mut stream = Iter::new("|.");
        stream.next();

        assert!(stream.error);
    }
}
