#![feature(derive_default_enum, path_try_exists, explicit_generic_args_with_impl_trait)]
#![allow(dead_code)]

mod cli;
mod exec;
mod helper;
mod io;
mod parse;
mod repl;

fn main() -> color_eyre::Result<()> {
    // TODO: fancy parsing error reporting via miette
    // TODO: use raw terminal to allow reply history?

    color_eyre::install()?;
    crate::cli::Eolina::new().run()?;
    Ok(())
}
