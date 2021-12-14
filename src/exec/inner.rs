use super::{func, Error, Value};
use crate::parse::{Token, TokenIter};
use std::{
    collections::VecDeque,
    io::{self, Write},
    rc::Rc,
};

///
/// Lazily yields tokens from a given input and interprets them
///
#[derive(Debug)]
pub struct Executor {
    input: Rc<String>,
    values: VecDeque<Value>,
    tokens: TokenIter,
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
    /// Returns if this executor encountered an error.
    ///
    pub fn error(&self) -> bool {
        self.tokens.error()
    }

    ///
    /// Resets this executor, resetting it's instruction iterator clears it's values.
    ///
    /// ### Returns
    ///
    /// * [`Ok(())`] if the executor could be reset
    /// * [`Err(str)`] if the executor could not be reset
    ///   * `str` contains the error reason
    ///
    pub fn reset(&mut self) -> Result<(), String> {
        if self.tokens.error() {
            Err(format!("the program must be malformed `{}`", self.input))
        } else {
            self.tokens = TokenIter::new(Rc::clone(&self.input));
            self.values.clear();
            Ok(())
        }
    }

    ///
    /// Pops the front of the queue or returns a empty queue error.
    ///
    /// ### Returns
    ///
    /// * [`Ok(value)`] if there was a value to pop
    ///   * `value` will contain the value fromt he head of the queue
    /// * [`Err(error)`] if there was no value in the queue
    ///   * `error` will contain an error of kind `EmptyQueue`
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
                print!(" in: ");
                io::stdout().flush()?;

                io::stdin().read_line(&mut val)?;

                // truncate the '\n'
                if val.ends_with('\n') {
                    val.truncate(val.len() - 1);
                }

                self.values.push_back(Value::String(val));
            }
            Token::Out => {
                let val = self.pop_queue()?;
                println!("out: {}", val);
            }
            Token::Rotate(num) => {
                self.values.rotate_left(num);
            }
            Token::Split(split) => {
                let val = self.pop_queue()?;
                let ret = func::split(val, split)?;
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
            Token::IsVowel => {
                let val = self.pop_queue()?;
                let ret = func::is_vowel(val)?;
                self.values.push_back(ret);
            }
            Token::IsConso => {
                let val = self.pop_queue()?;
                let ret = func::is_conso(val)?;
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
            Token::Map(map) => {
                let val = self.pop_queue()?;
                let ret = func::map(val, map)?;
                self.values.push_back(ret);
            }
            Token::Filter(filter) => {
                let val = self.pop_queue()?;
                let ret = func::filter(val, filter)?;
                self.values.push_back(ret);
            }
            Token::Slice(range) => {
                let val = self.pop_queue()?;
                let ret = func::slice(val, range)?;
                self.values.push_back(ret);
            }
        }
        Ok(())
    }
}

impl Iterator for Executor {
    type Item = Result<(), Error>;
    ///
    /// Advances this executor to the next instruction and attempts executing it.
    ///
    /// ### Returns
    ///
    /// * [`Some(Ok(()))`] if the current instruction could be executed
    /// * [`Some(Err(error))`] if the next instruction is invalid or could not be executed
    ///   * `error` contains the [`Error`] that occured during execution
    /// * [`None`] if the previous result was an [`Error`] or the all instructions were executed
    ///   * `self::error` returns whether or not an error was encountered before
    ///

    // TODO: use a generator here at some point
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
