#![feature(derive_default_enum)]
#![allow(dead_code)]

use crate::repl::Context as ReplContext;

mod exec;
mod helper;
mod io;
mod parse;
mod repl;

fn main() {
    // TODO: fancy error reporting via miette
    // TODO: use raw terminal to allow reply history?
    // TODO: use clap to allow non-repl/non-interactive (without IO-prompts)

    let mut inter = ReplContext::new();
    inter.run();
}
