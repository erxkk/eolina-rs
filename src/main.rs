#![feature(derive_default_enum, generator_trait, iter_intersperse, once_cell)]
#![allow(dead_code)]

mod cli;
mod helper;
mod io;
mod parse;
mod program;
mod repl;

fn main() -> eyre::Result<()> {
    // TODO: post v1: program analysis and fancy parsing error reporting via miette
    // TODO: post v1: use raw terminal to allow reply history?

    color_eyre::install()?;
    cli::Eolina::new().run()?;
    Ok(())
}
