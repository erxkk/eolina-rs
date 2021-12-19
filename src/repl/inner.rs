use super::Error;
use crate::{
    helper,
    io::Io,
    io::Kind,
    parse::EagerGen,
    program::{Context as ProgramContext, Value},
};
use std::{collections::VecDeque, ops::Generator, pin::Pin};

///
/// A repl context, used for interactively executing programs.
///
pub struct Context<'a> {
    io: &'a mut Io,
    exec_io: &'a mut Io,
    values: VecDeque<Value>,
}

impl<'a> Context<'a> {
    ///
    /// Creates a new [`Context`] with the given [`Io`] instances.
    ///
    pub fn new(io: &'a mut Io, exec_io: &'a mut Io) -> Self {
        Self {
            io,
            exec_io,
            values: VecDeque::new(),
        }
    }

    ///
    /// Continously runs this [`Context`] until the program is prompted to exit.
    ///
    /// **Note**: Program or command errors are **not** propagated.
    ///
    /// ### Returns
    ///
    /// * [`Ok`]
    ///   * the [`Context`] was successful run and exited normaly
    /// * [`Err`]
    ///   * the [`Context`] was not successful run
    ///
    pub fn run(&mut self) -> eyre::Result<()> {
        'outer: loop {
            let mut input = self.io.read_expect(">>> ");

            // truncate the '\n'
            if input.ends_with('\n') {
                input.truncate(input.len() - 1);
            }

            // skip
            if input.is_empty() {
                continue 'outer;
            }

            if let Some(input) = input.strip_prefix('!') {
                match self.command(input) {
                    Ok(shutdown) => {
                        if shutdown {
                            break 'outer Ok(());
                        }
                    }
                    Err(err) => {
                        self.io.write_expect(Kind::Error, None, err);
                    }
                }
                continue 'outer;
            }

            let gen = match EagerGen::new(&input) {
                Ok(gen) => gen,
                Err(err) => {
                    self.io.write_expect(Kind::Error, None, err);
                    continue 'outer;
                }
            };

            let mut program = ProgramContext::new(&input, gen, self.exec_io, &mut self.values);

            'inner: loop {
                match Pin::new(&mut program).resume(()) {
                    std::ops::GeneratorState::Yielded(_) => continue 'inner,
                    std::ops::GeneratorState::Complete(res) => {
                        match res {
                            Ok(_) => {}
                            Err(err) => self.io.write_expect(Kind::Error, None, err),
                        }
                        continue 'outer;
                    }
                }
            }
        }
    }

    ///
    /// Executes a command for this [`Context`].
    ///
    /// ### Returns
    ///
    /// * [`Ok`]
    ///   * the command was executed successfully, contains whether
    ///     or not the reply was prompted to shutdown
    /// * [`Err`]
    ///   * the command was not executed successfully contains the
    ///     error reason
    ///
    fn command(&mut self, cmd: &str) -> eyre::Result<bool> {
        match cmd {
            "help" | "h" => {
                let commands = [
                    "h | help     print all commands",
                    "q | queue    display the current queue",
                    "example      display examples",
                    "exit         exit the program",
                ];

                for cmd in commands {
                    self.io.write_expect(Kind::Output, None, cmd);
                }

                Ok(false)
            }
            "queue" | "q" => {
                self.io
                    .write_expect(Kind::Info, "queue", helper::fmt_iter(self.values.iter()));
                Ok(false)
            }
            "example" | "eg" => {
                let examples = [
                    "<>            echo program",
                    "<*>>          duplicate echo",
                    "<*[^][_]~>    orders by case, upper first",
                ];

                for eg in examples {
                    self.io.write_expect(Kind::Output, None, eg);
                }

                Ok(false)
            }
            "exit" => Ok(true),
            _ => eyre::bail!(Error::UnknownCommand(cmd.to_owned())),
        }
    }
}
