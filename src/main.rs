#![allow(dead_code)]

use crate::repl::ReplContext;

mod exec;
mod helper;
mod parse;
mod repl;

fn main() {
    let mut inter = ReplContext::new();
    inter.run();
}
