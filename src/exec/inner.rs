use super::{func, Error, Value};
use crate::{
    helper::Immutable,
    io::{Io, Kind},
    parse::{Iter as TokenIter, Token},
};
use std::{collections::VecDeque, ops::Deref};

///
/// The excution context for a program.
///
#[derive(Debug)]
pub struct Context {
    // this type is used as a marker that this will never be mutated
    // through a reference
    input: Immutable<String>,
    values: VecDeque<Value>,
}

impl Context {
    ///
    /// Creates a new [`Executor`] with the given `tokens` and an empty queue.
    ///
    pub fn new(input: String) -> Self {
        Self {
            input: Immutable::new(input),
            values: VecDeque::new(),
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
    /// Resets this executor, resetting it's values.
    ///
    pub fn reset(&mut self) {
        self.values.clear();
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
    fn next_token(&mut self, io: &mut Io, token: Token) -> Result<(), Error> {
        match token {
            Token::In => {
                let mut val = io.read_expect(" in", &**self.input);

                // truncate the '\n'
                if val.ends_with('\n') {
                    val.truncate(val.len() - 1);
                }

                self.values.push_back(Value::String(val));
            }
            Token::Out => {
                let val = self.pop_queue()?;
                io.write_expect(Kind::Output, val, &**self.input);
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

    ///
    /// Returns an [`ExecutorIter`] that executes the
    ///
    pub fn iter<'b>(&mut self, io: &'b mut Io) -> Iter<'_, 'b> {
        Iter::new(self, io)
    }
}

#[derive(Debug)]
pub struct Iter<'a, 'b> {
    exec: &'a mut Context,
    io: &'b mut Io,
    tokens: TokenIter<'a>,
}

impl<'a, 'b> Iter<'a, 'b> {
    ///
    /// Creates a new [`ExecutorIter`] for the given [`Executor`] and [`Rc<Mutex<IoContext>>`] or [`None`].
    ///
    fn new(exec: &'a mut Context, io: &'b mut Io) -> Self {
        // TODO: this design can be improved to remove then need for unsafe here

        // SAFTEY:
        // * The String is not mutated by this `ExecutorIter`
        // * The lifetime of the String is tied to it's executors lifetime,
        //   and therefore at least valid for as long as this mutable refernce
        // * The String is never mutated after it's creation because it is wrapped in `Immutable`
        let slice: &'a str =
            unsafe { std::mem::transmute(AsRef::<str>::as_ref(exec.input.deref())) };

        Self {
            exec,
            io,
            tokens: TokenIter::new(slice),
        }
    }
}

// TODO: use a generator here at some point
impl<'a, 'b> Iterator for Iter<'a, 'b> {
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
    fn next(&mut self) -> Option<Self::Item> {
        if self.tokens.error() {
            None
        } else {
            match self.tokens.next()? {
                Ok(token) => Some(self.exec.next_token(self.io, token)),
                Err(inner) => Some(Err(Error::parse(inner))),
            }
        }
    }
}
