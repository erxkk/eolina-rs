use super::Error;
use crate::{
    io::{self, Io},
    parse::EagerGen,
    program, repl,
};
use clap::{ArgEnum, IntoApp, Parser, Subcommand};
use std::{collections::VecDeque, fmt::Display, fs, io::Read, str::FromStr};

///
/// An exit code for handling errors that should not be bubbled up
/// to a top-level panic.
///
#[derive(Debug)]
#[repr(u8)]
pub enum ExitCode {
    ///
    /// No error.
    ///
    Ok = 0,

    ///
    /// Neither a subcommand nor arguments were supplied.
    ///
    MissingArgumentOrSubcommand = 1,

    ///
    /// Invalid program was supplied.
    ///
    InvlidProgram = 2,
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
    1    Missing Argument/Subcommand
    2    Invlid Program"
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
#[derive(Subcommand, Debug)]
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

        let mode = if self.quiet {
            io::Mode::Muted
        } else {
            match (self.color, *io::IS_FULL_TTY) {
                (Color::On | Color::Auto, true) => io::Mode::Colorful,
                (Color::On, false) => {
                    eyre::bail!(Error::User(
                        "`color=on` is not allowed in non-tty env".to_owned(),
                    ));
                }
                (Color::Auto, false) | (Color::Off, _) => io::Mode::Lean,
            }
        };

        match self.subcommand {
            Some(SubCommand::Repl) => cmd_repl(mode),
            None => {
                if let Some(program) = self.program {
                    cmd_eval(mode, program, self.inputs)
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
/// Executes the default command, which takes a program and optional inputs.
/// If no inputs are given they are read form stdin during execution.
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
fn cmd_eval(mode: io::Mode, program: String, inputs: Vec<String>) -> eyre::Result<ExitCode> {
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
                if err.kind() != std::io::ErrorKind::NotFound {
                    eyre::bail!(Error::Io(err));
                }

                // try using the input directly
                program
            }
        }
    };

    let mut io = Io::new(mode)
        .io("[".to_owned(), "]: ".to_owned())
        .log("[".to_owned(), "]: ".to_owned());

    if !inputs.is_empty() {
        // reverse because they are poped back to front
        io = io.attach(inputs.into_iter().rev().collect());
    }

    let gen = match EagerGen::new(&input) {
        Ok(gen) => gen,
        Err(err) => {
            io.write_expect(io::Kind::Error, None, err);
            return Ok(ExitCode::InvlidProgram);
        }
    };

    // create an executor context
    let context = program::Context::new(&input, gen, &mut io, &mut queue);

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
fn cmd_repl(mode: io::Mode) -> eyre::Result<ExitCode> {
    if !*io::IS_FULL_TTY {
        eyre::bail!(Error::User("cannot start repl in a non-tty env".to_owned()));
    }

    let mut io = Io::new(mode).log("[".to_owned(), "]: ".to_owned());

    let mut exec_io = Io::new(mode)
        .io("[".to_owned(), "]: ".to_owned())
        .log("[".to_owned(), "]: ".to_owned());

    let mut context = repl::Context::new(&mut io, &mut exec_io);

    context.run()?;

    Ok(ExitCode::Ok)
}
