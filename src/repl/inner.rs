use crate::exec::Executor;
use std::{
    collections::HashMap,
    io::{self, Write},
    process,
    rc::Rc,
};

///
/// A repl context, contains input and output.
///
pub struct ReplContext {
    stdin: io::Stdin,
    stdout: io::Stdout,
    execs: HashMap<String, Executor>,
}

impl ReplContext {
    ///
    /// Creates a new [`ReplContext`] with stdin and stdout as input and output.
    ///
    pub fn new() -> Self {
        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
            execs: HashMap::new(),
        }
    }

    ///
    /// Continously runs this repl context until the program is prompted to exit.
    ///
    pub fn run(&mut self) -> ! {
        'outer: loop {
            write!(self.stdout, ">>> ").expect("unrecoverable error during writing");
            self.stdout.flush().expect("could not flush");
            let mut input = String::new();
            self.stdin
                .read_line(&mut input)
                .expect("unrecoverable error during reading");

            // truncate the '\n'
            match input.pop() {
                Some('\n') => {}
                Some(x) => input.push(x),
                None => {
                    continue 'outer;
                }
            }

            // skip
            if input.is_empty() {
                continue 'outer;
            }

            if let Some(input) = input.strip_prefix('!') {
                match self.command(input) {
                    Ok(_) => {}
                    Err(err) => {
                        writeln!(self.stdout, "error: {}", err)
                            .expect("unrecoverable error during writing");
                    }
                }
                continue 'outer;
            }

            'inner: for res in Executor::new(Rc::new(input)) {
                if let Some(err) = res.err() {
                    eprintln!("error: {}", err);
                    break 'inner;
                }
            }
        }
    }

    ///
    /// Executes a command for this context.
    ///
    fn command(&mut self, cmd: &str) -> Result<(), String> {
        match cmd {
            "exit" | "quit" | "q" => process::exit(0),
            "help" | "h" | "?" => {
                writeln!(
                    self.stdout,
                    r#"commands:
exit: exits the program
help: prints all commands"#
                )
                .expect("unrecoverable error during writing");
                Ok(())
            }
            "list" | "ls" => {
                writeln!(self.stdout, "programs:").expect("unrecoverable error during writing");
                for (name, exec) in self.execs.iter() {
                    writeln!(self.stdout, "   `{}`: {}", name, exec.input())
                        .expect("unrecoverable error during writing")
                }
                Ok(())
            }
            "example" | "eg" => {
                writeln!(
                    self.stdout,
                    r#"examples:
<>          # echo program
<*>>        # duplicate echo
<*[^][_]~>  # oders by case, upper first"#
                )
                .expect("unrecoverable error during writing");
                Ok(())
            }
            x => match x.as_bytes() {
                [b's', b' ', ..] => {
                    let x = &x[2..];
                    let mut iter = x.split_ascii_whitespace();

                    let name = iter
                        .next()
                        .ok_or_else(|| "missing program name".to_owned())?
                        .to_owned();

                    let program = iter.next().ok_or_else(|| "missing program".to_owned())?;
                    let program = Rc::new(program.to_owned());
                    self.execs.insert(name, Executor::new(Rc::clone(&program)));
                    Ok(())
                }
                [b'c', b' ', ..] => {
                    let x = &x[2..];
                    let mut iter = x.split_ascii_whitespace();

                    let name = iter
                        .next()
                        .ok_or_else(|| "missing program name".to_owned())?;

                    match self.execs.get_mut(name) {
                        Some(exec) => {
                            writeln!(self.stdout, "running: `{}`: {}", name, exec.input())
                                .expect("unrecoverable error during writing");

                            for res in exec.by_ref() {
                                if let Some(err) = res.err() {
                                    eprintln!("error: {}", err);
                                    break;
                                }
                            }
                            exec.reset()
                                .map_err(|err| format!("error resetting: {}", err))?;
                            Ok(())
                        }
                        None => Err(format!("unknown program: `{}`", x)),
                    }
                }
                [b'r', b' ', ..] => {
                    let x = &x[2..];
                    let mut iter = x.split_ascii_whitespace();

                    let name = iter
                        .next()
                        .ok_or_else(|| "missing program name".to_owned())?;

                    match self.execs.remove(name) {
                        Some(exec) => {
                            writeln!(self.stdout, "removed program: `{}`: {}", name, exec.input())
                                .expect("unrecoverable error during writing");
                            Ok(())
                        }
                        None => Err(format!("unknown program: `{}`", x)),
                    }
                }
                _ => Err(format!("unknown command: `{}`", x)),
            },
        }
    }
}
