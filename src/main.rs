#![feature(
    derive_default_enum,
    generator_trait,
    iter_intersperse,
    maybe_uninit_array_assume_init,
    once_cell
)]

mod cli;
mod helper;
mod parse;
mod program;
mod repl;
mod token;

use clap::Parser;
use cli::{Eolina, ExitCode};

fn main() -> color_eyre::Result<()> {
    // TODO: post v1: program analysis and fancy parsing error reporting via miette
    // TODO: post v1: use raw terminal to allow reply history?

    color_eyre::install()?;

    match Eolina::parse().run()? {
        ExitCode::Ok => Ok(()),
        err_exit_code => {
            // exiting here is fine, all non primitive values have been dropped
            std::process::exit(err_exit_code as i32)
        }
    }
}
