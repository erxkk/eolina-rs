#![feature(derive_default_enum, path_try_exists)]
#![allow(dead_code)]

mod cli;
mod exec;
mod helper;
mod io;
mod parse;
mod repl;

fn main() {
    // TODO: migrate errors to anyhow/eyre
    // TODO: reduce reply to one running program with stepwise execution
    // TODO: fancy error reporting via miette
    // TODO: use raw terminal to allow reply history?

    crate::cli::Eolina::new().run().unwrap();
}
