use super::Error;
use crate::{
    io::{self, Io},
    parse::LazyGen,
    program, repl,
};
use clap::{crate_version, App, AppSettings, Arg, Shell, SubCommand};
use std::{borrow::Cow, collections::VecDeque, fs, io::Read};

///
/// Encapsualtes the whole command line application.
///
pub struct Eolina<'a, 'b> {
    app: App<'a, 'b>,
}

impl<'a, 'b> Eolina<'a, 'b> {
    ///
    /// Creates a new [`Eolina`] to use for cli arg parsing.
    ///
    pub fn new() -> Eolina<'static, 'static> {
        // app
        let mut app = App::new("eolina-rs")
            .author("Erik Bünnig (github.com/erxkk)")
            .version(crate_version!())
            .about("A cli interface for interpreting Eolina programs")
            .long_about(
                "\
A cli interface for Eolina, an esotheric string manipulation language.

There may be minor differences in the parsing or interpretation of the
original and my implementation.",
            )
            .global_settings(&[
                AppSettings::ArgsNegateSubcommands,
                AppSettings::ColoredHelp,
                AppSettings::DeriveDisplayOrder,
                AppSettings::DontCollapseArgsInUsage,
            ]);

        // global options
        app = app.args(&[
            Arg::with_name("color")
                .long("color")
                .short("c")
                .alias("colour")
                .help("Whether or not to use colored output")
                .value_name("MODE")
                .default_value("auto")
                .possible_values(&["on", "auto", "off"]),
            Arg::with_name("quiet")
                .long("quiet")
                .short("q")
                .help("Whether or not there should be any log messages"),
        ]);

        // args
        app = app.args(&[
            Arg::with_name("program")
                .value_name("PROGRAM|PATH")
                .help("A program or path to a file containing a program")
                .required(false),
            Arg::with_name("inputs")
                .value_name("INPUTS")
                .help("The optional inputs to the programm if it requires any")
                .required(false)
                .requires("program")
                .multiple(true),
        ]);

        // sub commands
        app = app.subcommands([
            SubCommand::with_name("repl").about("Enters an interactive read-eval-print-loop"),
            SubCommand::with_name("completions")
                .about("Generates a completion file for the given shell")
                .arg(
                    Arg::with_name("shell")
                        .long("shell")
                        .short("s")
                        .value_name("SHELL|ALL")
                        .help("The shell to generateo completions for supported shells [bash|elvish|fish|pwsh|zsh]")
                        .required(false),
                )
                .arg(
                    Arg::with_name("path")
                        .long("path")
                        .short("p")
                        .value_name("PATH")
                        .help("The folder to save the completion file(s) to, defaults to the value of $PWD")
                        .required(false),
                ),
        ]);

        Eolina { app }
    }

    ///
    /// Consumes the [`Eolina`] instance and starts the argument parsing process.
    ///
    /// ### Returns
    ///
    /// * [`Ok`]
    ///   * the program was run successfully
    /// * [`Err`]
    ///   * the program arguments are invalid
    ///   * an exec/repl context failed
    ///
    pub fn run(mut self) -> Result<(), Error> {
        // let clap handle --version/--help etc
        let matches = self.app.clone().get_matches();

        let mode = if matches.is_present("quiet") {
            io::Mode::Muted
        } else {
            match (
                matches
                    .value_of("color")
                    .expect("color has a defautl value"),
                atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stdin),
            ) {
                ("on" | "auto", true) => io::Mode::Colorful,
                ("on", false) => {
                    return Err(Error::User(
                        "`color=on` is not allowed in non-tty env".to_owned(),
                    ))
                }
                ("auto", false) | ("off", _) => io::Mode::Lean,
                _ => unreachable!(),
            }
        };

