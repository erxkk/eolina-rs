use crate::{
    cli,
    parse::EagerGen,
    program::{Context as ProgramContext, Value},
};
use std::{collections::VecDeque, io::Write, ops::Generator, pin::Pin};

///
/// A repl context, used for interactively executing programs.
///
pub struct Context {
    values: VecDeque<Value>,
}

impl Context {
    ///
    /// Creates a new [`Context`] with the given [`Io`] instances.
    ///
    pub fn new() -> Self {
        Self {
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
            print!(">>> ");
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

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
                        log::error!("{}", err);
                    }
                }
                continue 'outer;
            }

            let gen = match EagerGen::new(&input) {
                Ok(gen) => gen,
                Err(err) => {
                    log::error!("{}", err);
                    continue 'outer;
                }
            };

            let mut program = ProgramContext::new(&input, gen, None, &mut self.values, true);

            'inner: loop {
                match Pin::new(&mut program).resume(()) {
                    std::ops::GeneratorState::Yielded(_) => continue 'inner,
                    std::ops::GeneratorState::Complete(res) => {
                        match res {
                            Ok(_) => {}
                            Err(err) => log::error!("{}", err),
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
                    "h | help       print all commands",
                    "q | queue      display the current queue",
                    "c | clear      clear the current queue",
                    "v | v+ | v-    view/increase/decrease logging verbosity",
                    "example        display examples",
                    "exit           exit the program",
                ];

                for cmd in commands {
                    println!("{}", cmd);
                }

                Ok(false)
            }
            "v" => {
                println!(
                    "{}",
                    *cli::LOG_LEVEL_FILTER.lock().expect("mutext not acquired")
                );

                Ok(false)
            }
            "v+" | "v-" => {
                let (before, after) = cli::log_level_after_adjust(&cmd[1..] == "+");

                // we want to display the log change no matter the current or new level
                // do not keep lock around, log filter accesses logs
                *cli::LOG_LEVEL_FILTER.lock().expect("mutext not acquired") =
                    log::LevelFilter::Debug;

                if before == after {
                    log::debug!("log level unchanged [{}]", before);
                } else {
                    log::debug!("changed log level from [{}] to [{}]", before, after);
                }

                *cli::LOG_LEVEL_FILTER.lock().expect("mutext not acquired") = after;

                Ok(false)
            }
            "clear" | "c" => {
                self.values.clear();
                Ok(false)
            }
            "queue" | "q" => {
                println!("queue: {:?}", self.values);
                Ok(false)
            }
            "example" | "eg" => {
                let examples = [
                    "<>            echo program",
                    "<*>>          duplicate echo",
                    "<*[^][_]~>    orders by case, upper first",
                ];

                for eg in examples {
                    println!("{}", eg);
                }

                Ok(false)
            }
            "exit" => Ok(true),
            _ => eyre::bail!(format!("unknown command: '{}'", cmd)),
        }
    }
}
