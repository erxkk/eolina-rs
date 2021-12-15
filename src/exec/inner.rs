use super::{func, Error, Value};
use crate::{
    io::{Io, Kind},
    parse::{Iter as TokenIter, Token},
};
use crossterm::style::Stylize;
use std::collections::VecDeque;

///
/// The excution context for a program.
///
#[derive(Debug)]
pub struct Context<'p, 'io, 'v> {
    prev_len: usize,
    input: &'p str,
    io: &'io mut Io,
    tokens: TokenIter<'p>,
    values: &'v mut VecDeque<Value>,
}

impl<'p, 'io, 'v> Context<'p, 'io, 'v> {
    ///
    /// Creates a new [`Context`] with the given `tokens` and an empty queue.
    ///
    pub fn new(input: &'p str, io: &'io mut Io, values: &'v mut VecDeque<Value>) -> Self {
        Self {
            prev_len: 0,
            input,
            io,
            tokens: TokenIter::new(input),
            values,
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
    fn pop_queue(&mut self) -> color_eyre::Result<Value> {
        Ok(self.values.pop_front().ok_or(Error::QueueEmpty)?)
    }

    ///
    /// Returns a colored [`String`] indicating where the programm is curently in execution.
    ///
    /// ### Returns
    ///
    /// Returns the stylized [`String`].
    ///
    /// ### Panics
    /// Panics if the program was not yet started or is empty.
    ///
    fn get_context(&self) -> String {
        let (s, r) = self.input.split_at(self.prev_len - 1);
        let c = &r[0..1];
        let r = &r[1..];

        format!("{}{}{}", s, c.green(), r)
    }

    ///
    /// Advances to the next token.
    ///
    fn next_token(&mut self, token: Token) -> color_eyre::Result<()> {
        match token {
            Token::In => {
                let mut val = self.io.read_expect(self.get_context().as_str());

                // truncate the '\n'
                if val.ends_with('\n') {
                    val.truncate(val.len() - 1);
                }

                self.values.push_back(Value::String(val));
            }
            Token::Out => {
                let val = self.pop_queue()?;
                self.io
                    .write_expect(Kind::Output, self.get_context().as_str(), val);
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

// TODO: use a generator here at some point
impl<'p, 'io, 'v> Iterator for Context<'p, 'io, 'v> {
    type Item = color_eyre::Result<()>;
    ///
    /// Attempts parsing and executing the next instruction.
    ///
    /// ### Returns
    ///
    /// * [`Some(Ok(()))`] if the current instruction could be parsed and executed
    /// * [`Some(Err(error))`] if the next instruction is invalid or could not be parsed or executed
    ///   * `error` contains the [`Error`] that occured during execution
    /// * [`None`] if the previous result was an [`Error`] or the all instructions were executed
    ///   * `self::error` returns whether or not an error was encountered before
    ///
    fn next(&mut self) -> Option<Self::Item> {
        if self.tokens.error() {
            None
        } else {
            match self.tokens.next()? {
                Ok((token, pos)) => {
                    self.prev_len += pos;
                    Some(self.next_token(token))
                }
                Err(inner) => Some(Err(inner)),
            }
        }
    }
}
