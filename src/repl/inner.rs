use super::Error;
use crate::{
    exec::{Context as ExecContext, Value},
    io::Io,
    io::Kind,
    io::Mode,
};
use std::{collections::VecDeque, process};

///
/// A repl context, used for storing and executing programs.
///
pub struct Context {
    io: Io,
    exec_io: Io,
    values: VecDeque<Value>,
}

impl Context {
    ///
    /// Creates a new [`Context`], with [`Mode::Colorful`].
    ///
    pub fn new() -> Self {
        Self {
            io: Io::with(
                Mode::Lean,
                (None, None),
                (None, None),
                ("[".to_owned(), "]: ".to_owned()),
            ),
            exec_io: Io::with(
                Mode::Lean,
                ("[".to_owned(), "]: ".to_owned()),
                ("[".to_owned(), "]: ".to_owned()),
                ("[".to_owned(), "]: ".to_owned()),
            ),
            values: VecDeque::new(),
        }
    }

    ///
    /// Continously runs this repl context until the program is prompted to exit.
    ///
    /// ### Returns
    ///
    /// * [`Ok(())`] if the repl was run successful
    /// * [`Err(_)`] if the repl failed
    ///
    pub fn run(&mut self) -> color_eyre::Result<()> {
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

            for res in ExecContext::new(&input, &mut self.exec_io, &mut self.values) {
                if let Some(err) = res.err() {
                    self.io.write_expect(Kind::Error, None, err);
                    continue 'outer;
                }
            }
        }
    }

    ///
    /// Executes a command for this context.
    ///
    /// ### Returns
    ///
    /// * [`Ok(())`] if the command was executed successfully
    /// * [`Err(string)`] if the command could not be executed
    ///   * `string` contains the error reason
    ///
    fn command(&mut self, cmd: &str) -> color_eyre::Result<()> {
        match cmd {
            "exit" | "quit" | "q" => process::exit(0),
            "help" | "h" | "?" => {
                // TODO: see multiline handling in crate::io
                self.io
                    .write_expect(Kind::Output, None, "exit | quit | q    exits the program");
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
            _ => eyre::bail!(Error::UnknownCommand(cmd.to_owned())),
        }
    }
}
