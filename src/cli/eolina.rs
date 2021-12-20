use crate::{parse::LazyGen, program, repl};
use clap::{ArgEnum, IntoApp, Parser, Subcommand};
use std::{
    collections::VecDeque,
    fmt::Display,
    fs,
    io::{self, Read},
    str::FromStr,
};

///
/// An exit code for handling errors that should not be bubbled up to a top-level panic. Runetime
/// errors are handled by panics and return 1.
///
#[derive(Debug)]
#[repr(u8)]
pub enum ExitCode {
    ///
    /// No error.
    ///
    Ok = 0,

    ///
    /// A runtime error occured, this variant is listed for completeness. Runtime errors are
    /// bubbled up with eyre and displayed through a top-level panic, which returns error code 1.
    ///
    #[allow(dead_code)]
    RuntimeFailure = 1,

    ///
    /// Neither a subcommand nor arguments were supplied.
    ///
    MissingArgumentOrSubcommand = 2,
}

///
/// The color mode the app should use.
///
#[derive(ArgEnum, Default, Debug, Clone, Copy)]
enum Color {
    ///
    /// Use colors and prompts.
    ///
    On,

    ///
    /// Don't use colors or prompts.
    ///
    Off,

    ///
    /// Determine suitable mode from environment
    ///
    #[default]
    Auto,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::On => f.write_str("on"),
            Self::Off => f.write_str("off"),
            Self::Auto => f.write_str("auto"),
        }
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            "auto" => Ok(Self::Auto),
            _ => Err(format!("'{}'", s)),
        }
    }
}

// Note: Documentation is left without trailing periods `.` as it is used in clap's help message.

///
/// Encapsualtes the whole command line application.
///
#[derive(Parser, Debug)]
#[clap(
    about,
    version,
    author,
    after_help = "EXIT CODE:
    0    Ok
    1    Runtime Error
    2    Missing Argument/Subcommand"
)]
pub struct Eolina {
    ///
    /// Whether or not to use colored output
    ///
    #[clap(
        short, long, possible_values = ["on", "off", "auto"],
        default_value_t = Default::default()
    )]
    color: Color,

    ///
    /// Show trace information on errors [-t, -tt, -ttt]
    ///
    #[clap(short, long, parse(from_occurrences))]
    trace: usize,

    ///
    /// Log output verbosity [-v, -vv, -vvv]
    ///
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,

    ///
    /// Hide any log messages
    ///
    #[clap(short, long)]
    quiet: bool,

    ///
    /// A program or path to a file containing a program
    ///
    #[clap(value_name = "PROGRAM|PATH")]
    program: Option<String>,

    ///
    /// The optional inputs to the programm if it requires any
    ///
    #[clap(value_name = "INPUTS", requires = "program")]
    inputs: Vec<String>,

    ///
    /// A subcommand to execute.
    ///
    #[clap(subcommand)]
    subcommand: Option<SubCommand>,
}

///
/// A subcommand.
///
#[derive(Subcommand, Debug, PartialEq, Eq)]
enum SubCommand {
    ///
    /// Enters an interactive read-eval-print-loop
    ///
    Repl,
}

impl Eolina {
    ///
    /// Consumes this [`Eolina`] instance and runs the program.
    ///
    /// ### Returns
    ///
    /// * [`Ok`]
    ///   * the program was run successfully, contains the exit code
    /// * [`Err`]
    ///   * the program arguments are invalid
    ///   * an exec/repl context failed
    ///
    pub fn run(self) -> eyre::Result<ExitCode> {
        match self.trace {
            1 => {
                std::env::set_var("RUST_BACKTRACE", "1");
            }
            2 => {
                std::env::set_var("RUST_BACKTRACE", "full");
            }
            3.. => {
                std::env::set_var("RUST_BACKTRACE", "full");
                std::env::set_var("COLORBT_SHOW_HIDDEN", "1");
            }
            _ => {}
        }

        *super::LOG_LEVEL_FILTER.lock().expect("mutex not acquired") = if self.quiet {
            log::LevelFilter::Off
        } else {
            match (self.verbose, self.subcommand == Some(SubCommand::Repl)) {
                (0, false) => log::LevelFilter::Warn,
                (1, false) | (0, true) => log::LevelFilter::Info,
                (2, false) | (1, true) => log::LevelFilter::Debug,
                (3.., false) | (2, true) => log::LevelFilter::Trace,
                _ => log::LevelFilter::Trace,
            }
        };

        let is_fancy = match (self.color, *super::IS_ERR_TTY) {
            (Color::On | Color::Auto, true) => true,
            (Color::On, false) => {
                eyre::bail!("`color=on` is not allowed if stderr is not tty".to_owned());
            }
            (Color::Auto, false) | (Color::Off, _) => false,
        };

        let _ = super::IS_FANCY_CELL.get_or_init(|| is_fancy);

        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}] {}",
                    super::get_prompt(record.level()),
                    message
                ))
            })
            .filter(|meta| {
                meta.level() <= *super::LOG_LEVEL_FILTER.lock().expect("mutex not acquired")
            })
            .chain(io::stderr())
            .apply()?;

        match self.subcommand {
            Some(SubCommand::Repl) => cmd_repl(),
            None => {
                if let Some(program) = self.program {
                    cmd_eval(program, self.inputs)
                } else {
                    let mut app = Self::into_app();
                    app.print_help()?;
                    Ok(ExitCode::MissingArgumentOrSubcommand)
                }
            }
        }
    }
}

///
/// Executes the default command, which takes a program and optional inputs. If no inputs are given
/// they are read form stdin during execution.
///
/// **Note**: Program errors **are** propagated.
///
/// ### Returns
///
/// * [`Ok`]
///   * the exec was run succesfully, contains the exit code
/// * [`Err`]
///   * the program was was a file path but an error occured opening the file
///   * the program file could not be read
///   * the program was neither a path nor a valid program
///   * the program context failed
///
fn cmd_eval(program: String, mut inputs: Vec<String>) -> eyre::Result<ExitCode> {
    let mut queue = VecDeque::new();
    let mut file_contents = String::new();

    let input = {
        // try to open as file first
        match fs::File::options().read(true).open(&program) {
            Ok(mut file) => {
                // we do not buffer reading as we read the whole contents once
                file.read_to_string(&mut file_contents)?;
                file_contents
            }
            Err(err) => {
                // failed beacuse of any other error than file not found
                if err.kind() != io::ErrorKind::NotFound {
                    eyre::bail!(err);
                }

                // try using the input directly
                program
            }
        }
    };

    if !inputs.is_empty() {
        // reverse because they are poped back to front
        inputs.reverse();
    }

    // create an executor context
    let context = program::Context::new(
        &input,
        LazyGen::new(&input),
        Some(inputs),
        &mut queue,
        false,
    );

    // execute it
    context.run()?;

    Ok(ExitCode::Ok)
}

///
/// Executes the repl command, which executes programs interactively.
///
/// **Note**: Program errors **are not** propagated.
///
/// ### Returns
///
/// * [`Ok`]
///   * the repl was exited normally, contains the exit code
/// * [`Err`]
///   * the program was was a file path but an error occured opening the file
///   * the program file could not be read
///   * the program was neither a path nor a valid program
///   * the repl context failed
///
fn cmd_repl() -> eyre::Result<ExitCode> {
    if !*super::IS_IN_TTY || !*super::IS_OUT_TTY || !*super::IS_ERR_TTY {
        eyre::bail!("cannot start repl in a non-tty env".to_owned());
    }

    let mut context = repl::Context::new();
    context.run()?;

    Ok(ExitCode::Ok)
}
