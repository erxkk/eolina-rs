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
            io: Io::with(Mode::Colorful, None, None, "[".to_owned(), "]: ".to_owned()),
            exec_io: Io::with(
                Mode::Colorful,
                "[".to_owned(),
                "]: ".to_owned(),
                "[".to_owned(),
                "]: ".to_owned(),
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
    /// * [`Err(string)`] if the repl failed or a exec failed
    ///   * `string` contains the error reason
    ///
    pub fn run(&mut self) -> color_eyre::Result<()> {
        'outer: loop {
            let mut input = self.io.read_expect(">>> ", None);

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
                        self.io.write_expect(Kind::Error, err, None);
                    }
                }
                continue 'outer;
            }

            for res in ExecContext::new(&input, &mut self.exec_io, &mut self.values) {
                if let Some(err) = res.err() {
                    eyre::bail!(err);
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
                    .write_expect(Kind::Output, "exit | quit | q    exits the program", None);
                self.io
                    .write_expect(Kind::Output, "help | h | ?       prints all commands", None);
                self.io.write_expect(
                    Kind::Output,
                    "s                  saves a program `!s sort <*[^][_]~>`",
                    None,
                );
                self.io.write_expect(
                    Kind::Output,
                    "c                  calls a program `!s sort`",
                    None,
                );
                self.io.write_expect(
                    Kind::Output,
                    "r                  removes a program `!r sort`",
                    None,
                );

                Ok(())
            }
            "example" | "eg" => {
                self.io
                    .write_expect(Kind::Output, "<>            echo program", None);
                self.io
                    .write_expect(Kind::Output, "<*>>          duplicate echo", None);
                self.io.write_expect(
                    Kind::Output,
                    "<*[^][_]~>    orders by case, upper first",
                    None,
                );
                Ok(())
            }
            _ => eyre::bail!(Error::UnknownCommand(cmd.to_owned())),
        }
    }
}
