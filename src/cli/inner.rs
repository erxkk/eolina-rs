use crate::{
    exec,
    io::{self, Io},
    repl,
};
use clap::{App, Arg, Shell, SubCommand};
use std::{collections::VecDeque, fs, io::Read};

///
/// Encapsualtes the whole command line application.
///
pub struct Eolina<'a, 'b> {
    app: App<'a, 'b>,
}

impl<'a, 'b> Eolina<'a, 'b> {
    ///
    /// Creates a new [`App`] to use for cli arg parsing.
    ///
    pub fn new() -> Eolina<'static, 'static> {
        let path_validator = |str: String| match fs::try_exists(&str) {
            Ok(exists) => {
                if exists {
                    Ok(())
                } else {
                    Err(format!("path does not exist: `{}`", str))
                }
            }
            Err(err) => Err(format!("cannot access path: `{}`", err)),
        };

        let app = App::new("eolina-rs")
            .about("a cli interface for executing and interpreting eolina programs")
            .arg(
                Arg::with_name("color")
                    .long("color")
                    .short("c")
                    .alias("colour")
                    .takes_value(true)
                    .default_value("auto")
                    .possible_value("on")
                    .possible_value("auto")
                    .possible_value("off"),
            )
            .subcommand(
                SubCommand::with_name("exec")
                    .about("executes a program given in a file")
                    .arg(
                        Arg::with_name("file")
                            .required(true)
                            .validator(path_validator)
                            .possible_value("always")
                            .possible_value("auto")
                            .possible_value("never"),
                    ),
            )
            .subcommand(
                SubCommand::with_name("eval")
                    .about("evaluates a program given directly as arguments")
                    .arg(Arg::with_name("program").required(true))
                    .arg(Arg::with_name("inputs").required(false).multiple(true)),
            )
            .subcommand(
                SubCommand::with_name("repl").about("enters an interactive read-eval-print-loop"),
            )
            .subcommand(
                SubCommand::with_name("completions")
                    .about("generates a completion file for the given shell")
                    .arg(
                        Arg::with_name("shell")
                            .required(true)
                            .possible_value("bash")
                            .possible_value("elvish")
                            .possible_value("fish")
                            .possible_value("pwsh")
                            .possible_value("zsh"),
                    )
                    .arg(Arg::with_name("path").required(true)),
            );

        Eolina { app }
    }

    pub fn run(self) -> color_eyre::Result<()> {
        // let clap handle --version/--help etc
        let matches = self.app.get_matches();

        // TODO: move to separate function + color check
        let mut io = Io::new(
            match matches.value_of("color").expect("color has default value") {
                "on" => {
                    if atty::is(atty::Stream::Stdout) {
                        io::Mode::Colorful
                    } else {
                        eyre::bail!("`color=on` is not allowed in non-tty env");
                    }
                }
                "auto" => {
                    if atty::is(atty::Stream::Stdout) {
                        io::Mode::Colorful
                    } else {
                        io::Mode::Lean
                    }
                }
                "off" => io::Mode::Lean,
                _ => unreachable!(),
            },
        );

        // args are always Some() if a subcommand was used
        match matches.subcommand() {
            ("exec", Some(args)) => {
                cmd_exec(&mut io, args.value_of("file").expect("file is required"))
            }
            ("eval", Some(args)) => cmd_eval(
                &mut io,
                args.value_of("program").expect("program is required"),
                args.values_of("inputs"),
            ),
            ("repl", Some(_)) => cmd_repl(&mut io),
            ("completions", Some(args)) => cmd_completions(
                &mut io,
                args.value_of("shell").expect("shell is required"),
                args.value_of("path").expect("path is required"),
            ),
            _ => Ok(()),
        }
    }
}

fn cmd_exec(io: &mut Io, path: &str) -> color_eyre::Result<()> {
    // read all file contents into a string
    let input = {
        // we do not bugffer reading as we read the whole contents once
        let mut file = fs::File::options()
            .read(true)
            .open(path)
            .expect("path validator failed");

        let mut string = String::new();
        file.read_to_string(&mut string)?;
        string
    };

    let mut queue = VecDeque::new();

    // create an executor context
    let context = exec::Context::new(&input, io, &mut queue);

    // eagerly collect/execute it
    context.collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

// TODO: pipe inputs into Io if given directly
fn cmd_eval<'a>(
    io: &mut Io,
    program: &'a str,
    _inputs: Option<impl Iterator<Item = &'a str>>,
) -> color_eyre::Result<()> {
    let mut queue = VecDeque::new();

    // create an executor context
    let context = exec::Context::new(program, io, &mut queue);

    // eagerly collect/execute it
    context.collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

fn cmd_repl(_io: &mut Io) -> color_eyre::Result<()> {
    if atty::isnt(atty::Stream::Stdout) || atty::isnt(atty::Stream::Stdin) {
        eyre::bail!("cannot start repl in a non-tty env");
    }

    let mut context = repl::Context::new();
    context.run()
}

// TODO: make args optional
fn cmd_completions<'a>(_io: &mut Io, shell: &'a str, path: &'a str) -> color_eyre::Result<()> {
    let shell = match shell {
        "bash" => Shell::Bash,
        "elvish" => Shell::Elvish,
        "fish" => Shell::Fish,
        "pwsh" => Shell::PowerShell,
        "zsh" => Shell::Zsh,
        _ => unreachable!("invalid shell passed through `{}`", shell),
    };

    Eolina::new().app.gen_completions("eolina-rs", shell, path);
    Ok(())
}
