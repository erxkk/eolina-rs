use super::Error;
use crate::{
    helper,
    io::Io,
    io::Kind,
    parse::LazyGen,
    program::{Context as ProgramContext, Value},
};
use std::{collections::VecDeque, ops::Generator, pin::Pin, process};

///
/// A repl context, used for storing and executing programs.
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
    pub fn run(&mut self) -> Result<(), Error> {
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
                    Ok(_) => {}
                    Err(err) => {
                        self.io.write_expect(Kind::Error, None, err);
                    }
                }
                continue 'outer;
            }

            let mut program = ProgramContext::new(
                &input,
                LazyGen::new(&input),
                self.exec_io,
                &mut self.values,
            );

            'inner: loop {
                match Pin::new(&mut program).resume(()) {
                    std::ops::GeneratorState::Yielded(_) => continue 'inner,
                    std::ops::GeneratorState::Complete(res) => {
                        match res {
                            Ok(_) => {}
                            Err(err) => self.io.write_expect(Kind::Error, None, err),
                        }
                        break 'inner;
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
    ///   * the command was executed successfully
    /// * [`Err`]
    ///   * the command was not executed successfully contains the
    ///     error reason
    ///
    fn command(&mut self, cmd: &str) -> Result<(), Error> {
        match cmd {
            "exit" | "e" => process::exit(0),
            "help" | "h" | "?" => {
                // TODO: see multiline handling in crate::io
                self.io
                    .write_expect(Kind::Output, None, "exit | e           exits the program");
                self.io
                    .write_expect(Kind::Output, None, "help | h | ?       prints all commands");
                self.io.write_expect(
                    Kind::Output,
                    None,
                    "s                  saves a program `!s sort <*[^][_]~>`",
                );
                self.io.write_expect(
                    Kind::Output,
                    None,
                    "c                  calls a program `!s sort`",
                );
                self.io.write_expect(
                    Kind::Output,
                    None,
                    "r                  removes a program `!r sort`",
                );

                Ok(())
            }
            "queue" | "q" => {
                self.io
                    .write_expect(Kind::Info, "queue", helper::fmt_iter(self.values.iter()));
                Ok(())
            }
            "example" | "eg" => {
                self.io
                    .write_expect(Kind::Output, None, "<>            echo program");
                self.io
                    .write_expect(Kind::Output, None, "<*>>          duplicate echo");
                self.io.write_expect(
                    Kind::Output,
                    None,
                    "<*[^][_]~>    orders by case, upper first",
                );
                Ok(())
            }
            _ => Err(Error::UnknownCommand(cmd.to_owned())),
        }
    }
}