        // args are always Some() if a subcommand was used
        match matches.subcommand() {
            ("repl", Some(_)) => cmd_repl(mode),
            ("completions", Some(args)) => {
                cmd_completions(mode, args.value_of("shell"), args.value_of("path"))
            }
            // having both subcommands and required args on the first level doesn't work well with
            // AppSettings::ArgRequiredElseHelp, so we handle it ourselves
            ("", None) => {
                if let Some(program) = matches.value_of("program") {
                    cmd_eval(mode, program, matches.values_of("inputs"))
                } else {
                    self.app.print_help()?;
                    Ok(())
                }
            }
            _ => Ok(()),
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
///   * the exec was run succesfully
/// * [`Err`]
///   * the program was was a file path but an error occured opening the file
///   * the program file could not be read
///   * the program was neither a path nor a valid program
///   * the program context failed
///
fn cmd_eval<'a>(
    mode: io::Mode,
    program: &'a str,
    inputs: Option<impl DoubleEndedIterator<Item = &'a str>>,
) -> Result<(), Error> {
    let mut queue = VecDeque::new();
    let mut file_contents = String::new();

    let input = {
        // try to open as file first
        match fs::File::options().read(true).open(program) {
            Ok(mut file) => {
                // we do not buffer reading as we read the whole contents once
                file.read_to_string(&mut file_contents)?;
                &file_contents
            }
            Err(err) => {
                // failed beacuse of any other error than file not found
                if err.kind() != std::io::ErrorKind::NotFound {
                    return Err(Error::Io(err));
                }

                // try using the input directly
                program
            }
        }
    };

    let mut io = Io::new(mode)
        .io("[".to_owned(), "]: ".to_owned())
        .log("[".to_owned(), "]: ".to_owned());

    if let Some(inputs) = inputs {
        // reverse because they are poped back to front
        io = io.attach(inputs.rev().map(ToOwned::to_owned).collect());
    }

    // create an executor context
    let context = program::Context::new(input, LazyGen::new(input).eager()?, &mut io, &mut queue);

    // execute it
    context.run()?;

    Ok(())
}

///
/// Executes the repl command, which executes programs interactively.
///
/// **Note**: Program errors **are not** propagated.
///
/// ### Returns
///
/// * [`Ok`]
///   * the repl was exited normally
/// * [`Err`]
///   * the program was was a file path but an error occured opening the file
///   * the program file could not be read
///   * the program was neither a path nor a valid program
///   * the repl context failed
///
fn cmd_repl(mode: io::Mode) -> Result<(), Error> {
    if atty::isnt(atty::Stream::Stdout) || atty::isnt(atty::Stream::Stdin) {
        return Err(Error::User("cannot start repl in a non-tty env".to_owned()));
    }

    let mut io = Io::new(mode).log("[".to_owned(), "]: ".to_owned());

    let mut exec_io = Io::new(mode)
        .io("[".to_owned(), "]: ".to_owned())
        .log("[".to_owned(), "]: ".to_owned());

    let mut context = repl::Context::new(&mut io, &mut exec_io);

    context.run()?;

    Ok(())
}

///
/// Executes the completions command, which generates a completion file(s) for a given shell.
///
/// ### Returns
///
/// * [`Ok`]
///   * the completion file was successfully generated
/// * [`Err`]
///   * the program was was a file path but an error occured opening the file
///   * the program file could not be read
///   * the program was neither a path nor a valid program
///   * the repl context failed
///
fn cmd_completions(mode: io::Mode, shell: Option<&str>, path: Option<&str>) -> Result<(), Error> {
    let mut io = Io::new(mode).log("[".to_owned(), "]: ".to_owned());

    let shell = if let Some(shell) = shell {
        Cow::Borrowed(shell)
    } else {
        let shell = std::env::var("SHELL")?;
        io.write_expect(io::Kind::Info, None, format!("using $SHELL '{}'", shell));

        Cow::Owned(
            shell
                .split('/')
                .last()
                .expect("split always returns at least self")
                .to_owned(),
        )
    };

    // account for non set and absolute path
    let shell = match shell.as_ref() {
        "ALL" | "all" => None,
        "bash" | "/bin/bash" => Some(Shell::Bash),
        "elvish" | "/bin/elvish" => Some(Shell::Elvish),
        "fish" | "/bin/fish" => Some(Shell::Fish),
        "pwsh" | "/bin/pwsh" => Some(Shell::PowerShell),
        "zsh" | "/bin/zsh" => Some(Shell::Zsh),
        _ => return Err(Error::User(format!("unknown shell: '{}'", shell))),
    };

    let path = if let Some(path) = path {
        Cow::Borrowed(path)
    } else {
        let pwd = std::env::var("PWD")?;
        io.write_expect(io::Kind::Info, None, format!("using $PWD '{}'", pwd));
        Cow::Owned(pwd)
    };

    let mut app = Eolina::new().app;
    if let Some(shell) = shell {
        app.gen_completions("eolina-rs", shell, path.as_ref());
    } else {
        for shell in [
            Shell::Bash,
            Shell::Elvish,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Zsh,
        ] {
            app.gen_completions("eolina-rs", shell, path.as_ref());
        }
    }

    Ok(())
}
