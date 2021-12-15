use super::Result as CliResult;
use crate::{
    exec,
    io::{self, Io},
    repl,
};
use clap::{App, Arg, Shell, SubCommand};
use std::{fs, io::Read};

pub struct Eolina<'a, 'b> {
    app: App<'a, 'b>,
    io: io::Io,
}

impl<'a, 'b> Eolina<'a, 'b> {
    ///
    /// Creates a new [`App`] to use for cli arg parsing.
    ///
    pub fn new() -> Eolina<'static, 'static> {
        let shell_validator = |str: String| {
            match str.as_bytes() {
                b"bash" | b"evlish" | b"fish" | b"pwsh" | b"zsh" => Ok(()),
                _=> Err(format!("Unknown or unsupported shell `{}`, supported shells are (bash/elvish/fish/pwsh/zsh)", str))
            }
        };

        let path_validator = |str: String| match fs::try_exists(&str) {
            Ok(exists) => {
                if exists {
                    Ok(())
                } else {
                    Err(format!("Path does not exist: `{}`", str))
                }
            }
            Err(err) => Err(format!("Cannot access path: `{}`", err)),
        };

        let app = App::new("eolina-rs")
            .about("a cli interface for executing and interpreting eolina programs")
            .subcommand(
                SubCommand::with_name("exec")
                    .about("executes a program given in a file")
                    .arg(
                        Arg::with_name("file")
                            .required(true)
                            .validator(path_validator),
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
                            .validator(shell_validator),
                    )
                    .arg(Arg::with_name("path").required(true)),
            );

        Eolina {
            app,
            io: Io::new(io::Mode::Lean),
        }
    }

    pub fn run(mut self) -> CliResult {
        // let clap handle --version/--help etc
        let matches = self.app.get_matches();

        // args are always Some() if a subcommand was used
        match matches.subcommand() {
            ("exec", Some(args)) => cmd_exec(
                &mut self.io,
                args.value_of("file").expect("file is required"),
            ),
            ("eval", Some(args)) => cmd_eval(
                &mut self.io,
                args.value_of("program").expect("program is required"),
                args.values_of("inputs"),
            ),
            ("repl", Some(_)) => cmd_repl(&mut self.io),
            ("completions", Some(args)) => cmd_completions(
                &mut self.io,
                args.value_of("shell").expect("shell is required"),
                args.value_of("path").expect("path is required"),
            ),
            _ => Ok(()),
        }
    }
}

fn cmd_exec(io: &mut Io, path: &str) -> CliResult {
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

    // create an executor context
    let mut context = exec::Context::new(input);

    // eagerly collect/execute it
    context.iter(io).collect::<Result<Vec<_>, exec::Error>>()?;

    // reset the context
    context.reset();
    Ok(())
}

// TODO: pipe inputs into Io if given directly
fn cmd_eval<'a>(
    io: &mut Io,
    program: &'a str,
    _inputs: Option<impl Iterator<Item = &'a str>>,
) -> CliResult {
    // create an executor context
    let mut context = exec::Context::new(program.to_owned());

    // eagerly collect/execute it
    context.iter(io).collect::<Result<Vec<_>, exec::Error>>()?;

    // reset the context
    context.reset();
    Ok(())
}

// TODO: make reply run a result
fn cmd_repl(_io: &mut Io) -> CliResult {
    let mut context = repl::Context::new();
    context.run();
    // Ok(())
}

// TODO: make args optional
fn cmd_completions<'a>(_io: &mut Io, shell: &'a str, path: &'a str) -> CliResult {
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
