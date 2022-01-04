use super::{func, Error, Value};
use crate::{
    cli,
    parse::Gen,
    token::{RepeatKind, Token},
};
use crossterm::style::Stylize;
use std::{
    collections::VecDeque,
    io::{self, Write},
    mem::MaybeUninit,
    ops::{Generator, GeneratorState},
    pin::Pin,
};

///
/// The excution context for a program.
///
#[derive(Debug)]
pub struct Context<'p, 'v, G> {
    ///
    /// The current token byte start index and length in `input`.
    ///
    current_token: (usize, usize),

    ///
    /// The input program.
    ///
    input: &'p str,

    ///
    /// The token generator.
    ///
    gen: G,

    ///
    /// The command line args if any were passed.
    ///
    args: Option<Vec<String>>,

    ///
    /// The value queue to use.
    ///
    values: &'v mut VecDeque<Value>,

    ///
    /// Whether or not this context is used in a repl.
    ///
    is_repl: bool,
}

impl<'p, 'v, G> Context<'p, 'v, G> {
    ///
    /// Creates a new [`Context`] with the given token generator and an empty queue.
    ///
    pub fn new(
        input: &'p str,
        gen: G,
        args: Option<Vec<String>>,
        values: &'v mut VecDeque<Value>,
        is_repl: bool,
    ) -> Self {
        Self {
            current_token: (0, 0),
            input,
            gen,
            args,
            values,
            is_repl,
        }
    }

    ///
    /// Pops the front of the queue or returns a [`Error::QueueTooShort`] error.
    ///
    /// ### Returns
    ///
    /// * [`Ok`]
    ///   * the queue had enough elements, contains the [`Value`]s in the order they were popped
    ///     off
    /// * [`Err`]
    ///   * the queue was too short, contains the [`Error`]
    ///
    fn pop_queue<const N: usize>(&mut self) -> Result<[Value; N], Error> {
        if self.values.len() < N {
            Err(Error::QueueTooShort(N, self.values.len()))
        } else {
            // SAFETY:
            // * The contained values are `MaybeUninit` and therefore cause no UB if not
            //   initialized
            // * We have enough elements because we check for at least N before
            let mut ret: [MaybeUninit<Value>; N] = unsafe { MaybeUninit::uninit().assume_init() };

            // All memory is on the stack and requires no special drop handling and can be left
            // undropped without leaking memory if a panic occurs
            for n in &mut ret[..] {
                n.write(self.values.pop_front().unwrap());
            }

            // SAFETY:
            // * All values must be valid if we reached this part of the program because we wrote
            //   exactly N elements
            Ok(unsafe { MaybeUninit::array_assume_init(ret) })
        }
    }

    ///
    /// Pushes the given [`Value`]s on the back of the queue in the order they're given.
    ///
    fn push_queue(&mut self, values: impl IntoIterator<Item = Value>) {
        for value in values {
            self.values.push_back(value);
        }

        self.log_queue();
    }

    ///
    /// Logs the current queue as debug.
    ///
    fn log_queue(&self) {
        log::debug!("[{}]: queue: {:?}", self.get_context(), self.values);
    }

    ///
    /// Returns a [`String`] indicating where the programm is curently in execution.
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
        let (s, r) = self.input.split_at(self.current_token.0);
        let c = &r[0..self.current_token.1];
        let r = &r[self.current_token.1..];

        if *cli::IS_FANCY {
            format!("{}{}{}", s.green(), c.cyan(), r.grey())
        } else {
            format!("{}'{}'{}", s, c, r)
        }
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
    fn exec_token(&mut self, token: Token) -> color_eyre::Result<()> {
        match token {
            Token::Repeat(repeat) => match repeat.kind() {
                RepeatKind::In => {
                    for _ in 0..repeat.count() {
                        let prompt = |this: &mut Self| -> io::Result<()> {
                            if this.is_repl {
                                if *cli::IS_FANCY {
                                    eprint!("[{}] [{}]: ", "inp".green(), this.get_context())
                                } else {
                                    eprint!("[inp] [{}]: ", this.get_context())
                                }
                                io::stdout().flush()?;
                            }
                            Ok(())
                        };

                        let val = if let Some(value) = self.args.as_mut().and_then(|vec| vec.pop())
                        {
                            prompt(self)?;
                            value
                        } else {
                            prompt(self)?;

                            let mut input = String::new();
                            io::stdin().read_line(&mut input)?;

                            // truncate the '\n'
                            if input.ends_with('\n') {
                                input.truncate(input.len() - 1);
                            }

                            input
                        };

                        self.push_queue([Value::String(val)]);
                    }
                }
                RepeatKind::Out => {
                    for _ in 0..repeat.count() {
                        let [val] = self.pop_queue()?;
                        if self.is_repl {
                            if *cli::IS_FANCY {
                                eprintln!("[{}] [{}]: {}", "out".green(), self.get_context(), val)
                            } else {
                                eprintln!("[out] [{}]: {}", self.get_context(), val)
                            }
                        } else {
                            println!("{}", val);
                        }
                    }
                }
                RepeatKind::Concat => {
                    for _ in 0..repeat.count() {
                        let [val1, val2] = self.pop_queue()?;
                        let ret = func::concat(val1, val2)?;
                        self.push_queue([ret]);
                    }
                }
                RepeatKind::Duplicate => {
                    let [val] = self.pop_queue()?;
                    self.push_queue((0..repeat.count()).map(|_| val.clone()));
                }
                RepeatKind::Rotate => {
                    self.values.rotate_left(repeat.count());
                    self.log_queue();
                }
            },
            Token::Inline(inline) => {
                todo!()
            }
            Token::Transform(split) => {
                let [val] = self.pop_queue()?;
                let ret = func::transform(val, split)?;
                self.push_queue([ret]);
            }
            Token::Index(idx) => {
                let [val] = self.pop_queue()?;
                let ret = func::index(val, idx)?;
                self.push_queue([ret]);
            }
            Token::Slice(range) => {
                let [val] = self.pop_queue()?;
                let ret = func::slice(val, range)?;
                self.push_queue([ret]);
            }
        }
        Ok(())
    }
}

impl<'p, 'v, G> Context<'p, 'v, G>
where
    G: Gen<'p> + Unpin,
{
    ///
    /// Consumes this [`Context`] and attempts executed it to completion.
    ///
    /// ### Returns
    ///
    /// * [`Ok`]
    ///   * the context was successfully executed
    /// * [`Err`]
    ///   * the context was not successfully executed
    ///
    pub fn run(mut self) -> color_eyre::Result<()> {
        loop {
            match Pin::new(&mut self).resume(()) {
                GeneratorState::Yielded(_) => continue,
                GeneratorState::Complete(res) => break res,
            }
        }
    }
}

impl<'p, 'v, G> Generator for Context<'p, 'v, G>
where
    G: Gen<'p> + Unpin,
{
    type Yield = ();
    type Return = color_eyre::Result<()>;

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
            GeneratorState::Yielded((token, len)) => {
                this.current_token.1 = len;

                let res = this.exec_token(token);
                this.current_token.0 += len;

                match res {
                    Ok(()) => GeneratorState::Yielded(()),
                    Err(inner) => GeneratorState::Complete(Err(inner)),
                }
            }
            GeneratorState::Complete(inner) => GeneratorState::Complete(inner),
        }
    }
}
