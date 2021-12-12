use super::{func, Error, Value};
use crate::parse::{Token, TokenIter};
use std::{
    collections::VecDeque,
    io::{self, Write},
    rc::Rc,
};

///
/// Lazily yields tokens from a `&'a str` and interprets them
///
#[derive(Debug)]
pub struct Executor {
    input: Rc<String>,
    values: VecDeque<Value>,
    tokens: TokenIter,
    stdin: io::Stdin,
    stdout: io::Stdout,
}

impl Executor {
    ///
    /// Creates a new [`Executor`] with the given `tokens` and an empty queue.
    ///
    pub fn new(input: Rc<String>) -> Self {
        let clone = Rc::clone(&input);
        Self {
            input,
            values: VecDeque::new(),
            tokens: TokenIter::new(clone),
            stdin: io::stdin(),
            stdout: io::stdout(),
        }
    }

    ///
    /// Returns the input this interpreter is interpreting.
    ///
    pub fn input(&self) -> &str {
        &self.input
    }

    ///
    /// Returns the current values.
    ///
    pub fn values(&self) -> &VecDeque<Value> {
        &self.values
    }

    ///
    /// Resets this executor, flushing it's output and clears it's values.
    ///
    pub fn reset(&mut self) -> Result<(), io::Error> {
        self.tokens = TokenIter::new(Rc::clone(&self.input));
        self.values.clear();
        self.stdout.flush()?;
        Ok(())
    }

    ///
    /// Pops the front of the queue or returns a empty queue error.
    ///
    fn pop_queue(&mut self) -> Result<Value, Error> {
        self.values.pop_front().ok_or_else(Error::empty)
    }

    ///
    /// Advances to the next token.
    ///
    fn next_token(&mut self, token: Token) -> Result<(), Error> {
        match token {
            Token::In => {
                let mut val = String::new();
                write!(self.stdout, " in: ")?;
                self.stdout.flush()?;
                self.stdin.read_line(&mut val)?;
                self.values.push_back(Value::String(val));
            }
            Token::Out => {
                let val = self.pop_queue()?;
                writeln!(self.stdout, "out: {}", val)?;
            }
            Token::Rotate => {
                self.values.rotate_left(1);
            }
            Token::Split => {
                let val = self.pop_queue()?;
                let ret = func::split(val)?;
                self.values.push_back(ret);
            }
            Token::Join => {
                let val = self.pop_queue()?;
                let ret = func::join(val)?;
                self.values.push_back(ret);
            }
            Token::Concat => {
                let val1 = self.pop_queue()?;
                let val2 = self.pop_queue()?;
                let ret = func::concat(val1, val2)?;
                self.values.push_back(ret);
            }
            Token::Copy => {
                let val = self.pop_queue()?;
                let copy = val.clone();
                self.values.push_back(val);
                self.values.push_back(copy);
            }
            Token::Vowel => {
                let val = self.pop_queue()?;
                let ret = func::is_vowel(val)?;
                self.values.push_back(ret);
            }
            Token::Conso => {
                let val = self.pop_queue()?;
                let ret = func::is_conso(val)?;
                self.values.push_back(ret);
            }
            Token::ToUpper => {
                let val = self.pop_queue()?;
                let ret = func::to_upper(val)?;
                self.values.push_back(ret);
            }
            Token::ToLower => {
                let val = self.pop_queue()?;
                let ret = func::to_lower(val)?;
                self.values.push_back(ret);
            }
            Token::IsUpper => {
                let val = self.pop_queue()?;
                let ret = func::is_upper(val)?;
                self.values.push_back(ret);
            }
            Token::IsLower => {
                let val = self.pop_queue()?;
                let ret = func::is_lower(val)?;
                self.values.push_back(ret);
            }
            Token::Filter(filter) => {
                let val = self.pop_queue()?;
                let ret = func::filter(val, filter)?;
                self.values.push_back(ret);
            }
            Token::Slice(lower, upper) => {
                let val = self.pop_queue()?;
                let ret = func::slice(val, lower, upper)?;
                self.values.push_back(ret);
            }
        }
        Ok(())
    }
}

impl Iterator for Executor {
    type Item = Result<(), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tokens.error() {
            None
        } else {
            match self.tokens.next()? {
                Ok(token) => Some(self.next_token(token)),
                Err(inner) => Some(Err(Error::parse(inner))),
            }
        }
    }
}

// TODO: tests
