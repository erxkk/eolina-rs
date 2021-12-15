#![feature(derive_default_enum, path_try_exists)]
#![allow(dead_code)]

mod cli;
mod exec;
mod helper;
mod io;
mod parse;
mod repl;

fn main() -> color_eyre::Result<()> {
    // TODO: reduce repl to one running program with stepwise execution
    // TODO: fancy parsing error reporting via miette
    // TODO: use raw terminal to allow reply history?

    color_eyre::install()?;
    crate::cli::Eolina::new().run()?;
    Ok(())
}
