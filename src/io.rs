use crossterm::style::Stylize;
use std::{
    fmt::Display,
    io::{self, Stderr, Stdin, Stdout, Write},
};

///
/// An IO-Abstraction that allows for simple prompted and colorful
/// output if set and ignoring prompts and colors if not set.
///
#[derive(Debug)]
pub struct Io {
    stdin: Stdin,
    stdout: Stdout,
    stderr: Stderr,
    prepared_in: Option<Vec<String>>,
    pub io_pre_prompt: Option<String>,
    pub io_post_prompt: Option<String>,
    pub log_pre_prompt: Option<String>,
    pub log_post_prompt: Option<String>,
    pub mode: Mode,
}

impl Io {
    ///
    /// Creates a new [`Io`] with the given pre- and post-prompts.
    ///
    pub fn new(mode: Mode) -> Self {
        Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
            stderr: io::stderr(),
            prepared_in: None,
            io_pre_prompt: None,
            io_post_prompt: None,
            log_pre_prompt: None,
            log_post_prompt: None,
            mode,
        }
    }

    ///
    /// Uses the given prompts for input and regular output.
    ///
    pub fn io(self, pre: impl Into<Option<String>>, post: impl Into<Option<String>>) -> Self {
        Self {
            io_pre_prompt: pre.into(),
            io_post_prompt: post.into(),
            ..self
        }
    }

    ///
    /// Uses the given prompts for log output.
    ///
    pub fn log(self, pre: impl Into<Option<String>>, post: impl Into<Option<String>>) -> Self {
        Self {
            log_pre_prompt: pre.into(),
            log_post_prompt: post.into(),
            ..self
        }
    }

    ///
    /// Attatches a [`Vec<String>`] to pop as input before using [`Stdin`],
    /// no prompts are issued on these read requests, even if they are enabled.
    ///
    pub fn attach(self, prepared_in: Vec<String>) -> Self {
        Self {
            prepared_in: Some(prepared_in),
            ..self
        }
    }

    ///
    /// Same as [`Self::read`] but panics on failure.
    ///
    /// ### Panics
    ///
    /// Panics if writing to [`Stdout`]/[`Stderr`] was unsuccessful.
    /// Panics if reading from [`Stdin`] was unsuccessful.
    ///
    pub fn read_expect<'a>(&mut self, prompt: impl Into<Option<&'a str>>) -> String {
        self.read(prompt).expect("io failure")
    }

    ///
    /// Reads a line from [`Stdout`] after sending the given `prompt` to [`Stdout`]. If
    /// a `prompt` is supplied, pre- and post-prompts are applied if set.
    ///
    /// ### Returns
    ///
    /// * [`Ok`]
    ///   * [`Stdout`]/[`Stderr`] was writen to successful
    ///   * [`Stdin`] was read from successful
    /// * [`Err`]
    ///   * [`Stdout`]/[`Stderr`] was writen to unsuccessful, contains the [`io::Error`]
    ///   * [`Stdin`] was read from unsuccessful
    ///
    pub fn read<'a>(&mut self, prompt: impl Into<Option<&'a str>>) -> io::Result<String> {
        if let Some(vec) = &mut self.prepared_in {
            if let Some(input) = vec.pop() {
                return Ok(input);
            }
        }

        if self.mode >= Mode::Colorful {
            if let Some(prompt) = prompt.into() {
                if let Some(pre_prompt) = &self.io_pre_prompt {
                    write!(self.stdout, "{}", pre_prompt)?;
                }

                write!(self.stdout, "{}", prompt.white())?;

                if let Some(post_prompt) = &self.io_post_prompt {
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
    /// Panics if writing to [`Stdout`]/[`Stderr`] was unsuccessful.
    ///
    pub fn write_expect<'a>(
        &mut self,
        kind: Kind,
        prompt: impl Into<Option<&'a str>>,
        msg: impl Display,
    ) {
        self.write(kind, prompt, msg).expect("io failure")
    }

    ///
    /// Writes the given msg to [`Stdout`] with an optional `prompt`. If
    /// a `prompt` is supplied, pre- and post-prompts are applied if set.
    ///
    /// ### Returns
    ///
    /// * [`Ok`]
    ///   * [`Stdout`]/[`Stderr`] was writen to successful
    /// * [`Err`]
    ///   * [`Stdout`]/[`Stderr`] was writen to unsuccessful, contains the [`io::Error`]
    ///
    pub fn write<'a>(
        &mut self,
        kind: Kind,
        prompt: impl Into<Option<&'a str>>,
        msg: impl Display,
    ) -> io::Result<()> {
        // ignore muted, or log messages on lean
        if self.mode == Mode::Muted || (self.mode == Mode::Lean && kind > Kind::Output) {
            return Ok(());
        }

        let (buffer, prompt, pre, post): (&mut dyn Write, _, _, _) = match kind {
            Kind::Output => (
                &mut self.stdout,
                prompt
                    .into()
                    .map(|prompt| prompt.stylize())
                    .or_else(|| Some("out".white())),
                &self.io_pre_prompt,
                &self.io_post_prompt,
            ),
            Kind::Info => (
                &mut self.stderr,
                prompt
                    .into()
                    .map(|prompt| prompt.stylize())
                    .or_else(|| Some("info".green())),
                &self.log_pre_prompt,
                &self.log_post_prompt,
            ),
            Kind::Warning => (
                &mut self.stderr,
                prompt
                    .into()
                    .map(|prompt| prompt.stylize())
                    .or_else(|| Some("warn".yellow())),
                &self.log_pre_prompt,
                &self.log_post_prompt,
            ),
            Kind::Error => (
                &mut self.stderr,
                prompt
                    .into()
                    .map(|prompt| prompt.stylize())
                    .or_else(|| Some("error".red())),
                &self.log_pre_prompt,
                &self.log_post_prompt,
            ),
        };

        if self.mode >= Mode::Colorful {
            if let Some(prompt) = prompt {
                if let Some(pre_prompt) = pre {
                    write!(buffer, "{}", pre_prompt)?;
                }

                write!(buffer, "{}", prompt)?;

                if let Some(post_prompt) = post {
                    write!(buffer, "{}", post_prompt)?;
                }
            }
        }

        writeln!(buffer, "{}", msg)?;
        buffer.flush()?;

        Ok(())
    }
}

///
/// The mode at which IO is used, used to turn off prompts and colors for non-tty input/output.
///
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[repr(u8)]
pub enum Mode {
    ///
    /// Hides input prompts and all output and logging.
    ///
    Muted = 0,

    ///
    /// Lean input/output, no prompts or colors are used and no log messages are issued,
    /// useful for piped input/output.
    ///
    #[default]
    Lean = 1,

    ///
    /// Colorful prompts and logs are used for a interactive input/output.
    ///
    Colorful = 2,
}

///
/// The kind of output to print, determines prompts and colors if enabled.
///
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[repr(u8)]
pub enum Kind {
    ///
    /// Output is printed as with the output prompts and styling to [`Stdout`].
    ///
    #[default]
    Output = 0,

    ///
    /// Output is printed as info with the log prompts and styling to [`Stderr`].
    ///
    Info = 1,

    ///
    /// Output is printed as warning with the log prompts and styling to [`Stderr`].
    ///
    Warning = 2,

    ///
    /// Output is printed as error with the log prompts and styling to [`Stderr`].
    ///
    Error = 3,
}
