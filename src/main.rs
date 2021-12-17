#![feature(derive_default_enum, iter_intersperse, generator_trait)]
#![allow(dead_code)]

mod cli;
mod helper;
mod io;
mod parse;
mod program;
mod repl;

fn main() -> color_eyre::Result<()> {
    // TODO: post v1: program analysis and fancy parsing error reporting via miette
    // TODO: post v1: use raw terminal to allow reply history?

    color_eyre::install()?;
    cli::Eolina::new().run()?;
    Ok(())
}
