use super::Error;
use crate::{exec::Executor, io::Io, io::Kind, io::Mode};
use std::{collections::HashMap, process, rc::Rc, sync::Mutex};

// TODO: write tests
// TODO: allow spaces in programs when parsing `!s` (parse program arg as remainder)

///
/// A repl context, used for storing and executing programs.
///
pub struct Context {
    io: Io,
    exec_io: Rc<Mutex<Io>>,
    execs: HashMap<String, Executor>,
}

impl Context {
    ///
    /// Creates a new [`Context`], with [`Mode::Colorful`].
    ///
    pub fn new() -> Self {
        Self {
            io: Io::with(Mode::Colorful, None, None, "[".to_owned(), "]: ".to_owned()),
            exec_io: Rc::new(Mutex::new(Io::with(
                Mode::Colorful,
                "[".to_owned(),
                "]: ".to_owned(),
                "[".to_owned(),
                "]: ".to_owned(),
            ))),
            execs: HashMap::new(),
        }
    }

    ///
    /// Continously runs this repl context until the program is prompted to exit.
    ///
    /// ### Returns
    ///
    /// This function does not return.
    ///
    pub fn run(&mut self) -> ! {
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

            let mut exec = Executor::new(Rc::new(input));
            if let Err(err) = self.run_exec(&mut exec) {
                self.io.write_expect(Kind::Error, err, exec.input());
            }
        }
    }

    ///
    /// Executes and then resets a given [`Executor`] for this context.
    ///
    /// ### Returns
    ///
    /// * [`Ok(())`] if the executor was executed successfully
    /// * [`Err(string)`] if the executor could not be executed or reset
    ///   * `string` contains the error reason
    ///
    fn run_exec(&self, exec: &mut Executor) -> Result<(), Error> {
        for res in exec.iter(Rc::clone(&self.exec_io)) {
            if let Some(err) = res.err() {
                return Err(Error::exec(err));
            }
        }

        exec.reset()?;
        Ok(())
    }

    // TODO: clean up command handling

    ///
    /// Executes a command for this context.
    ///
    /// ### Returns
    ///
    /// * [`Ok(())`] if the command was executed successfully
    /// * [`Err(string)`] if the command could not be executed
    ///   * `string` contains the error reason
    ///
    fn command(&mut self, cmd: &str) -> Result<(), Error> {
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
            "list" | "ls" => {
                for (name, exec) in self.execs.iter() {
                    println!("    {} # {}", name, exec.input());
                }
                Ok(())
            }
            "example" | "eg" => {
                self.io
                    .write_expect(Kind::Output, "    <>          # echo program", None);
                self.io
                    .write_expect(Kind::Output, "    <*>>        # duplicate echo", None);
                self.io.write_expect(
                    Kind::Output,
                    "    <*[^][_]~>  # orders by case, upper first",
                    None,
                );
                Ok(())
            }
            x => match x.as_bytes() {
                [b's', b' ', ..] => {
                    let x = &x[2..];
                    let mut iter = x.split_ascii_whitespace();

                    let name = iter
                        .next()
                        .ok_or_else(|| Error::missing_param("name", 0))?
                        .to_owned();

                    let program = iter
                        .next()
                        .ok_or_else(|| Error::missing_param("program", 1))?;
                    let program = Rc::new(program.to_owned());
                    self.execs.insert(name, Executor::new(Rc::clone(&program)));
                    Ok(())
                }
                [b'c', b' ', ..] => {
                    let x = &x[2..];
                    let mut iter = x.split_ascii_whitespace();

                    let name = iter.next().ok_or_else(|| Error::missing_param("name", 0))?;

                    // move out executor to avoid doublemutable borrow
                    match self.execs.remove(name) {
                        Some(mut exec) => {
                            self.io.write_expect(
                                Kind::Info,
                                format!("running: `{}`", exec.input()),
                                None,
                            );

                            let res = self.run_exec(&mut exec);
                            self.execs.insert(name.to_string(), exec);
                            res
                        }
                        None => Err(Error::unknown_program(x.to_owned())),
                    }
                }
                [b'r', b' ', ..] => {
                    let x = &x[2..];
                    let mut iter = x.split_ascii_whitespace();

                    let name = iter.next().ok_or_else(|| Error::missing_param("name", 0))?;

                    match self.execs.remove(name) {
                        Some(exec) => {
                            println!("removed program: `{}`: {}", name, exec.input());
                            Ok(())
                        }
                        None => Err(Error::unknown_program(x.to_owned())),
                    }
                }
                _ => Err(Error::unknown_command(x.to_owned())),
            },
        }
    }
}
