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
    execs: HashMap<String, Executor>,
}

impl ReplContext {
    ///
    /// Creates a new [`ReplContext`].
    ///
    pub fn new() -> Self {
        Self {
            execs: HashMap::new(),
        }
    }

    ///
    /// Continously runs this repl context until the program is prompted to exit.
    ///
    /// ### Returns
    /// This function does not return.
    ///
    pub fn run(&mut self) -> ! {
        'outer: loop {
            print!(">>> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
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
                        println!("    error: {}", err);
                    }
                }
                continue 'outer;
            }

            'inner: for res in Executor::new(Rc::new(input)) {
                if let Some(err) = res.err() {
                    eprintln!("    error: {}", err);
                    break 'inner;
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
    fn command(&mut self, cmd: &str) -> Result<(), String> {
        match cmd {
            "exit" | "quit" | "q" => process::exit(0),
            "help" | "h" | "?" => {
                println!(
                    r#"    exit | quit | q # exits the program
    help | h | ?    # prints all commands
    s               # saves a program `!s sort <*[^][_]~>`
    c               # calls a program `!s sort`
    r               # removes a program `!r sort`"#
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
                println!(
                    r#"    <>          # echo program
    <*>>        # duplicate echo
    <*[^][_]~>  # orders by case, upper first"#
                );
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
                            println!("    running: `{}`: {}", name, exec.input());

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
                            println!("removed program: `{}`: {}", name, exec.input());
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
