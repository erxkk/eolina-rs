use crossterm::style::Stylize;
use std::{
    fmt::Display,
    io::{self, Stderr, Stdin, Stdout, Write},
};

// TODO: multiline indentation

///
/// An IO-Abstraction that allows for simple prompted or colorful output if avaiable.
///
#[derive(Debug)]
pub struct Io {
    stdin: Stdin,
    stdout: Stdout,
    stderr: Stderr,
    pub out_pre_prompt: Option<String>,
    pub out_post_prompt: Option<String>,
    pub in_pre_prompt: Option<String>,
    pub in_post_prompt: Option<String>,
    pub stdout_mode: Mode,
    pub stdin_mode: Mode,
    pub stderr_mode: Mode,
}

impl Io {
    ///
    /// Creates a new [`Io`].
    ///
    pub fn new(mode: Mode) -> Self {
        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
            stderr: io::stderr(),
            in_pre_prompt: None,
            in_post_prompt: None,
            out_pre_prompt: None,
            out_post_prompt: None,
            stdin_mode: mode,
            stdout_mode: mode,
            stderr_mode: mode,
        }
    }

    ///
    /// Creates a new [`Io`] with the given prompt delimiterss.
    ///
    pub fn with(
        mode: Mode,
        in_pre_prompt: impl Into<Option<String>>,
        in_post_prompt: impl Into<Option<String>>,
        out_pre_prompt: impl Into<Option<String>>,
        out_post_prompt: impl Into<Option<String>>,
    ) -> Self {
        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
            stderr: io::stderr(),
            out_pre_prompt: out_pre_prompt.into(),
            out_post_prompt: out_post_prompt.into(),
            in_pre_prompt: in_pre_prompt.into(),
            in_post_prompt: in_post_prompt.into(),
            stdin_mode: mode,
            stdout_mode: mode,
            stderr_mode: mode,
        }
    }

    ///
    /// Same as [`Self::read`] but panics on failure.
    ///
    /// ### Panics
    ///
    /// Panics if writing to [`Stdout`] was unsuccessful.
    /// Panics if reading from [`Stdin`] was unsuccessful.
    ///
    pub fn read_expect<'a, 'b, T>(
        &mut self,
        prompt: impl Into<Option<T>>,
        context: impl Into<Option<&'b str>>,
    ) -> String
    where
        T: Stylize + Display,
        T::Styled: Display,
    {
        self.read(prompt, context).expect("io failure")
    }

    ///
    /// Reads a line from [`Stdin`] after prompting for input with an optional
    /// `promot` and `context`. Prompts and colors are controlled by
    /// the values given on the [`Io`].
    ///
    /// ### Returns
    ///
    /// * [`Ok(())`] if writing and reading was successful
    /// * [`Err(error)`] if writing and reading failed
    ///   * `error` contains the [`io::Error`]
    ///
    pub fn read<'a, 'b, T>(
        &mut self,
        prompt: impl Into<Option<T>>,
        context: impl Into<Option<&'b str>>,
    ) -> io::Result<String>
    where
        T: Stylize + Display,
        T::Styled: Display,
    {
        if self.stdout_mode >= Mode::Prompt {
            if let Some(context) = context.into() {
                if matches!(self.stdout_mode, Mode::Colorful) {
                    write!(self.stdout, "[`{}`] ", context.grey())?;
                } else {
                    write!(self.stdout, "[`{}`] ", context)?;
                }
            }

            if let Some(prompt) = prompt.into() {
                if let Some(pre_prompt) = &self.in_pre_prompt {
                    write!(self.stdout, "{}", pre_prompt)?;
                }

                if matches!(self.stdout_mode, Mode::Colorful) {
                    write!(self.stdout, "{}", prompt.white())?;
                } else {
                    write!(self.stdout, "{}", prompt)?;
                }

                if let Some(post_prompt) = &self.in_post_prompt {
                    write!(self.stdout, "{}", post_prompt)?;
                }

                self.stdout.flush()?;
            }
        }

        let mut ret = String::new();
        self.stdin.read_line(&mut ret)?;

        Ok(ret)
    }

    ///
    /// Same as [`Self::write`] but panics on failure.
    ///
    /// ### Panics
    ///
    /// Panics if writing to [`Stdout`] was unsuccessful.
    ///
    pub fn write_expect<'b>(
        &mut self,
        kind: Kind,
        msg: impl Display,
        context: impl Into<Option<&'b str>>,
    ) {
        self.write(kind, msg, context).expect("io failure")
    }

    ///
    /// Writes the given msg to [`Stdout`] with an optional `context`.
    /// Prompts and colors are controlled by the values given on the [`Io`].
    ///
    /// ### Returns
    ///
    /// * [`Ok(())`] if writing was successful
    /// * [`Err(error)`] if writing failed
    ///   * `error` contains the [`io::Error`]
    ///
    pub fn write<'b>(
        &mut self,
        kind: Kind,
        msg: impl Display,
        context: impl Into<Option<&'b str>>,
    ) -> io::Result<()> {
        match kind {
            Kind::Output | Kind::Info if self.stdout_mode == Mode::Muted => return Ok(()),
            Kind::Warning | Kind::Error if self.stderr_mode == Mode::Muted => return Ok(()),
            _ => {}
        }

        let (buffer, mode, prompt): (&mut dyn Write, _, _) = match kind {
            Kind::Output => (
                &mut self.stdout,
                self.stdout_mode,
                match self.stdout_mode {
                    Mode::Prompt => Some("out".white()),
                    Mode::Colorful => Some("out".white()),
                    _ => None,
                },
            ),
            Kind::Info => (
                &mut self.stdout,
                self.stdout_mode,
                match self.stdout_mode {
                    Mode::Prompt => Some("info".white()),
                    Mode::Colorful => Some("info".green()),
                    _ => None,
                },
            ),
            Kind::Warning => (
                &mut self.stderr,
                self.stderr_mode,
                match self.stderr_mode {
                    Mode::Prompt => Some("warn".white()),
                    Mode::Colorful => Some("warn".yellow()),
                    _ => None,
                },
            ),
            Kind::Error => (
                &mut self.stderr,
                self.stderr_mode,
                match self.stderr_mode {
                    Mode::Prompt => Some("error".white()),
                    Mode::Colorful => Some("error".red()),
                    _ => None,
                },
            ),
        };

        if let Some(context) = context.into() {
            if matches!(mode, Mode::Colorful) {
                write!(buffer, "[`{}`] ", context.grey())?;
            } else {
                write!(buffer, "[`{}`] ", context)?;
            }
        }

        if let Some(prompt) = prompt {
            if let Some(pre_prompt) = &self.out_pre_prompt {
                write!(buffer, "{}", pre_prompt)?;
            }

            write!(buffer, "{}", prompt)?;

            if let Some(post_prompt) = &self.out_post_prompt {
                write!(buffer, "{}", post_prompt)?;
            }
        }

        writeln!(buffer, "{}", msg)?;
        buffer.flush()?;

        Ok(())
    }
}

