//#![allow(dead_code)]

use crate::repl::ReplContext;

mod exec;
mod helper;
mod parse;
mod repl;

fn main() {
    // TODO: improve allocs overall, reduce copies and clones
    // TODO: fancy error reporting via miette
    // TODO: use raw terminal to allow reply history?
    // TODO: use clap to allow non-repl/non-interactive (without IO-prompts)
    let mut inter = ReplContext::new();
    inter.run();
}
