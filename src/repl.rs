use color_eyre::Report;

use crate::{
    cli,
    parse::EagerGen,
    program::{Context as ProgramContext, Value},
};
use std::{
    collections::VecDeque,
    io::{self, Write},
    ops::{Generator, GeneratorState},
    pin::Pin,
};

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
    pub fn run(&mut self) -> color_eyre::Result<()> {
        'outer: loop {
            print!(">>> ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

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
                    Err(err) => log_error_chain(err),
                }
                continue 'outer;
            }

            let gen = match EagerGen::new(&input) {
                Ok(gen) => gen,
                Err(err) => {
                    log_error_chain(err);
                    continue 'outer;
                }
            };

            let mut program = ProgramContext::new(&input, gen, None, &mut self.values, true);

            'inner: loop {
                match Pin::new(&mut program).resume(()) {
                    GeneratorState::Yielded(_) => continue 'inner,
                    GeneratorState::Complete(res) => {
                        match res {
                            Ok(_) => {}
                            Err(err) => log_error_chain(err),
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
    fn command(&mut self, cmd: &str) -> color_eyre::Result<bool> {
        match cmd {
            "help" | "h" => {
                // TODO: abstract with box drawing crate
                // same for tokens + example, add cli commands
                let commands = [
                    "  h |  help    print all commands",
                    "  q | queue    display the current queue",
                    "  c | clear    clear the current queue",
                    "v | v+ | v-    view/increase/decrease logging verbosity",
                    "     tokens    display all token descriptions",
                    "    example    display examples",
                    "       exit    exit the program",
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
                // do not keep lock around while logging or this will deadlock
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
            "tokens" => {
                let tokens = [
                    "Token    take:push description",
                    "Basic Tokens:",
                    "    <    0:1       input",
                    "    >    1:0       output",
                    "    ~    2:1       concat",
                    "    *    1:2       duplicate",
                    "   @x    x:x       rotate queue x times",
                    "Checks:",
                    "    v    1:1       check all ascii vowel",
                    "    c    1:1       check all ascii consonant",
                    "    _    1:1       check all ascii lower",
                    "    ^    1:1       check all ascii upper",
                    "Transforms:",
                    "    .    1:1       join array elements to string",
                    "  /x/    1:1       splits string by literal or into chars if x not given",
                    "|x.y|    1:1       slices by abs or rel indecies",
                    "  |x|    1:1       indexes by abs or rel index",
                    "  [x]    1:1       filter all by x: Checks",
                    "  {x}    1:1       map all by x: Maps",
                    "Maps:",
                    "    _    ---       to ascii lower case",
                    "    ^    ---       to ascii upper case",
                    "    %    ---       to swaped ascii case",
                ];

                for token in tokens {
                    println!("{}", token);
                }

                Ok(false)
            }
            "example" | "eg" => {
                let examples = [
                    "        <>    echo program",
                    "      <*>>    duplicate echo",
                    "<*[^][_]~>    orders by case, upper first",
                    "   <|.-3|>    relative slicing like [..len - 3]",
                ];

                for eg in examples {
                    println!("{}", eg);
                }

                Ok(false)
            }
            "exit" => Ok(true),
            _ => color_eyre::eyre::bail!(format!("unknown command: '{}'", cmd)),
        }
    }
}

///
/// Follows the chain of an error and logs each error subsequently.
///
fn log_error_chain(err: Report) {
    for (idx, err) in err.chain().enumerate() {
        log::error!("{}: {}", idx, err);
    }
}
