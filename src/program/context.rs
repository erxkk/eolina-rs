use super::{func, Error, Value};
use crate::{
    io::{Io, Kind},
    parse::{Gen, Token},
};
use crossterm::style::Stylize;
use std::{
    collections::VecDeque,
    ops::{Generator, GeneratorState},
    pin::Pin,
};

///
/// The excution context for a program.
///
#[derive(Debug)]
pub struct Context<'p, 'io, 'v, G> {
    prev_len: usize,
    input: &'p str,
    io: &'io mut Io,
    gen: G,
    values: &'v mut VecDeque<Value>,
}

impl<'p, 'io, 'v, G> Context<'p, 'io, 'v, G>
where
    G: Gen<'p> + Unpin,
{
    ///
    /// Creates a new [`Context`] with the given token generator and an empty queue.
    ///
    pub fn new(input: &'p str, gen: G, io: &'io mut Io, values: &'v mut VecDeque<Value>) -> Self {
        Self {
            prev_len: 0,
            input,
            io,
            gen,
            values,
        }
    }

    ///
    /// Pops the front of the queue or returns an [`Error::QueueEmpty`] error.
    ///
    /// ### Returns
    ///
    /// * [`Ok`]
    ///   * the queue was not empty, contains the [`Value`]
    /// * [`Err`]
    ///   * the queue was empty, contains the [`Error`]
    ///
    fn pop_queue(&mut self) -> Result<Value, Error> {
        self.values.pop_front().ok_or(Error::QueueEmpty)
    }

    ///
    /// Returns a colored [`String`] indicating where the programm is curently in execution.
    /// The [`Io`] instance will ignore this when it's `mode` is not set to
    /// [`Mode::Colorful`](crate::io::Mode::Colorful).
    ///
    /// ### Returns
    ///
    /// Returns the stylized [`String`], with the current [`Token`] highlighted.
    ///
    /// ### Panics
    ///
    /// Panics if the program is empty, which are however usually not parsed.
    ///
    fn get_context(&self) -> String {
        let (s, r) = self.input.split_at(self.prev_len);
        let c = &r[0..1];
        let r = &r[1..];

        format!("{}{}{}", s, c.green(), r)
    }

    ///
    /// Executes the function associated with the given [`Token`].
    ///
    /// ### Returns
    ///
    /// * [`Ok`]
    ///   * the function was successfully executed
    /// * [`Err`]
    ///   * the function was not successfully executed
    ///   * an IO operation failed
    ///
    fn next_token(&mut self, token: Token) -> eyre::Result<()> {
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

    // TODO: doc
    ///
    ///
    ///
    pub fn run(mut self) -> eyre::Result<()> {
        loop {
            match Pin::new(&mut self).resume(()) {
                GeneratorState::Yielded(_) => continue,
                GeneratorState::Complete(res) => break res.map_err(|err| err.into()),
            }
        }
    }
}

impl<'p, 'io, 'v, G> Generator for Context<'p, 'io, 'v, G>
where
    G: Gen<'p> + Unpin,
{
    type Yield = ();
    type Return = eyre::Result<()>;

    ///
    /// Attempts to parse and execute the next instruction.
    ///
    /// ### Returns
    ///
    /// * [`GeneratorState::Yielded`]
    ///   * the next instruction was successfully parsed and executed
    /// * [`GeneratorState::Complete(Ok)`]
    ///   * the generator has completed
    /// * [`GeneratorState::Complete(Err)`]
    ///   * the next instruction could not be parsed or executed,
    ///     contains the [`Error`] that occured
    ///
    fn resume(self: Pin<&mut Self>, _arg: ()) -> GeneratorState<Self::Yield, Self::Return> {
        let this = Pin::into_inner(self);

        match Pin::new(&mut this.gen).resume(()) {
            GeneratorState::Yielded((token, pos)) => match this.next_token(token) {
                Ok(()) => {
                    this.prev_len += pos;
                    GeneratorState::Yielded(())
                }
                Err(inner) => GeneratorState::Complete(Err(inner)),
            },
            GeneratorState::Complete(inner) => {
                GeneratorState::Complete(inner.map_err(|err| err.into()))
            }
        }
    }
}