///
/// The mode at which IO is used, used to turn off prompts and colors for piped input/output.
///
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[repr(u8)]
pub enum Mode {
    ///
    /// Ignores input/output.
    ///
    Muted = 0,

    ///
    /// Lean input/output, no colors or prompts are used, and no info is given
    /// out, useful for piped input/output.
    ///
    #[default]
    Lean = 1,

    ///
    /// Prompts are used for a interactive input/output.
    ///
    Prompt = 2,

    ///
    /// Colorful prompts and errors are used for a interactive input/output.
    ///
    Colorful = 3,
}

///
/// The kind of output to print, determines prompt's and colors if enabled.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[repr(u8)]
pub enum Kind {
    ///
    /// Output is printed as programm output to [`Stdout`]
    ///
    Output = 0,

    ///
    /// Output is printed as information to [`Stdout`] or ignored
    /// if the [`Io`] `stdout_mode` is set to [`Mode::Lean`] or below.
    ///
    Info = 1,

    ///
    /// Output is printed as warning to [`Stderr`] or ignored
    /// if the [`Io`] `stderr_mode` is set to [`Mode::Lean`] or below.
    ///
    Warning = 2,

    ///
    /// Output is printed as error to [`Stderr`] or ignored
    /// if the [`Io`] `stderr_mode` is set to [`Mode::Lean`] or below.
    ///
    Error = 3,
}
